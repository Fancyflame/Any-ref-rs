//! Internal traits and their implementation for `&T` and `&mut T`.
//! It is **not** recommended to manually implement traits here and
//! there is **no need** to know this traits for practical use.
//! Please use `make_any_ref` macro if possible.

use super::make_any_ref;
use crate as any_ref;

/// Indicates that a struct is exist to return other type with any lifetime
pub trait ReturnType<'a> {
    type Target: 'a;
}

/// The target type of the struct implemented `ReturnType`
/// could "downgrade" its lifetime from higher to lower.
/// Generally, this function may not requires more complex
/// operation than return itself directly.
pub trait LifetimeDowncast: for<'a> ReturnType<'a> {
    fn lifetime_downcast<'a, 'b: 'a>(
        from: &'a <Self as ReturnType<'b>>::Target,
    ) -> &'a <Self as ReturnType<'a>>::Target;
}

/*impl<T: for<'a> ReturnType<'a> + 'static> LifetimeDowncast for T {
    fn lifetime_downcast<'a, 'b: 'a>(
        from: &'a <Self as ReturnType<'b>>::Target,
    ) -> &'a <Self as ReturnType<'a>>::Target {
        from
    }
}*/

make_any_ref!(
    pub type Reference<T:'static + ?Sized> = for<'a> &'a T;
    pub type ReferenceMut<T:'static + ?Sized> = for<'a> &'a mut T;
);
