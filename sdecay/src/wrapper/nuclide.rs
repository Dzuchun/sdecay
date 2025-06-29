use core::{
    ffi::{c_double, c_float, c_short},
    fmt::{Debug, Display},
};

use sdecay_sys::sdecay::transition_ptr_vec;

use crate::{
    containers, wrapper,
    wrapper::{
        BindgenString, StdString, VecNuclideActivityPair, VecNuclideRef, VecNuclideTimeEvolution,
        VecTransitionPtr,
    },
};

wrapper! {
    /// NOTE: this documentation is mostly identical to the one in `SandiaDecay`'s header
    ///
    /// Struct to store information about a nuclide
    sdecay_sys::sandia_decay::Nuclide => Nuclide['l] {
        /// The normalized ascii string symbol for this nuclide
        ///
        /// Examples: `U238`, `Pu237m`, `Co60`, `Au192m2`
        pub symbol -> symbol: BindgenString => StdString,
        /// The atomic number for this nuclide i.e. number of protons in the nucleus
        pub atomicNumber -> atomic_number: c_short => c_short,
        /// The atomic number for this nuclide i.e. number of nucleons in the nucleus
        pub massNumber -> mass_number: c_short => c_short,
        /// Nuclear excitation state (isomer number)
        pub isomerNumber -> isomer_number: c_short => c_short,
        /// Atomic mass in a.m.u.
        ///
        /// Example: ${}^{12}C$ is 12.0 a.m.u.
        pub atomicMass -> atomic_mass: c_float => f32,
        /// Nuclide half-life in units of [`crate::cst`]
        pub halfLife -> half_life: c_double => f64,
        /// The nuclear transitions this nuclide decays through
        pub decaysToChildren -> decays_to_children: transition_ptr_vec => VecTransitionPtr<'l>,
        /// The nuclear transitions that this nuclide can be the result of
        pub decaysFromParents -> decay_from_parents: transition_ptr_vec => VecTransitionPtr<'l>,
        @pin: _pin,
        @no_constr: _no_constr,
    }
}

impl Debug for Nuclide<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Nuclide")
            .field("symbol", &self.symbol)
            .field("atomic_number", &self.atomic_number)
            .field("mass_number", &self.mass_number)
            .field("isomer_number", &self.isomer_number)
            .field("atomic_mass", &self.atomic_mass)
            .field("half_life", &self.half_life)
            .finish_non_exhaustive()
    }
}

impl Display for Nuclide<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.symbol, f)
    }
}

impl PartialEq for Nuclide<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
            && self.atomic_number == other.atomic_number
            && self.mass_number == other.mass_number
            && self.isomer_number == other.isomer_number
            && self.atomic_mass == other.atomic_mass
            && self.half_life == other.half_life
    }
}

impl Nuclide<'_> {
    /// Checks if [`Nuclide`] has a finite half-life
    ///
    /// Infinite half-life implies a stable nuclide
    #[inline]
    pub fn is_stable(&self) -> bool {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_isStable(self_ptr) }
    }

    /// The fraction of decays of this nuclide that proceeds through the specified descendant (which may be multiple generations down the chain)
    pub fn branching_ratio_to_descendant(&self, descendant: &Nuclide<'_>) -> f32 {
        let self_ptr = self.ptr();
        let descendant_ptr = descendant.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe {
            sdecay_sys::sandia_decay::Nuclide_branchRatioToDecendant(self_ptr, descendant_ptr)
        }
    }

    /// The fraction of decays of the specified ancestor (which can be multiple generations above) that proceeds through this nuclide
    pub fn branching_ratio_from_forebear(&self, ancestor: &Nuclide<'_>) -> f32 {
        let self_ptr = self.ptr();
        let ancestor_ptr = ancestor.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_branchRatioFromForebear(self_ptr, ancestor_ptr) }
    }

    /// The decay constant $\lambda$ that is defined as
    /// $$
    /// 0.5 = \exp( - \text{decay_const} \cdot \text{half_life} )
    /// $$
    /// or put another way $\lambda = \ln(2)/\text{half_life}$.
    #[expect(clippy::doc_markdown)]
    #[inline]
    pub fn decay_constant(&self) -> f64 {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_decayConstant(self_ptr) }
    }

    /// Maximum half-life of all nuclide descendants; dictates time-scale for secular equilibrium. If this value exceeds or equals nuclide half-life, secular equilibrium cannot be achieved, however this function will still return this value.
    #[inline]
    pub fn secular_equilibrium_half_life(&self) -> f64 {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_secularEquilibriumHalfLife(self_ptr) }
    }

    /// Returns true if all decendants have a shorter half life than this nuclide
    #[inline]
    pub fn can_obtain_secular_equilibrium(&self) -> bool {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_canObtainSecularEquilibrium(self_ptr) }
    }

    /// Prompt equilibrium half life is maximum half-life of nuclide descendants in decay series with monotonically decreasing half-lives; dictates scale for prompt equilibrium.  If prompt equilibrium half-life exceeds or equals nuclide half-life, prompt equilibrium cannot be achieved so zero is returned. For instance, an analysis application may use this to define photopeaks for `U`, `Pu`, `Th`, etc, by aging a Nuclide by $\log_2(1000)\cdot\text{promptEquilibriumHalfLife}$
    ///
    /// If a `promptEquilibriumHalfLife` doesn't exist, then the secular equilibrium is used, and if this doesn't exist, only for the isotope
    #[expect(clippy::doc_markdown)]
    #[inline]
    pub fn prompt_equilibrium_half_life(&self) -> f64 {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_promptEquilibriumHalfLife(self_ptr) }
    }

    /// Returns true if the prompt equilibrium half life is defined
    #[inline]
    pub fn can_obtain_prompt_equilibrium(&self) -> bool {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_canObtainPromptEquilibrium(self_ptr) }
    }

    /// The number of atoms of this nuclide it would take to equal 1 gram
    #[inline]
    pub fn atoms_per_gram(&self) -> f64 {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_atomsPerGram(self_ptr) }
    }

    /// The activity (in `SandiaDecay` units) of one gram of this nuclide
    #[inline]
    pub fn activity_per_gram(&self) -> f64 {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_activityPerGram(self_ptr) }
    }

    /// Determine if all child isotopes are stable (e.g. aging doesn't effect gamma energies emitted)
    #[inline]
    pub fn decays_to_stable_children(&self) -> bool {
        let self_ptr = self.ptr();
        // SAFETY: ffi call with
        // - statically validated type representations
        // - correct pointer constness (as of bindgen, that is)
        // - `self_ptr` points to a live object, since it was just created from the reference
        unsafe { sdecay_sys::sandia_decay::Nuclide_decaysToStableChildren(self_ptr) }
    }
}

containers! { Nuclide['l]: sdecay_sys::sdecay::nuclide::descendants =>
    /// Returns all progeny (descendant) isotopes (child, grand-child, etc.) and not just immediate child nuclides
    ///
    /// Results will also include this nuclide.
    ///
    /// Results are sorted roughly according to decay chain, but this ordering may not be unique
    descendants() -> VecNuclideRef['l]
}
containers! { Nuclide['l]: sdecay_sys::sdecay::nuclide::forebearers =>
    /// Returns all isotopes where this nuclide will be in their decay chain (e.g., parent, grandparent, great-grandparent, etc)
    ///
    /// Results will also include this nuclide.
    forebearers() -> VecNuclideRef['l]
}
containers! { Nuclide['l]: sdecay_sys::sdecay::nuclide::human_str_summary =>
    /// A human readable summary
    human_str_summary() -> StdString
}
containers! { Nuclide['l]: sdecay_sys::sdecay::database::decay_single =>
    /// Decays a single nuclide of specified activity, and returns a list of descendant nuclide activities at the certain time
    decay(
        original_activity: f64 => original_activity,
        time_in_seconds: f64 => time_in_seconds
    ) -> VecNuclideActivityPair['l]
}
containers! { Nuclide['l]: sdecay_sys::sdecay::database::evolution_single =>
    /// Finds evolution of this and descendant nuclides at specified original activity
    evolution(
        original_activity: f64 => original_activity,
    ) -> VecNuclideTimeEvolution['l]
}
