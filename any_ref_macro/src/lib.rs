use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Nothing, Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
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

        let struct_body = if generics.type_params().count() == 0 {
            quote! {}.into_token_stream()
        } else {
            let mut punct: Punctuated<TokenStream2, Comma> = Punctuated::new();
            for x in generics.params.iter() {
                let y = match x {
                    GenericParam::Const(c) => {
                        //let n = &c.ident;
                        quote! {()}
                    }
                    GenericParam::Lifetime(l) => {
                        let n = &l.lifetime;
                        quote! {&#n ()}
                    }
                    GenericParam::Type(t) => {
                        let n = &t.ident;
                        quote! {#n}
                    }
                };
                punct.push(y);
            }
            quote! { (core::marker::PhantomData<(#punct,)>) }
        };

        let g_in_i_long = {
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
        let (impl_g_long, _, _) = g_in_i_long.split_for_impl();

        let q: TokenStream = quote! {
            #vis struct #ident #generics #struct_body;

            impl #impl_g_long any_ref::SelfDeref<#bl> for #ident #type_g #where_clause{
                type Target = #target;
            }

            impl #impl_g any_ref::LifetimeDowncast for #ident #type_g #where_clause{
                #[inline]
                fn lifetime_downcast<'_shorter_lifetime_, '_longer_lifetime_: '_shorter_lifetime_>(
                    from: &'_shorter_lifetime_ <Self as SelfDeref<'_longer_lifetime_>>::Target,
                ) -> &'_shorter_lifetime_ <Self as SelfDeref<'_shorter_lifetime_>>::Target {
                    from
                }
            }
        }
        .into();
        //println!("{}\n", q.to_string());
        tstream.extend(q);
    }
    return tstream;
}
