//! Defines ways to identify [`Element`] in the database
//!
//! Unsafe: no

use core::ffi::{CStr, c_int};

use crate::wrapper::{Element, SandiaDecayDataBase, StdString};

/// Defines a way to identify [`Element`] in the database
pub trait ElementSpec {
    #[expect(missing_docs)]
    fn get_element<'l>(&self, database: &'l SandiaDecayDataBase) -> Option<&'l Element<'l>>;
}

macro_rules! impl_as_cpp_string {
    (<$l:ident> $t:ty) => {

        impl<const $l: usize> ElementSpec for $t {
            #[inline]
            fn get_element<'l>(
                &self,
                database: &'l SandiaDecayDataBase,
            ) -> Option<&'l Element<'l>> {
                database.element_by_label(*self)
            }
        }

    };
    ($t:ty) => {
        impl_as_cpp_string!(@$t);
        #[cfg(feature = "alloc")]
        impl_as_cpp_string!(@alloc::boxed::Box<$t>);
        #[cfg(feature = "alloc")]
        impl_as_cpp_string!(@alloc::borrow::Cow<'_, $t>);
    };
    (@$t:ty) => {

        impl ElementSpec for $t {
            #[inline]
            fn get_element<'l>(
                &self,
                database: &'l SandiaDecayDataBase,
            ) -> Option<&'l Element<'l>> {
                database.element_by_label(self)
            }
        }
        impl_as_cpp_string!(@@&$t);
        impl_as_cpp_string!(@@&&$t);
    };
    (@@$t:ty) => {
        impl ElementSpec for $t {
            #[inline]
            fn get_element<'l>(
                &self,
                database: &'l SandiaDecayDataBase,
            ) -> Option<&'l Element<'l>> {
                database.element_by_label(&**self)
            }
        }
    };
}

impl_as_cpp_string!(@StdString);

impl_as_cpp_string!(CStr);
#[cfg(feature = "alloc")]
impl_as_cpp_string!(@alloc::ffi::CString);

impl_as_cpp_string!(str);
#[cfg(feature = "alloc")]
impl_as_cpp_string!(@alloc::string::String);

impl_as_cpp_string!([u8]);
#[cfg(feature = "alloc")]
impl_as_cpp_string!(@alloc::vec::Vec<u8>);

#[cfg(feature = "std")]
impl_as_cpp_string!(std::ffi::OsStr);
#[cfg(feature = "std")]
impl_as_cpp_string!(@std::ffi::OsString);

#[cfg(feature = "std")]
impl_as_cpp_string!(std::path::Path);
#[cfg(feature = "std")]
impl_as_cpp_string!(@std::path::PathBuf);

impl_as_cpp_string!(<N> [u8; N]);
impl_as_cpp_string!(<N> &[u8; N]);
impl_as_cpp_string!(<N> &&[u8; N]);

/// Numeric description of the [`Element`]
#[derive(Debug)]
pub struct ElementNum(pub c_int);

impl ElementSpec for ElementNum {
    #[inline]
    fn get_element<'l>(&self, database: &'l SandiaDecayDataBase) -> Option<&'l Element<'l>> {
        database.element_by_atomic_number(self.0)
    }
}

/// A helper macro statically converting identifier representing chemical element into element's proton count
///
/// ### Examples
/// ```rust
/// # use sdecay::{element, element_spec::ElementNum};
/// // uppercase works
/// assert_eq!(element!(H).0, 1);
/// assert_eq!(element!(Mo).0, 42);
///
/// // lowercase works too
/// assert_eq!(element!(os).0, 76);
/// assert_eq!(element!(in).0, 49); // <-- in Rust, `in` is a keyword, but this macro would still produce atomic number for Indium
/// ```
///
/// Note, that conversion is static, so non-existing name will not compile:
/// ```rust,compile_fail
/// # use sdecay::element;
/// let draconium = element!(Dr);
/// ```
#[macro_export]
macro_rules! element {
    ($e:tt) => {
        $crate::element_spec::ElementNum($crate::element_inner!($e))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! element_inner {
    (H) => {
        1
    };
    (He) => {
        2
    };
    (Li) => {
        3
    };
    (Be) => {
        4
    };
    (B) => {
        5
    };
    (C) => {
        6
    };
    (N) => {
        7
    };
    (O) => {
        8
    };
    (F) => {
        9
    };
    (Ne) => {
        10
    };
    (Na) => {
        11
    };
    (Mg) => {
        12
    };
    (Al) => {
        13
    };
    (Si) => {
        14
    };
    (P) => {
        15
    };
    (S) => {
        16
    };
    (Cl) => {
        17
    };
    (Ar) => {
        18
    };
    (K) => {
        19
    };
    (Ca) => {
        20
    };
    (Sc) => {
        21
    };
    (Ti) => {
        22
    };
    (V) => {
        23
    };
    (Cr) => {
        24
    };
    (Mn) => {
        25
    };
    (Fe) => {
        26
    };
    (Co) => {
        27
    };
    (Ni) => {
        28
    };
    (Cu) => {
        29
    };
    (Zn) => {
        30
    };
    (Ga) => {
        31
    };
    (Ge) => {
        32
    };
    (As) => {
        33
    };
    (Se) => {
        34
    };
    (Br) => {
        35
    };
    (Kr) => {
        36
    };
    (Rb) => {
        37
    };
    (Sr) => {
        38
    };
    (Y) => {
        39
    };
    (Zr) => {
        40
    };
    (Nb) => {
        41
    };
    (Mo) => {
        42
    };
    (Tc) => {
        43
    };
    (Ru) => {
        44
    };
    (Rh) => {
        45
    };
    (Pd) => {
        46
    };
    (Ag) => {
        47
    };
    (Cd) => {
        48
    };
    (In) => {
        49
    };
    (Sn) => {
        50
    };
    (Sb) => {
        51
    };
    (Te) => {
        52
    };
    (I) => {
        53
    };
    (Xe) => {
        54
    };
    (Cs) => {
        55
    };
    (Ba) => {
        56
    };
    (La) => {
        57
    };
    (Ce) => {
        58
    };
    (Pr) => {
        59
    };
    (Nd) => {
        60
    };
    (Pm) => {
        61
    };
    (Sm) => {
        62
    };
    (Eu) => {
        63
    };
    (Gd) => {
        64
    };
    (Tb) => {
        65
    };
    (Dy) => {
        66
    };
    (Ho) => {
        67
    };
    (Er) => {
        68
    };
    (Tm) => {
        69
    };
    (Yb) => {
        70
    };
    (Lu) => {
        71
    };
    (Hf) => {
        72
    };
    (Ta) => {
        73
    };
    (W) => {
        74
    };
    (Re) => {
        75
    };
    (Os) => {
        76
    };
    (Ir) => {
        77
    };
    (Pt) => {
        78
    };
    (Au) => {
        79
    };
    (Hg) => {
        80
    };
    (Tl) => {
        81
    };
    (Pb) => {
        82
    };
    (Bi) => {
        83
    };
    (Po) => {
        84
    };
    (At) => {
        85
    };
    (Rn) => {
        86
    };
    (Fr) => {
        87
    };
    (Ra) => {
        88
    };
    (Ac) => {
        89
    };
    (Th) => {
        90
    };
    (Pa) => {
        91
    };
    (U) => {
        92
    };
    (Np) => {
        93
    };
    (Pu) => {
        94
    };
    (Am) => {
        95
    };
    (Cm) => {
        96
    };
    (Bk) => {
        97
    };
    (Cf) => {
        98
    };
    (Es) => {
        99
    };
    (Fm) => {
        100
    };
    (Md) => {
        101
    };
    (No) => {
        102
    };
    (Lr) => {
        103
    };
    (Rf) => {
        104
    };
    (Db) => {
        105
    };
    (Sg) => {
        106
    };
    (Bh) => {
        107
    };
    (Hs) => {
        108
    };
    (Mt) => {
        109
    };
    (Ds) => {
        110
    };
    (Rg) => {
        111
    };
    (Cn) => {
        112
    };
    (Nh) => {
        113
    };
    (Fl) => {
        114
    };
    (Mc) => {
        115
    };
    (Lv) => {
        116
    };
    (Ts) => {
        117
    };
    (Og) => {
        118
    };

    (h) => {
        1
    };
    (he) => {
        2
    };
    (li) => {
        3
    };
    (be) => {
        4
    };
    (b) => {
        5
    };
    (c) => {
        6
    };
    (n) => {
        7
    };
    (o) => {
        8
    };
    (f) => {
        9
    };
    (ne) => {
        10
    };
    (na) => {
        11
    };
    (mg) => {
        12
    };
    (al) => {
        13
    };
    (si) => {
        14
    };
    (p) => {
        15
    };
    (s) => {
        16
    };
    (cl) => {
        17
    };
    (ar) => {
        18
    };
    (k) => {
        19
    };
    (ca) => {
        20
    };
    (sc) => {
        21
    };
    (ti) => {
        22
    };
    (v) => {
        23
    };
    (cr) => {
        24
    };
    (mn) => {
        25
    };
    (fe) => {
        26
    };
    (co) => {
        27
    };
    (ni) => {
        28
    };
    (cu) => {
        29
    };
    (zn) => {
        30
    };
    (ga) => {
        31
    };
    (ge) => {
        32
    };
    (as) => {
        33
    };
    (se) => {
        34
    };
    (br) => {
        35
    };
    (kr) => {
        36
    };
    (rb) => {
        37
    };
    (sr) => {
        38
    };
    (y) => {
        39
    };
    (zr) => {
        40
    };
    (nb) => {
        41
    };
    (mo) => {
        42
    };
    (tc) => {
        43
    };
    (ru) => {
        44
    };
    (rh) => {
        45
    };
    (pd) => {
        46
    };
    (ag) => {
        47
    };
    (cd) => {
        48
    };
    (in) => {
        49
    };
    (sn) => {
        50
    };
    (sb) => {
        51
    };
    (te) => {
        52
    };
    (i) => {
        53
    };
    (xe) => {
        54
    };
    (cs) => {
        55
    };
    (ba) => {
        56
    };
    (la) => {
        57
    };
    (ce) => {
        58
    };
    (pr) => {
        59
    };
    (nd) => {
        60
    };
    (pm) => {
        61
    };
    (sm) => {
        62
    };
    (eu) => {
        63
    };
    (gd) => {
        64
    };
    (tb) => {
        65
    };
    (dy) => {
        66
    };
    (ho) => {
        67
    };
    (er) => {
        68
    };
    (tm) => {
        69
    };
    (yb) => {
        70
    };
    (lu) => {
        71
    };
    (hf) => {
        72
    };
    (ta) => {
        73
    };
    (w) => {
        74
    };
    (re) => {
        75
    };
    (os) => {
        76
    };
    (ir) => {
        77
    };
    (pt) => {
        78
    };
    (au) => {
        79
    };
    (hg) => {
        80
    };
    (tl) => {
        81
    };
    (pb) => {
        82
    };
    (bi) => {
        83
    };
    (po) => {
        84
    };
    (at) => {
        85
    };
    (rn) => {
        86
    };
    (fr) => {
        87
    };
    (ra) => {
        88
    };
    (ac) => {
        89
    };
    (th) => {
        90
    };
    (pa) => {
        91
    };
    (u) => {
        92
    };
    (np) => {
        93
    };
    (pu) => {
        94
    };
    (am) => {
        95
    };
    (cm) => {
        96
    };
    (bk) => {
        97
    };
    (cf) => {
        98
    };
    (es) => {
        99
    };
    (fm) => {
        100
    };
    (md) => {
        101
    };
    (no) => {
        102
    };
    (lr) => {
        103
    };
    (rf) => {
        104
    };
    (db) => {
        105
    };
    (sg) => {
        106
    };
    (bh) => {
        107
    };
    (hs) => {
        108
    };
    (mt) => {
        109
    };
    (ds) => {
        110
    };
    (rg) => {
        111
    };
    (cn) => {
        112
    };
    (nh) => {
        113
    };
    (fl) => {
        114
    };
    (mc) => {
        115
    };
    (lv) => {
        116
    };
    (ts) => {
        117
    };
    (og) => {
        118
    };
}
