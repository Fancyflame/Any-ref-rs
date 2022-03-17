use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Nothing, Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    BoundLifetimes, GenericParam, Generics, Ident, Token, Type, Visibility, WhereClause,
};

struct ImplItem {
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    bound_lifetimes: BoundLifetimes,
    target: Type,
    where_clause: Option<WhereClause>,
}

struct ImplItemIter(Punctuated<ImplItem, Nothing>);

impl Parse for ImplItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis: Visibility = input.parse()?; //pub
        input.parse::<Token![struct]>()?; //struct
        let ident: Ident = input.parse()?; //StructName
        let generics: Generics = input.parse()?; //<...>
        input.parse::<Token![=]>()?; //=
        let bound_lifetimes: BoundLifetimes = input.parse()?; //for<...>
        let target: Type = input.parse()?; //...
        let where_clause: Option<WhereClause> = input.parse()?; //where ...
        input.parse::<Token![;]>()?; //;
        Ok(ImplItem {
            vis,
            ident,
            generics,
            bound_lifetimes,
            target,
            where_clause,
        })
    }
}

impl Parse for ImplItemIter {
    #[inline]
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ImplItemIter(Punctuated::parse_terminated(input)?))
    }
}

#[proc_macro]
pub fn make_any_ref(input: TokenStream) -> TokenStream {
    let mut tstream = TokenStream::new();
    let punct = parse_macro_input!(input as ImplItemIter);
    for pair in punct.0.into_pairs() {
        let ImplItem {
            vis,
            ident,
            generics,
            bound_lifetimes,
            target,
            where_clause,
        } = pair.into_value();

        let bl = bound_lifetimes.lifetimes.clone();
        let (impl_g, type_g, _) = generics.split_for_impl();
        let g_in_i = {
            let mut g = generics.clone();
            g.params.extend(
                bound_lifetimes
                    .lifetimes
                    .into_iter()
                    .map(|x| GenericParam::Lifetime(x)),
            );
            g
        };

        //impl_g_long refers to generics+bound_lifetimes
        let (impl_g_long, _, _) = g_in_i.split_for_impl();

        let q: TokenStream = quote! {
            #vis struct #ident #generics (core::marker::PhantomData #type_g);

            impl #impl_g_long SelfDeref<#bl> for #ident #type_g #where_clause{
                type Target = #target;
            }

            impl #impl_g LifetimeDowncast for #ident #type_g #where_clause{
                #[inline]
                fn lifetime_downcast<'_shorter_lifetime_, '_longer_lifetime_: '_shorter_lifetime_>(
                    from: &'_shorter_lifetime_ <Self as SelfDeref<'_longer_lifetime_>>::Target,
                ) -> &'_shorter_lifetime_ <Self as SelfDeref<'_shorter_lifetime_>>::Target {
                    from
                }
            }
        }
        .into();
        println!("{}", q.to_string());
        tstream.extend(q);
    }
    return tstream;
}
