//! `any_ref` is here to solve the problem that you are worry about have no idea carrying a struct
//! with lifetime annotation anywhere. It's alternative to take a look at [owning_ref](https://crates.io/crates/owning_ref).
//! One of the vital advantages of `any_ref` is that it allows you capture values with any type,
//! not limited to references. Have a wonderful time with `any_ref`!
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
//! use any_ref::{make_any_ref, AnyRef, Reference};
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
//!     let ar = AnyRef::<ReturnVec<u16>, _>::new(num, |x| vec![&x.0, &x.1, &x.2, &x.3]);
//!     moved_ar = ar; // Move out of this scope
//! }
//! assert_eq!(moved_ar.get(), &vec![&1, &2, &3, &4]);
//!
//! let moved_ar;
//! {
//!     let s = "hello world".to_string();
//!     let ar = AnyRef::<Reference<str>, _>::new(s, |x| &x[..5]);
//!     moved_ar = ar;
//! }
//! assert_eq!(*moved_ar.get(), "hello");
//! ```
//!
//! # To `build` multiple `AnyRef`s
//! ```
//! use any_ref::{make_any_ref, AnyRef, Reference};
//! use std::rc::Rc;
//!
//! let bytes: Rc<[u8]> = Rc::from((vec![1, 2, 3, 4, 5, 6]).into_boxed_slice());
//!
//! let (first_half, second_half) = any_ref::build(bytes.clone(), |array, mut builder| {
//!     let split_at = array.len() / 2;
//!     (
//!         builder.build::<Reference<[u8]>>(&array[..split_at]),
//!         builder.build::<Reference<[u8]>>(&array[split_at..])
//!     )
//! });
//!
//! assert_eq!(first_half.get(), &[1, 2, 3]);
//! assert_eq!(second_half.get(), &[4, 5, 6]);
//! ```
//!
//! # Stable Deref
//! It is worth noting that `O` must implement `std::ops::Deref` and
//! the memory address of its target must be stable even if moved.
//! In other words, the target `O` de-reference to must be allocated on the heap.
//!
//! We use [stable_deref_trait](https://crates.io/crates/stable_deref_trait)
//! crate to realize this limitation. Please read its [`StableDeref`] passage
//! for details.
//!
//! `Clone` trait is also implemented for `AnyRef`, but it requires `O` to
//! implements [`CloneStableDeref`]. That is, besides all the limitations of
//! `StableDeref`, `O` requires deref to the same address when cloned. At present,
//! only [`Rc`](std::rc::Rc) and [`Arc`](std::sync::Arc) implemented `CloneStableDeref`.

pub mod builder;
pub mod type_substitute;
#[doc(hidden)]
pub use any_ref_macro::_make_any_ref;
pub use builder::build;
use builder::AnyRefBuilder;
pub use stable_deref_trait::{CloneStableDeref, StableDeref};
use std::ops::Deref;
pub use type_substitute::*;

#[macro_export]
macro_rules! make_any_ref {
    ($($tt:tt)*) => {
        $crate::_make_any_ref!{
            $crate;$($tt)*
        }
    };
}

/// The wrapper that holding `O` and the return type of `T`.
pub struct AnyRef<T: LifetimeDowncast + ?Sized, O: 'static> {
    // NOTICE: Cannot swap positions of `holder` and `owner`!
    holder: <T as ReturnType<'static>>::Target,
    owner: O,
}

impl<T, O> AnyRef<T, O>
where
    T: LifetimeDowncast + ?Sized,
    O: StableDeref + 'static,
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
    #[inline]
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
    /// use any_ref::{AnyRef, Reference};
    ///
    /// let string="hello world".to_string();
    /// let ar: AnyRef<Reference<str>, String> = AnyRef::new(string, |s| &s[6..]);
    /// assert_eq!(*ar.get(), "world");
    ///
    /// let ar: AnyRef<Reference<[u8]>, _> = ar.map(|world, string|{
    ///     assert_eq!(world, "world");
    ///     assert_eq!(string, "hello world");
    ///     (&string[..5]).as_bytes()
    /// });
    /// assert_eq!(*ar.get(), b"hello");
    /// ```
    #[inline]
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

    /// Like `map`, but using builder.
    ///
    /// #Example
    /// ```
    /// use any_ref::{AnyRef, Reference};
    /// use std::rc::Rc;
    ///
    /// let string = Rc::new("hello world".to_string());
    /// let ar: AnyRef<Reference<str>, _> = AnyRef::new(string, |s| &s[6..]);
    /// assert_eq!(*ar.get(), "world");
    ///
    /// let (wo, bytes) = ar.map_build(|value, mut builder|{
    ///     (
    ///         builder.build::<Reference<str>>(&value[..2]),
    ///         builder.build::<Reference<[u8]>>(value.as_bytes())
    ///     )
    /// });
    ///
    /// assert_eq!(*wo.get(), "wo");
    /// assert_eq!(*bytes.get(), b"world");
    /// ```
    #[inline]
    pub fn map_build<F, R>(self, func: F) -> R
    where
        O: CloneStableDeref,
        F: for<'a> FnOnce(<T as ReturnType<'a>>::Target, AnyRefBuilder<'a, O>) -> R,
    {
        func(self.holder, AnyRefBuilder::new(self.owner))
    }

    /// Get the reference
    #[inline(always)]
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
    T: LifetimeDowncast + ?Sized,
    O: StableDeref + 'static,
    F: for<'a> FnOnce(&'a <O as Deref>::Target) -> <T as ReturnType<'a>>::Target,
{
    let tar = unsafe { &*(&*owner as *const _) };
    AnyRef {
        holder: func(tar),
        owner: owner,
    }
}
