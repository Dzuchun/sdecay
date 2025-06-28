//! Contains definitions of Rust representations of `SandiaDecay`'s `enum`s
//!
//! ### Safety
//! This module does not contain any unsafe code
#![forbid(unsafe_code)]

macro_rules! enum_wrapper {
    ($(#[$($attr:tt)+])* enum $name:ident {
        $($(#[$($vattr:tt)+])* $variant:ident,)+
    }) => {
        ::paste::paste! {
            $(#[$($attr)+])*
            #[doc = concat!("\n This is a \"discriminator enum\" for [`", stringify!($name), "`]")]
            #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum [<$name D>] {
                $($(#[$($vattr)+])* $variant,)+
                /// Unknown, unexpected value returned from C++ side
                ///
                /// This probably indicates a build error
                Unknown,
            }

            impl crate::wrapper::Wrapper for $name {
                type CSide = sdecay_sys::sandia_decay::$name::Type;
            }

            $(#[$($attr)+])*
            #[repr(C)]
            #[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
            pub struct $name(pub(crate) sdecay_sys::sandia_decay::$name::Type);

            const _: () = const {
                use core::mem::{size_of, align_of, offset_of};
                assert!(size_of::<$name>() == size_of::<sdecay_sys::sandia_decay::$name::Type>());
                assert!(align_of::<$name>() == align_of::<sdecay_sys::sandia_decay::$name::Type>());
                assert!(offset_of!($name, 0) == 0);
            };

            impl core::fmt::Debug for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self.d() {
                        $([<$name D>]::$variant => f.write_str(concat!(stringify!($name), "::", stringify!($variant))),)+
                        _ => f.write_str("<Unknown>"),
                    }
                }
            }

            impl core::fmt::Display for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self.d() {
                        $([<$name D>]::$variant => f.write_str(stringify!($variant)),)+
                        _ => f.write_str("<Unknown>"),
                    }
                }
            }

            impl $name {
                #[doc = concat!("Obtains a desctriminator enum [`", stringify!([<$name D>]), "`], for easier matching\n\nNote, the [`", stringify!([<$name D>]), "::Unknown`], representing numeric enum value that was not known to Rust side during build")]
                pub fn d(self) -> [<$name D>] {
                    match self.0 {
                        $(sdecay_sys::sandia_decay::$name::$variant => [<$name D>]::$variant,)+
                        _ => [<$name D>]::Unknown,
                    }
                }
            }

            $(
            impl $name {
                $(#[$($vattr)+])*
                #[expect(non_upper_case_globals)]
                pub const $variant: Self = Self(sdecay_sys::sandia_decay::$name::$variant);
            }
            )+
        }
    };
}

enum_wrapper! {
    /// Possible decay modes, considered by `SandiaDecay`
    enum DecayMode {
        /// $\alpha$ particle emission
        AlphaDecay,
        /// $\beta^{-}$ particle i.e. $n -> p + e^{-} + \tilde{\nu_e}$ emission
        BetaDecay,
        /// $\beta^{+}$ particle i.e. $p -> n + e^{+} + \nu_e$ emission
        BetaPlusDecay,
        /// TODO: not sure on this one?
        ProtonDecay,
        /// Isomeric transition, i.e. decay of a nuclei without nucleon composition change
        IsometricTransitionDecay,
        /// TODO: not sure on this one?
        BetaAndNeutronDecay,
        /// TODO: not sure on this one?
        BetaAndTwoNeutronDecay,
        /// EC, i.e. nuclei absorbing an electron from it's shell
        ///
        /// $p + e^{-} -> n + \nu_e$
        ElectronCaptureDecay,
        /// TODO: not sure on this one?
        ElectronCaptureAndProtonDecay,
        /// TODO: not sure on this one?
        ElectronCaptureAndAlphaDecay,
        /// TODO: not sure on this one?
        ElectronCaptureAndTwoProtonDecay,
        /// TODO: not sure on this one?
        BetaAndAlphaDecay,
        /// TODO: not sure on this one?
        BetaPlusAndProtonDecay,
        /// TODO: not sure on this one?
        BetaPlusAndTwoProtonDecay,
        /// TODO: not sure on this one?
        BetaPlusAndThreeProtonDecay,
        /// TODO: not sure on this one?
        BetaPlusAndAlphaDecay,
        /// Two simultaneous $\beta$ decays, i.e. $2n -> 2p + 2e^{-} + 2\tilde{\nu_e}$
        DoubleBetaDecay,
        /// TODO: not sure on this one?
        DoubleElectronCaptureDecay,
        /// TODO: not sure on this one?
        Carbon14Decay,
        /// TODO: not sure on this one?
        SpontaneousFissionDecay,
        /// TODO: not sure on this one?
        ///
        /// NOTE by `SandiaDecay`: no longer used
        ClusterDecay,
        /// TODO: not sure on this one?
        DoubleProton,
        #[expect(missing_docs)]
        UndefinedDecay,
    }
}

enum_wrapper! {
    /// Particle type specifier used by [`decay_particle`](crate::wrapper::nuclide_mixture::NuclideMixture::decay_particle) and [`decay_particles_in_interval`](crate::wrapper::nuclide_mixture::NuclideMixture::decay_particles_in_interval)
    enum ProductType {
        /// $e^{-}$, i.e. electron
        BetaParticle,
        /// $\gamma$ i.e. photons above couple keV, usually originated from nuclei state transitions
        GammaParticle,
        /// $\alpha$, i.e. ${}^{4} He$ nuclei
        AlphaParticle,
        /// $e^{+}$
        PositronParticle,
        /// $\nu$ i.e. neutrino (according to comment near it's definition)
        CaptureElectronParticle,
        /// photons below couple keV, usually originated from atomic electron shell
        XrayParticle,
    }
}

enum_wrapper! {
    /// An enumeration of possible forbiddeness of decays
    ///
    /// Applies to beta, positron, and capture electron decays
    enum ForbiddennessType {
        #[expect(missing_docs)]
        NoForbiddenness,
        #[expect(missing_docs)]
        FirstForbidden,
        #[expect(missing_docs)]
        FirstUniqueForbidden,
        #[expect(missing_docs)]
        SecondForbidden,
        #[expect(missing_docs)]
        SecondUniqueForbidden,
        #[expect(missing_docs)]
        ThirdForbidden,
        #[expect(missing_docs)]
        ThirdUniqueForbidden,
        #[expect(missing_docs)]
        FourthForbidden,
    }
}

enum_wrapper! {
    /// Ordering of result vectors for calls like [`crate::wrapper::nuclide_mixture::NuclideMixture::activities`]
    enum HowToOrder {
        #[expect(missing_docs)]
        OrderByAbundance,
        #[expect(missing_docs)]
        OrderByEnergy,
    }
}
