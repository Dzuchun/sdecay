//! Defines safe outer nuclide mixture types
//!
//! Unsafe: no

use core::{fmt::Debug, ops::Deref, pin::Pin};

use crate::{
    container::{Container, RefContainer},
    forward_pin_mut_call,
    wrapper::NuclideMixture,
};

/// `SandiaDecay`'s nuclide mixture
#[derive(Debug)]
pub struct GenericMixture<'l, C: Container<Inner = NuclideMixture<'l>>>(C);
/// Nuclide mixture stored in a [`alloc::boxed::Box`]
#[cfg(feature = "alloc")]
pub type Mixture<'l> = GenericMixture<'l, crate::container::BoxContainer<NuclideMixture<'l>>>;
/// Nuclide mixture stored wherever pointed [`core::mem::MaybeUninit`] is
pub type LocalMixture<'l> = GenericMixture<'l, RefContainer<'l, NuclideMixture<'l>>>;

impl<'l, C: Container<Inner = NuclideMixture<'l>>> Deref for GenericMixture<'l, C> {
    type Target = NuclideMixture<'l>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'l, C: Container<Inner = NuclideMixture<'l>>> Default for GenericMixture<'l, C>
where
    C::Allocator: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'l, C: Container<Inner = NuclideMixture<'l>>> GenericMixture<'l, C> {
    /// Allocates empty nuclide mixture
    pub fn new_in(allocator: C::Allocator) -> Self {
        Self(NuclideMixture::new(allocator))
    }

    /// Same as [`GenericMixture::new_in`], but allocator is created via [`Default::default`]
    #[inline]
    pub fn new() -> Self
    where
        C::Allocator: Default,
    {
        Self::new_in(C::Allocator::default())
    }

    #[inline]
    fn inner_mut(&mut self) -> Option<Pin<&mut NuclideMixture<'l>>> {
        self.0.try_inner()
    }
}

forward_pin_mut_call!({'l, C: Container<Inner = NuclideMixture<'l>>} GenericMixture<'l, C> :
    /// Clear all the nuclides added to the mixture
    clear() -> bool [true;false]);
