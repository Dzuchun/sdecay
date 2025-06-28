//! Helper macros used around the crate
//!
//! NOTE: macro definitions do contain `unsafe` keyword, but there's no actual code unsafe code expanded here. Only modules allowing `unsafe` code are permitted to expand macro with it

pub(crate) use nolt::nolt;

macro_rules! generic_list {
    (!self $recv:ty as $methods:path: $cname:ident -> $(#[$($attr:tt)+])* $name:ident ($($arg:ident: $atype:ty),* $(,)?) ($($carg:expr),*$(,)?) -> $rtype:path $(| $l:lifetime)?) => {
        ::paste::paste! {
            impl $recv {
                $(#[$($attr)+])*
                // #[cfg_attr(doc, doc(hidden))]
                pub fn [< $name _in >] <$($l, )? C: crate::container::Container<Inner = $rtype $(<$l>)?>>(
                    allocator: C::Allocator,
                    $($arg: $atype,)*
                ) -> C {
                    let mut container = C::uninit(allocator);
                    let container_ptr = C::uninit_inner_ptr(&mut container);
                    // SAFETY: ffi call with
                    // - statically validated type representations
                    // - correct pointer constness (as of bindgen, that is)
                    // - pointed objects (except container one) are all live, since pointers were just created from references
                    unsafe { $methods::$cname(
                        container_ptr.cast::<<$rtype $(<$l>)? as crate::wrapper::Wrapper>::CSide>(),
                        $($carg,)*
                    ) };
                    // SAFETY: ffi call above moves a live struct into the container, initializing it
                    unsafe { C::init(container) }
                }

                $(#[$($attr)+])*
                #[doc = concat!("\n\nThis function is identical to [`", stringify!($name), "_in`](", stringify!($recv), "::", stringify!($name), "_in), but hard-coded to use [`Box`]-based container")]
                #[cfg(feature = "alloc")]
                #[inline]
                pub fn $name $(<$l>)?(
                    $($arg: $atype,)*
                ) -> crate::container::BoxContainer<$rtype $(<$l>)?> {
                    Self::[<$name _in>]((), $($arg,)*)
                }

                $(#[$($attr)+])*
                #[doc = concat!("\n\nThis function is identical to [`", stringify!($name), "_in`](", stringify!($recv), "::", stringify!($name), "_in), but hard-coded to use [`Arc`](std::sync::Arc)-based container")]
                #[doc = "\n\nNOTE: if `std` feature is not enabled, this method uses [`Rc`](alloc::rc::Rc)-based container instead!"]
                #[cfg(feature = "alloc")]
                #[inline]
                pub fn [<$name _shared>]$(<$l>)?(
                    $($arg: $atype,)*
                ) -> crate::container::ArcContainer<$rtype $(<$l>)?> {
                    Self::[<$name _in>]((), $($arg,)*)
                }

                $(#[$($attr)+])*
                #[doc = concat!("\n\nThis function is identical to [`", stringify!($name), "_in`](", stringify!($recv), "::", stringify!($name), "_in), but hard-coded to use manually-allocated container")]
                #[inline]
                pub fn [<$name _local>]<$($l,)? 'r>(
                    allocator: &'r mut ::core::mem::MaybeUninit<$rtype $(<$l>)?>,
                    $($arg: $atype,)*
                ) -> crate::container::RefContainer<'r, $rtype $(<$l>)?> {
                    Self::[<$name _in>](allocator, $($arg,)*)
                }
            }
        }
    };
}
pub(crate) use generic_list;

macro_rules! wrapper {
    ($(#[$($attr:tt)+])* $inner:path => $wrapper:ident $([$($garg:tt)+])? {
        $($(#[$($fattr:tt)+])* $v:vis $finner:ident -> $fouter:ident: $finner_ty:ty => $fouter_ty:ty,)+
        $(@pin: $pin:ident,)?
        $(@no_constr: $no_constr:ident,)?
    }) => {
        $(#[$($attr)+])*
        #[repr(C)]
        pub struct $wrapper $(<$($garg),+>)? {
            $($(#[$($fattr)+])* $v $fouter: $fouter_ty,)+
            $($pin: core::marker::PhantomPinned,)?
            $($no_constr: core::marker::PhantomData<()>,)?
            // $(_lifetime: core::marker::PhantomData<& $l ()>,)?
        }

        impl$(<$($garg),+>)? crate::wrapper::Wrapper for $wrapper $(<$($garg),+>)? {
            type CSide = $inner;
        }

        ::paste::paste!{
            #[doc(hidden)]
            #[expect(non_camel_case_types)]
            #[allow(unused)]
            type [<$wrapper _nolt>] = crate::nolt!($wrapper $(<$($garg),+>)?);
        }

        ::paste::paste!{crate::impl_moveable!{[< $wrapper:snake >], $wrapper $([$($garg)+])? }}

        // assert same size and alignment as bindgen representation
        const _:() = const {
            type SelfNolt = ::paste::paste!{[<$wrapper _nolt>]};
            use core::mem::{align_of, size_of};
            assert!(size_of::<SelfNolt>() == size_of::<$inner>());
            assert!(align_of::<SelfNolt>() == align_of::<$inner>());
        };

        // assert same size, alignment and offset of each field
        $(const _:() = const {
            type SelfNolt = ::paste::paste!{[<$wrapper _nolt>]};
            type ItemNolt = crate::nolt!($fouter_ty);
            use core::mem::{align_of, size_of, offset_of};
            assert!(size_of::<ItemNolt>() == size_of::<$finner_ty>());
            assert!(align_of::<ItemNolt>() == align_of::<$finner_ty>());
            assert!(offset_of!(SelfNolt, $fouter) == offset_of!($inner, $finner));
        };)+

        impl $(<$($garg),+>)? $wrapper $(<$($garg),+>)? {
            /// Dereferences pointer to [`Self`] reference, **even if it is null**
            ///
            /// ### Safety
            /// - `ptr` must not be nullptr
            /// - `ptr` must point to a valid live instance of [`Self`]
            #[allow(unused)]
            #[inline]
            pub(crate) unsafe fn from_ptr_unchecked<'r>(ptr: *const $inner) -> &'r Self {
                // SAFETY: `ptr` points to a valid [`Self`] (function invariant)
                unsafe { &*ptr.cast::<Self>() }
            }
            /// Dereferences pointer to [`Self`] reference, if it is not null
            ///
            /// ### Safety
            /// `ptr` must point to a valid live instance of [`Self`]
            #[allow(unused)]
            #[inline]
            pub(crate) unsafe fn from_ptr<'r>(ptr: *const $inner) -> Option<&'r Self> {
                if ptr.is_null() {
                    None
                } else {
                    // SAFETY:
                    // - `ptr` is not nullptr (checked above)
                    // - `ptr` points to a valid [`Self`] (function invariant)
                    let rf = unsafe { &*ptr.cast() };
                    Some(rf)
                }
            }

            #[allow(unused)]
            #[inline]
            pub(crate) fn ptr(&self) -> *const $inner {
                core::ptr::from_ref(self).cast()
            }

            #[allow(unused)]
            #[inline]
            pub(crate) fn ptr_mut(&mut self) -> *mut $inner {
                core::ptr::from_mut(self).cast()
            }
        }
    };
}
pub(crate) use wrapper;
macro_rules! impl_moveable {
    ($name:ident, $rtype:ident $([$($garg:tt),+])?) => {
        // SAFETY: moving is handled on C++ side, via following function:
        // ```cpp
        // template <typename T> inline void write(T *dst, T src) {
        //     new (dst) T(std::move(src));
        // }
        // ```
        // As you can (hopefully) see, this function
        // - assumes `dst` points to properly aligned, but uninitialized memory
        // - assumes `src` points to a live, valid version of the type
        // - after it's call, `dst` contains live, valid version of the type
        unsafe impl $(<$($garg),+>)? crate::container::Moveable for $rtype $(<$($garg),+>)? {
            unsafe fn mv(dst: *mut Self, src: *mut Self) {
                let dst = dst.cast();
                let src = src.cast();
                // SAFETY: ffi to controlled function on C++ side
                unsafe { ::paste::paste! { sdecay_sys::sdecay::[<move_ $name>](dst, src) } };
            }
        }
    };
}
pub(crate) use impl_moveable;
