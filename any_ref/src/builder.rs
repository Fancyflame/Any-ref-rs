use std::{marker::PhantomData, mem::MaybeUninit, ops::Deref};

use stable_deref_trait::CloneStableDeref;

use crate::{AnyRef, LifetimeDowncast, ReturnType};

/// This builder is used to build serveral any ref in one function.
pub struct AnyRefBuilder<'a, O>(O, PhantomData<&'a ()>);

impl<'a, O> AnyRefBuilder<'a, O>
where
    O: CloneStableDeref + 'static,
{
    #[inline]
    pub(super) fn new(owner: O) -> Self {
        AnyRefBuilder(owner, PhantomData)
    }

    /// Build an `AnyRef`. Even though we don't actually need this function
    /// to borrow `self` as mutable, but this could prevent short lived stuff
    /// from being captured.
    pub fn build<T>(&mut self, holder: <T as ReturnType<'a>>::Target) -> AnyRef<T, O>
    where
        T: LifetimeDowncast + ?Sized,
    {
        unsafe {
            let mut ar: MaybeUninit<AnyRef<T, O>> = MaybeUninit::uninit();
            let ptr = ar.as_mut_ptr();
            (&mut (*ptr).owner as *mut O).write(self.0.clone());
            (&mut (*ptr).holder as *mut _ as *mut () as *mut <T as ReturnType<'a>>::Target)
                .write(holder);
            ar.assume_init()
        }
    }
}

/// To create multiple `AnyRef`s through one function.
pub fn build<O, F>(owner: O, func: F)
where
    O: CloneStableDeref + 'static,
    F: for<'a> FnOnce(&'a <O as Deref>::Target, AnyRefBuilder<'a, O>),
{
    let tar = unsafe { &*(&*owner as *const _) };
    func(tar, AnyRefBuilder::new(owner));
}
