//! Defines ways to add some [`crate::wrapper::Nuclide`] amount to the [`crate::Mixture`]
//!
//! Unsafe: no

use core::{ops::Deref, pin::Pin};

use crate::wrapper::{NuclideActivityPair, NuclideMixture, NuclideNumAtomsPair};

/// Declares type's ability add a certain [`crate::wrapper::Nuclide`] amount to the [`crate::Mixture`]
pub trait AddNuclideSpec {
    #[expect(missing_docs)]
    fn add_nuclide(&self, mixture: Pin<&mut NuclideMixture<'_>>);
}

impl<S> AddNuclideSpec for S
where
    S: Deref,
    S::Target: AddNuclideSpec,
{
    fn add_nuclide(&self, mixture: Pin<&mut NuclideMixture<'_>>) {
        S::Target::add_nuclide(self, mixture);
    }
}

impl AddNuclideSpec for NuclideNumAtomsPair<'_> {
    fn add_nuclide(&self, mixture: Pin<&mut NuclideMixture<'_>>) {
        mixture.add_nuclide_num_atoms_pair(self);
    }
}

impl AddNuclideSpec for NuclideActivityPair<'_> {
    fn add_nuclide(&self, mixture: Pin<&mut NuclideMixture<'_>>) {
        mixture.add_nuclide_activity_pair(self);
    }
}
