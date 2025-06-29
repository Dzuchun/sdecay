use core::{
    ffi::{c_double, c_float, c_short},
    fmt::{Debug, Display},
};

use sdecay_sys::sdecay::transition_ptr_vec;

use crate::wrapper::{
    BindgenString, StdString, VecNuclideActivityPair, VecNuclideRef, VecNuclideTimeEvolution,
    VecTransitionPtr, containers, wrapper,
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
