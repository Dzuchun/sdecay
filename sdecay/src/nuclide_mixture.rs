//! Defines safe outer nuclide mixture types
//!
//! Unsafe: no

use core::{fmt::Debug, ops::Deref, pin::Pin};

use crate::{
    add_nuclide_spec::AddNuclideSpec,
    container::{Container, RefContainer},
    forward_pin_mut_call,
    wrapper::{CppException, Nuclide, NuclideMixture},
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
    /// Add a nuclide by specifying the initial activity of the nuclide
    ///
    /// Activity should be in `SandiaDecay` units
    ///
    /// ### Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::database::Database;
    /// let database = Database::from_env().unwrap();
    /// # use sdecay::nuclide;
    /// let u238 = database.nuclide(nuclide!(U-238));
    /// # use sdecay::nuclide_mixture::Mixture;
    /// let mut mixture = Mixture::new();
    /// # use sdecay::cst::curie;
    /// assert!(mixture.add_nuclide_by_activity(u238, 1e-3 * curie));
    /// # }
    /// ```
    add_nuclide_by_activity(
        nuclide: &Nuclide<'l>,
        start_activity: f64,
    ) -> bool [true;false]);
forward_pin_mut_call!({'l, C: Container<Inner = NuclideMixture<'l>>} GenericMixture<'l, C> :
    /// Add a nuclide by specifying how many nuclide atoms are initially in the mixture
    add_nuclide_by_abundance(
        nuclide: &Nuclide<'l>,
        num_init_atoms: f64,
) -> bool [true;false]);
forward_pin_mut_call!({'l, C: Container<Inner = NuclideMixture<'l>>} GenericMixture<'l, C> :
    /// Adds nuclide to the mixture
    ///
    /// Note that this function accepts [`AddNuclideSpec`], see it's doc for list of implementors
    add_nuclide(
        spec: impl AddNuclideSpec,
) -> bool [true;false]);

/// Error returned by [`GenericMixture::add_aged_nuclide_by_activity`] and [`GenericMixture::add_aged_nuclide_by_num_atoms`]
#[derive(Debug, Error)]
pub enum AgedNuclideError {
    /// Container's access to the mixture is not exclusive
    #[error("Container's access to the mixture is not exclusive")]
    NonExclusive,
    /// Exception occurred on C++ side
    ///
    /// According to C++ side, this happens in a case of long age (a little over 45 half lives of the nuclide)
    #[error(transparent)]
    Exception(CppException),
    // (YOUR AIJ TOO LONG)
}

forward_pin_mut_call!({'l, C: Container<Inner = NuclideMixture<'l>>} GenericMixture<'l, C> :
    /// NOTE: this documentation is mostly identical to the one in `SandiaDecay`'s header
    ///
    /// Add a nuclide to the mixture that is already pre-aged
    ///
    /// The activity corresponds to the parent nuclide's activity at the mixture's t=0 age. For example,
    /// - if you add 1uCi of U232 ($t_{1/2}$=68.8y) with an initial age of 20 years, and then ask for the gammas at time 68.8y, you will get the gammas of a 88.8y old sample with a current U232 activity of 0.5uCi
    /// - if you were to ask for gammas at a time of 0y, you would get the gammas of a 20 year old sample that has an activity of 1uCi
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::database::Database;
    /// let db = Database::from_env().unwrap();
    /// # use sdecay::nuclide;
    /// let u238 = db.nuclide(nuclide!(U-238));
    /// # use sdecay::nuclide_mixture::Mixture;
    /// let mut mixture = Mixture::default();
    /// # use sdecay::cst::{curie, year};
    /// mixture.add_aged_nuclide_by_activity(u238, 1.0e-3*curie, 20.0*year );
    /// # }
    /// ```
    ///
    /// Note: when this function is used, the [`NuclideMixture::nuclide_atoms`] family of functions will NOT return the number of atoms present, at the given time, for the stable nuclides
    ///
    /// ### Errors
    /// - [`AgedNuclideError::NonExclusive`] indicates container's non-exclusive access to the [`NuclideMixture`]
    /// - [`AgedNuclideError::Exception`] indicates exception on C++ side, likely caused by age being too long
    add_aged_nuclide_by_activity(
        nuclide: &Nuclide<'_>,
        activity: f64,
        age_in_seconds: f64,
) -> Result<(), AgedNuclideError> [
        match res { Ok(()) => Ok(()), Err(exception) => Err(AgedNuclideError::Exception(exception)) }, res;
        Err(AgedNuclideError::NonExclusive)
]);
forward_pin_mut_call!({'l, C: Container<Inner = NuclideMixture<'l>>} GenericMixture<'l, C> :
    /// NOTE: this documentation is mostly identical to the one in `SandiaDecay`'s header
    ///
    /// Add a nuclide to the mixture that is already pre-aged
    ///
    /// Note: when this function is used, the `numAtoms(...)` family of functions will return the number of atoms present for all descendant nuclides, including stable nuclides.
    ///
    /// ### Errors
    /// - [`AgedNuclideError::NonExclusive`] indicates container's non-exclusive access to the [`NuclideMixture`]
    /// - [`AgedNuclideError::Exception`] indicates exception on C++ side, likely caused by age being too long
    add_aged_nuclide_by_num_atoms(
        nuclide: &Nuclide<'l>,
        number_atoms: f64,
        age_in_seconds: f64,
) -> Result<(), AgedNuclideError> [
        match res { Ok(()) => Ok(()), Err(exception) => Err(AgedNuclideError::Exception(exception)) }, res;
        Err(AgedNuclideError::NonExclusive)
]);

/// Error returned by [`GenericMixture::add_nuclide_in_secular_equilibrium`]
#[derive(Debug, Error)]
pub enum AddSecularEquilibriumNuclideError {
    /// Container's access to the mixture is not exclusive
    #[error("Container's access to the mixture is not exclusive")]
    NonExclusive,
    /// Nuclide cannot obtain secular equilibrium
    #[error("Nuclide cannot obtain secular equilibrium")]
    NoSecularEquilibrium,
}

forward_pin_mut_call!({'l, C: Container<Inner = NuclideMixture<'l>>} GenericMixture<'l, C> :
    /// Add a nuclide to the mixture that has already obtained secular equilibrium
    ///
    /// The activity specified is of the parent nuclide a the mixtures t=0.
    ///
    /// ### Errors
    /// - [`AddSecularEquilibriumNuclideError::NonExclusive`] indicates container's non-exclusive access to the [`NuclideMixture`]
    /// - [`AddSecularEquilibriumNuclideError::NoSecularEquilibrium`] indicates that nuclide wan't able to obtain secular equilibrium
    add_nuclide_in_secular_equilibrium(
        parent: &Nuclide<'_>,
        parent_activity: f64,
) -> Result<(), AddSecularEquilibriumNuclideError> [
        if res { Ok(()) } else { Err(AddSecularEquilibriumNuclideError::NoSecularEquilibrium) }, res;
        Err(AddSecularEquilibriumNuclideError::NonExclusive)
]);
forward_pin_mut_call!({'l, C: Container<Inner = NuclideMixture<'l>>} GenericMixture<'l, C> :
    /// NOTE: this documentation is mostly identical to the one in `SandiaDecay`'s header
    ///
    /// Adds the children nuclides of 'parent' of whose half lives are monotonically decreasing, some examples are:
    ///
    /// |Parent|What is Added to Mixture|
    /// |:---:|:---:|
    /// |Th232|Th232, Ra228, Ac228|
    /// |U234|U234, Th230, Ra226, Rn222, Po218, At218, Rn218, Po214|
    /// |U235|U235, Th231|
    /// |U238|U238, Th234m, Pa234m|
    /// 
    /// The parent nuclide is always added in (unless its stable)
    add_nuclide_in_prompt_equilibrium(
        parent: &Nuclide<'_>,
        parent_activity: f64,
) -> bool [true;false]);
forward_pin_mut_call!({'l, C: Container<Inner = NuclideMixture<'l>>} GenericMixture<'l, C> :
    /// Clear all the nuclides added to the mixture
    clear() -> bool [true;false]);
