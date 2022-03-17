pub mod self_deref;

pub use any_ref_macro::make_any_ref;
pub use self_deref::*;
use stable_deref_trait::StableDeref;
use std::ops::Deref;

#[macro_export]
macro_rules! self_deref_for_type {
    {
        $($vis:vis type $Alias:ident=for<$arg:tt> $Struct:ty;)*
    } => {
        $(
            $vis struct $Alias;

            impl<$arg> $crate::SelfDeref<$arg> for $Alias{
                type Target = $Struct;
            }

            impl LifetimeDowncast for $Alias{
                #[inline]
                fn lifetime_downcast<'a, 'b: 'a>(
                    from: &'a <Self as SelfDeref<'b>>::Target,
                ) -> &'a <Self as SelfDeref<'a>>::Target {
                    from
                }
            }
        )*
    };
}

pub struct AnyRef<O, T: LifetimeDowncast + ?Sized> {
    owner: O,
    holder: <T as SelfDeref<'static>>::Target,
}

pub struct Ref<'a, T: LifetimeDowncast + ?Sized>(&'a <T as SelfDeref<'static>>::Target);

impl<O, _T> AnyRef<O, _T>
where
    O: StableDeref + 'static,
    <O as Deref>::Target: 'static,
    _T: LifetimeDowncast + ?Sized,
{
    pub fn new<T, F>(owner: O, func: F) -> AnyRef<O, T>
    where
        T: LifetimeDowncast + ?Sized,
        F: for<'a> FnOnce(&'a <O as Deref>::Target) -> <T as SelfDeref<'a>>::Target,
    {
        let tar = unsafe { &*(&*owner as *const _) };
        AnyRef {
            holder: func(tar),
            owner: owner,
        }
    }

    pub fn map<T2, F>(self, func: F) -> AnyRef<O, T2>
    where
        T2: LifetimeDowncast + ?Sized,
        F: for<'a> FnOnce(&'a <_T as SelfDeref<'a>>::Target) -> <T2 as SelfDeref<'a>>::Target,
    {
        let tar = unsafe { &*(&self.holder as *const _) };
        AnyRef {
            holder: func(tar),
            owner: self.owner,
        }
    }

    pub fn map_clone<T2, F>(&self, func: F) -> AnyRef<O, T2>
    where
        O: Clone,
        T2: LifetimeDowncast + ?Sized + 'static,
        F: for<'a> FnOnce(&'a <O as Deref>::Target) -> <T2 as SelfDeref<'a>>::Target,
    {
        let o = self.owner.clone();
        let tar = unsafe { &*(&*self.owner as *const _) };
        AnyRef {
            holder: func(tar),
            owner: o,
        }
    }

    #[inline]
    pub fn into_owner(self) -> O {
        self.owner
    }

    #[inline]
    pub fn get<'a>(&'a self) -> &'a <_T as SelfDeref<'a>>::Target {
        <_T as LifetimeDowncast>::lifetime_downcast(&self.holder)
        //$Ref(&self.holder)
    }
}

impl<'a, T> Deref for Ref<'a, T>
where
    T: ?Sized + LifetimeDowncast,
{
    type Target = <T as SelfDeref<'a>>::Target;

    #[inline]
    fn deref(&self) -> &Self::Target {
        <T as LifetimeDowncast>::lifetime_downcast(self.0)
    }
}

pub fn new_any_ref<T, O, F>(owner: O, func: F) -> AnyRef<O, T>
where
    O: StableDeref + 'static,
    <O as Deref>::Target: 'static,
    T: LifetimeDowncast + ?Sized,
    F: for<'a> FnOnce(&'a <O as Deref>::Target) -> <T as SelfDeref<'a>>::Target,
{
    let tar = unsafe { &*(&*owner as *const _) };
    AnyRef {
        holder: func(tar),
        owner: owner,
    }
}

#[test]
fn test() {
    use crate as any_ref;

    make_any_ref! {
        pub struct Foo=for<'a> &'a str;
        pub struct Bar<T:'static> = for<'a> Vec<&'a T>;
        struct AAA<T:'static,const U:usize> = for<'lifetime> std::marker::PhantomData<&'lifetime (T,)>;
    }

    let moved_ar;
    {
        let num = Box::new((1, 2, 3, 4));
        let ar = new_any_ref::<Bar<u16>, _, _>(num, |x| vec![&x.0, &x.1, &x.2, &x.3]);
        moved_ar = ar;
    }
    assert_eq!(moved_ar.get(), &vec![&1, &2, &3, &4]);
}
