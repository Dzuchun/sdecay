//! Helper macros used around the crate
//!
//! NOTE: macro definitions do contain `unsafe` keyword, but there's no actual code unsafe code expanded here. Only modules allowing `unsafe` code are permitted to expand macro with it

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
                #[cfg(feature = "std")]
                #[inline]
                pub fn [<$name _shared>]$(<$l>)?(
                    $($arg: $atype,)*
                ) -> crate::container::ArcContainer<$rtype $(<$l>)?> {
                    Self::[<$name _in>]((), $($arg,)*)
                }

                $(#[$($attr)+])*
                #[doc = concat!("\n\nThis function is identical to [`", stringify!($name), "_in`](", stringify!($recv), "::", stringify!($name), "_in), but hard-coded to use [`Rc`](alloc::rc::Rc)-based container")]
                #[doc = "\n\nWARNING: `std` feature is not enabled, note the container type!"]
                #[cfg(all(feature = "alloc", not(feature = "std")))]
                #[inline]
                pub fn [<$name _shared>]$(<$l>)?(
                    & $($l)? self,
                    $($arg: $atype,)*
                ) -> crate::container::RcContainer<$rtype $(<$l>)?> {
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
                let src = src.cast_const().cast();
                // SAFETY: ffi to controlled function on C++ side
                unsafe { ::paste::paste! { sdecay_sys::sdecay::[<move_ $name>](dst, src) } };
            }
        }
    };
}
pub(crate) use impl_moveable;
