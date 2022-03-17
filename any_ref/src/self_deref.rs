use super::make_any_ref;
use crate as any_ref;

pub trait SelfDeref<'a> {
    type Target: 'a;
}

pub trait LifetimeDowncast: for<'a> SelfDeref<'a> {
    fn lifetime_downcast<'a, 'b: 'a>(
        from: &'a <Self as SelfDeref<'b>>::Target,
    ) -> &'a <Self as SelfDeref<'a>>::Target;
}

impl<'a, T: ?Sized> SelfDeref<'a> for &'a T {
    type Target = Self;
}

// impl for &T

make_any_ref!(
    pub struct Reference<T:'static> = for<'a> &'a T;
    pub struct ReferenceMut<T:'static> = for<'a> &'a mut T;
);
