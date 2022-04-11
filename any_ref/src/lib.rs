//! Please take a look at [owning_ref](https://crates.io/crates/owning_ref) before using this crate,
//! which is excellent and more suitable for simple usage. One of the shortages of `owning_ref` is that `owning_ref` can
//! only keeps reference and cannot keeps arbitrary struct with lifetime annotation. `any_ref` is here
//! to resolve this problem.
//!
//! Pre-made types:
//! ```no_run
//! make_any_ref!(
//!    pub type Reference<T:'static + ?Sized> = for<'a> &'a T;
//!    pub type ReferenceMut<T:'static + ?Sized> = for<'a> &'a mut T;
//! );
//! ```
//!
//! # Example
//! ```
//! use any_ref::{make_any_ref, new_any_ref, Reference};
//!
//! make_any_ref! {
//!     pub type _ReturnStr=for<'a> &'a str;
//!     pub(crate) type ReturnVec<T:'static> = for<'a> Vec<&'a T>;
//!     type _ReturnPhantomData<T:'static,const U:usize> = for<'lifetime> std::marker::PhantomData<&'lifetime (T,)>;
//! }
//!
//! let moved_ar;
//! {
//!     let num = Box::new((1, 2, 3, 4));
//!     let ar = new_any_ref::<ReturnVec<u16>, _, _>(num, |x| vec![&x.0, &x.1, &x.2, &x.3]);
//!     // We cannot use this temporarily due to a compiler bug. Use `new_any_ref` instead.
//!     // let ar = AnyRef::<ReturnVec<u16>, _>::new(num, |x| vec![&x.0, &x.1, &x.2, &x.3]);
//!     moved_ar = ar; // Move out of this scope
//! }
//! assert_eq!(moved_ar.get(), &vec![&1, &2, &3, &4]);
//!
//! let moved_ar;
//! {
//!     let s = "hello world".to_string();
//!     let ar = new_any_ref::<Reference<str>, _, _>(s, |x| &x[..5]);
//!     moved_ar = ar;
//! }
//! assert_eq!(moved_ar.get(), &"hello");
//! ```

pub mod type_substitute;
pub use any_ref_macro::make_any_ref;
use stable_deref_trait::{CloneStableDeref, StableDeref};
use std::ops::Deref;
pub use type_substitute::*;

/// The wrapper that holding `O` and the return type of `T`.
pub struct AnyRef<T: LifetimeDowncast + ?Sized, O> {
    // NOTICE: Cannot swap positions of `holder` and `owner`!
    holder: <T as ReturnType<'static>>::Target,
    owner: O,
}

impl<T, O> AnyRef<T, O>
where
    T: LifetimeDowncast + ?Sized,
    O: StableDeref + 'static,
    <O as Deref>::Target: 'static,
{
    /// Create `AnyRef`. Using this function cannot compile anyway due to a
    /// compiler bug, use `new_any_ref` instead. This
    /// function will be used as a formal way to create `AnyRef` in the future.
    /// ```no_run
    /// use any_ref::{AnyRef, Reference};
    ///
    /// let s = "hello world".to_string();
    /// let ar = AnyRef::<Reference<str>, _>::new(s, |x| &x[..5]);
    /// ```
    pub fn new<F>(owner: O, func: F) -> AnyRef<T, O>
    where
        //_T: LifetimeDowncast + ?Sized,
        F: for<'a> FnOnce(&'a <O as Deref>::Target) -> <T as ReturnType<'a>>::Target,
    {
        let tar = unsafe { &*(&*owner as *const _) };
        AnyRef {
            holder: func(tar),
            owner: owner,
        }
    }

    /// Call the given function on the target type, convert it
    /// to a new type.
    ///
    /// # Example
    /// ```
    /// use any_ref::{new_any_ref, Reference};
    ///
    /// let string="hello world".to_string();
    /// let ar = new_any_ref::<Reference<str>, _, _>(string, |s| &s[6..]);
    /// assert_eq!(*ar.get(), "world");
    ///
    /// let ar = ar.map::<Reference<[u8]>,_>(|world, string|{
    ///     assert_eq!(world, "world");
    ///     assert_eq!(string, "hello world");
    ///     (&string[..5]).as_bytes()
    /// });
    /// assert_eq!(*ar.get(), b"hello");
    /// ```
    pub fn map<T2, F>(self, func: F) -> AnyRef<T2, O>
    where
        T2: LifetimeDowncast + ?Sized,
        F: for<'a> FnOnce(
            <T as ReturnType<'a>>::Target,
            &'a <O as Deref>::Target,
        ) -> <T2 as ReturnType<'a>>::Target,
    {
        let r = unsafe { &*(&*self.owner as *const _) };
        AnyRef {
            holder: func(self.holder, r),
            owner: self.owner,
        }
    }

    /// Get the reference
    #[inline]
    pub fn get<'a>(&'a self) -> &'a <T as ReturnType<'a>>::Target {
        <T as LifetimeDowncast>::lifetime_downcast(&self.holder)
        //&self.holder
    }

    /// Return the owned struct
    #[inline]
    pub fn into_inner(self) -> O {
        self.owner
    }
}

impl<T, O> Clone for AnyRef<T, O>
where
    T: LifetimeDowncast + ?Sized,
    for<'a> <T as ReturnType<'a>>::Target: Clone,
    O: CloneStableDeref + 'static,
    //<O as Deref>::Target: 'static,
{
    #[inline]
    fn clone(&self) -> Self {
        AnyRef {
            holder: self.holder.clone(),
            owner: self.owner.clone(),
        }
    }
}

/// Create `AnyRef`. This function will be continue to use before we
/// could use `AnyRef::new`.
/// ```
/// use any_ref::{new_any_ref, Reference};
///
/// let vec = vec![1, 2, 3, 4, 5];
/// new_any_ref::<Reference<i32>, _, _>(vec, |x| &x[0]);
/// ```
pub fn new_any_ref<T, O, F>(owner: O, func: F) -> AnyRef<T, O>
where
    O: StableDeref + 'static,
    <O as Deref>::Target: 'static,
    T: LifetimeDowncast + ?Sized,
    F: for<'a> FnOnce(&'a <O as Deref>::Target) -> <T as ReturnType<'a>>::Target,
{
    let tar = unsafe { &*(&*owner as *const _) };
    AnyRef {
        holder: func(tar),
        owner: owner,
    }
}

#[test]
fn test() {
    //use crate as any_ref;
}
