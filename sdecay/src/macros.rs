//! Helper macros used around the crate
//!
//! NOTE: macro definitions do contain `unsafe` keyword, but there's no actual code unsafe code expanded here. Only modules allowing `unsafe` code are permitted to expand macro with it

pub(crate) use nolt::nolt;

macro_rules! containers {
    ($recv:path $([$($garg:tt),+])?: $cname:path =>
        $(#[$($attr:tt)+])* $name:ident $([ $(self: $l:lifetime $(,)?)? $($narg:tt),+])? (
            $($arg:ident: $atype:ty => $carg:expr),* $(,)?
        ) -> $rtype:path $([$($rarg:tt),+])?) => {
        ::paste::paste! {
            impl $(<$($garg),+>)? $recv $(<$($garg),+>)? {
                $(#[$($attr)+])*
                // #[cfg_attr(doc, doc(hidden))]
                pub fn [< $name _in>] <$($l ,)?$($narg, )? C: crate::container::Container<Inner = $rtype $(<$($rarg),+>)?>>(
                    & $($($l)?)? self,
                    allocator: C::Allocator,
                    $($arg: $atype,)*
                ) -> C {
                    let mut container = C::uninit(allocator);
                    let container_ptr = C::uninit_inner_ptr(&mut container);
                    let self_ptr = self.ptr();
                    // SAFETY: ffi call with
                    // - statically validated type representations
                    // - correct pointer constness (as of bindgen, that is)
                    // - pointed objects (except container one) are all live, since pointers were just created from references
                    unsafe { $cname(
                        container_ptr.cast::<<$rtype $(<$($rarg),+>)? as crate::wrapper::Wrapper>::CSide>(),
                        self_ptr.cast::<<Self as crate::wrapper::Wrapper>::CSide>(),
                        $($carg,)*
                    ) };
                    // SAFETY: ffi call above moves a live struct into the container, initializing it
                    unsafe { C::init(container) }
                }

                $(#[$($attr)+])*
                #[doc = concat!("\n\nThis function is identical to [`", stringify!($name), "_in`](", stringify!($recv), "::", stringify!($name), "_in), but hard-coded to use [`Box`]-based container")]
                #[cfg(feature = "alloc")]
                #[inline]
                pub fn $name $(<$($l ,)?$($narg,)+>)?(
                    & $($($l)?)? self,
                    $($arg: $atype,)*
                ) -> crate::container::BoxContainer<$rtype $(<$($rarg),+>)?> {
                    self.[<$name _in>]((), $($arg,)*)
                }

                $(#[$($attr)+])*
                #[doc = concat!("\n\nThis function is identical to [`", stringify!($name), "_in`](", stringify!($recv), "::", stringify!($name), "_in), but hard-coded to use [`Arc`](std::sync::Arc)-based container")]
                #[doc = "\n\nNOTE: if `std` feature is not enabled, this method uses [`Rc`](alloc::rc::Rc)-based container instead!"]
                #[cfg(feature = "alloc")]
                #[inline]
                pub fn [<$name _shared>] $(<$($l ,)?$($narg,)+>)?(
                    & $($($l)?)? self,
                    $($arg: $atype,)*
                ) -> crate::container::ArcContainer<$rtype $(<$($rarg),+>)?> {
                    self.[<$name _in>]((), $($arg,)*)
                }

                // $(#[$($attr)+])*
                // #[doc = concat!("\n\nThis function is identical to [`", stringify!($name), "_in`](", stringify!($recv), "::", stringify!($name), "_in), but hard-coded to use [`Rc`](alloc::rc::Rc)-based container")]
                // #[doc = "\n\nWARNING: `std` feature is not enabled, note the container type!"]
                // #[cfg(all(feature = "alloc", not(feature = "std")))]
                // #[inline]
                // pub fn [<$name _shared>] $(<$($l ,)?$($narg,)+>)?(
                //     & $($($l)?)? self,
                //     $($arg: $atype,)*
                // ) -> crate::container::RcContainer<$rtype $(<$($rarg),+>)?> {
                //     self.[<$name _in>]((), $($arg,)*)
                // }

                $(#[$($attr)+])*
                #[doc = concat!("\n\nThis function is identical to [`", stringify!($name), "_in`](", stringify!($recv), "::", stringify!($name), "_in), but hard-coded to use manually-allocated container")]
                #[inline]
                pub fn [<$name _local>] <'r, $($($l ,)?$($narg,)+)?>(
                    & $($($l)?)? self,
                    allocator: &'r mut core::mem::MaybeUninit<$rtype $(<$($rarg),+>)?>,
                    $($arg: $atype,)*
                ) -> crate::container::RefContainer<'r, $rtype $(<$($rarg),+>)?> {
                    self.[<$name _in>](allocator, $($arg,)*)
                }
            }
        }
    };
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

                // $(#[$($attr)+])*
                // #[doc = concat!("\n\nThis function is identical to [`", stringify!($name), "_in`](", stringify!($recv), "::", stringify!($name), "_in), but hard-coded to use [`Rc`](alloc::rc::Rc)-based container")]
                // #[doc = "\n\nWARNING: `std` feature is not enabled, note the container type!"]
                // #[cfg(all(feature = "alloc", not(feature = "std")))]
                // #[inline]
                // pub fn [<$name _shared>]$(<$l>)?(
                //     & $($l)? self,
                //     $($arg: $atype,)*
                // ) -> crate::container::RcContainer<$rtype $(<$l>)?> {
                //     Self::[<$name _in>]((), $($arg,)*)
                // }

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
pub(crate) use containers;

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

macro_rules! vec_wrapper {
    ($name:ident $([$l:lifetime])?, $ctype:ty, $rtype:ty) => {
        ::paste::paste! {
            #[doc = concat!("Rust representation of C++ `std::string<", stringify!($ctype), ">`\n\nThis type aims to be similar to `Vec<`[`", stringify!($rtype), "`]`>`, but please note that it should never be constructed or handled directly - safe handling is only possible behind the [`Container`] implementor")]
            #[repr(C)]
            pub struct [<Vec $name:camel>] $(<$l>)? {
                /// Actual `std::vector`
                inner: sdecay_sys::sdecay::[<$name _vec>],
                /// This struct should never be moved
                _pin: core::marker::PhantomPinned,
                /// This struct should never be constructed
                _private: core::marker::PhantomData<()>,
                $(
                /// Lifetime marker, restricting vector's lifetime to that of the elements
                _lifetime: core::marker::PhantomData<& $l ()>,
                )?
            }
        }

        impl $(<$l>)? crate::wrapper::Wrapper for ::paste::paste!([<Vec $name:camel>] $(<$l>)?) {
            type CSide = ::paste::paste!( sdecay_sys::sdecay::[<$name _vec>] );
        }

        // impl $(<$l>)? ::paste::paste!{[<Vec $name:camel>] $(<$l>)?} {
        //     /// Vector's element type
        //     ///
        //     /// Intended for easier documentation navigation
        //     type Item = $rtype;
        // }

        ::paste::paste!{crate::impl_moveable!{[<$name _vec>], [<Vec $name:camel>] $([$l])? }}

        const _: () = const {
            use core::mem::{size_of, align_of, offset_of};
            type SelfNolt = ::paste::paste!{crate::nolt!([<Vec $name:camel>] $(<$l>)?)};
            assert!(offset_of!(SelfNolt, inner) == 0, "Offset of vec inner");
            assert!(size_of::<SelfNolt>() == ::paste::paste!(size_of::<sdecay_sys::sdecay::[<$name _vec>]>)(), "Size of vec wrapper");
            assert!(align_of::<SelfNolt>() == ::paste::paste!(align_of::<sdecay_sys::sdecay::[<$name _vec>]>)(), "Size of vec wrapper");
            // at this point
            // - wrapper's size and alignment are equal to inner vector
            // - inner vector's offset is 0
            // meaning, we can safely cast vector wrapper to inner vector (but not the other way round, that still needs a proof of identical elements)
        };

        const _: () = const {
            type ItemNolt = crate::nolt!($rtype);
            // elements must AT LEAST have the same size and layout
            use core::mem::{size_of, align_of};
            assert!(size_of::<$ctype>() == size_of::<ItemNolt>(), "Size of vec wrapper");
            assert!(align_of::<$ctype>() == align_of::<ItemNolt>(), "Size of vec wrapper");
        };

        impl $(<$l>)? ::paste::paste!{[<Vec $name:camel>] $(<$l>)?} {
            #[inline]
            pub(crate) fn bindgen_ptr(&self) -> *const ::paste::paste!(sdecay_sys::sdecay::[<$name _vec>]) {
                core::ptr::from_ref(&self.inner)
            }

            /// ### Safety
            /// Returned pointer should **NEVER** be used to move the value out
            #[inline]
            pub(crate) unsafe fn bindgen_ptr_mut(self: core::pin::Pin<&mut Self>) -> *mut ::paste::paste!(sdecay_sys::sdecay::[<$name _vec>]) {
                // SAFETY: obtained reference will be immediately converted to a pointer
                let rf = unsafe { self.get_unchecked_mut() };
                core::ptr::from_mut(rf).cast()
            }

            #[inline]
            pub(crate) fn ptr(&self) -> *const $rtype {
                // SAFETY: ffi call forwarded to <https://cplusplus.com/reference/vector/vector/data/>
                unsafe { ::paste::paste!{ sdecay_sys::sdecay::[<std_vector_ $name:lower _ptr>](self.bindgen_ptr()) } }.cast()
            }

            #[inline]
            pub(crate) fn ptr_mut(self: core::pin::Pin<&mut Self>) -> *mut $rtype {
                // SAFETY: obtained pointer will only be used to obtain data pointer from C++ side
                let self_ptr = unsafe { self.bindgen_ptr_mut() };
                // SAFETY: ffi call forwarded to <https://cplusplus.com/reference/vector/vector/data/>
                unsafe { ::paste::paste!{sdecay_sys::sdecay::[<std_vector_ $name:lower _ptr_mut>](self_ptr)} }.cast()
            }

            #[allow(unused)]
            #[inline]
            pub(crate) fn from_ptr(inner: *const ::paste::paste!(sdecay_sys::sdecay::[<$name _vec>])) -> *const Self {
                inner.cast()
            }

            /// Creates empty vector stored in container `C` allocated via provided allocator
            #[inline]
            pub fn new_in<C:crate::container::Container<Inner = Self>>(allocator: C::Allocator) -> C {
                let mut uninit = C::uninit(allocator);
                let ptr = C::uninit_inner_ptr(&mut uninit).cast();
                // SAFETY: ffi call forwarded to `std::vector`'s default constructor
                unsafe { ::paste::paste!{sdecay_sys::sdecay::[<std_vector_ $name:lower _new>](ptr)} };
                // SAFETY: `std::vector`'s constructor initialized contained value
                unsafe { C::init(uninit) }
            }

            /// Same as [`Self::new_in`], but uses `C::Allocator`'s default implementation
            #[inline]
            #[expect(clippy::new_ret_no_self)]
            pub fn new<C:crate::container::Container<Inner = Self>>() -> C
            where
                C::Allocator: Default
            {
                Self::new_in(C::Allocator::default())
            }

            /// Forwarded to <https://cplusplus.com/reference/vector/vector/reserve/>
            #[inline]
            pub fn reserve(self: core::pin::Pin<&mut Self>, capacity: usize) {
                // SAFETY: obtained pointer will only be used to reverse more memory in `std::vector` buffer
                let self_ptr = unsafe { self.bindgen_ptr_mut() }.cast();
                // SAFETY: ffi call forwarded to <https://cplusplus.com/reference/vector/vector/reserve/>
                unsafe { ::paste::paste!{sdecay_sys::sdecay::[<std_vector_ $name:lower _reserve>](self_ptr, capacity)} }
            }

            /// Same as consequent [`Self::new_in`] and [`Self::reserve`] calls
            #[inline]
            pub fn new_reserve_in<C:crate::container::Container<Inner = Self>>(allocator: C::Allocator, capacity: usize) -> C {
                let mut new = Self::new_in::<C>(allocator);
                let r = new.try_inner().expect("Container was just created and should not be shared yet");
                r.reserve(capacity);
                new
            }

            /// Same as consequent [`Self::new_reserve_in`], but uses `C::Allocator`'s default implementation to obtain the allocator
            #[inline]
            pub fn new_reserve<C:crate::container::Container<Inner = Self>>(capacity: usize) -> C
            where
                C::Allocator: Default
            {
                Self::new_reserve_in(C::Allocator::default(), capacity)
            }

            /// Push `item` to the vector
            ///
            /// Note, that this is only possible if **element can be created**, namely, [`Self`] can never be the `item` here
            pub fn push(self: core::pin::Pin<&mut Self>, item: $rtype) {
                // SAFETY: obtained pointer will only be used to push item to the std::vector
                let self_ptr = unsafe { self.bindgen_ptr_mut() }.cast();
                let mut item = core::mem::MaybeUninit::new(item);
                let item_ptr: *mut $ctype = item.as_mut_ptr().cast::<$ctype>();
                // SAFETY: ffi call forwarded to <https://cplusplus.com/reference/vector/vector/push_back/>
                unsafe { ::paste::paste!{sdecay_sys::sdecay::[<std_vector_ $name:lower _push>](self_ptr, item_ptr)} };
            }


            /// Returns `std::vector`'s length (element count)
            #[inline]
            pub fn len(&self) -> usize {
                let self_ptr = self.bindgen_ptr().cast();
                // SAFETY: ffi call forwarded to <https://cplusplus.com/reference/vector/vector/size/>
                unsafe { ::paste::paste!{sdecay_sys::sdecay::[<std_vector_ $name:lower _size>](self_ptr)} }
            }

            /// Checks is `std::vector` is empty
            ///
            /// (yes, it does forward to C side, that's just how I feel)
            #[inline]
            pub fn is_empty(&self) -> bool {
                let self_ptr = self.bindgen_ptr().cast();
                // SAFETY: ffi call forwarded to <https://cplusplus.com/reference/vector/vector/empty/>
                unsafe{ ::paste::paste!{sdecay_sys::sdecay::[<std_vector_ $name:lower _empty>](self_ptr)} }
            }

            #[doc = concat!("Returns contained elements as `&[`[`", stringify!($rtype), "`]")]
            #[inline]
            pub fn as_slice(&self) -> &[$rtype] {
                let len = self.len();
                if len == 0 {
                    return &[];
                }
                let ptr = self.ptr();
                // SAFETY:
                // - data pointer points to first of vector's items
                // - slice length is a vector item count
                unsafe { core::slice::from_raw_parts(ptr, len) }
            }

            #[doc = concat!("Returns contained elements as `&mut [`[`", stringify!($rtype), "`]")]
            #[inline]
            pub fn as_mut_slice(self: core::pin::Pin<&mut Self>) -> &mut [$rtype] {
                let len = self.len();
                if len == 0 {
                    return &mut [];
                }
                let ptr = self.ptr_mut();
                // SAFETY:
                // - data pointer points to first of vector's items
                // - slice length is a vector item count
                unsafe { core::slice::from_raw_parts_mut(ptr, len) }
            }
        }

        impl $(<$l>)? core::fmt::Debug for ::paste::paste!{[<Vec $name:camel>] $(<$l>)?} {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Debug::fmt(self.as_ref(), f)
            }
        }


        impl $(<$l>)? AsRef<[$rtype]> for ::paste::paste!{[<Vec $name:camel>] $(<$l>)?} {
            #[inline]
            fn as_ref(&self) -> &[$rtype] {
                self.as_slice()
            }
        }

        impl<$($l, )? Idx> core::ops::Index<Idx> for ::paste::paste!{[<Vec $name:camel>] $(<$l>)?}
            where [$rtype]: core::ops::Index<Idx>
        {
            type Output = <[$rtype] as core::ops::Index<Idx>>::Output;

            #[inline]
            fn index(&self, index: Idx) -> &Self::Output {
                self.as_slice().index(index)
            }
        }

        impl<$($l, )? 'r> IntoIterator for &'r ::paste::paste!{[<Vec $name:camel>] $(<$l>)?} {
            type Item = <&'r [$rtype] as IntoIterator>::Item;
            type IntoIter = <&'r [$rtype] as IntoIterator>::IntoIter;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                self.as_slice().into_iter()
            }
        }

        impl $(<$l>)? core::ops::Deref for ::paste::paste!{[<Vec $name:camel>] $(<$l>)?} {
            type Target = [$rtype];

            #[inline]
            fn deref(&self) -> &[$rtype] {
                self.as_slice()
            }
        }

        impl $(<$l>)? Drop for ::paste::paste!{[<Vec $name:camel>] $(<$l>)?} {
            #[inline]
            fn drop(&mut self) {
                let self_ptr = core::ptr::from_mut(self).cast();
                // SAFETY: ffi call forwarded to `std::vector` destructor
                unsafe { ::paste::paste!{ sdecay_sys::sdecay::[<std_vector_ $name:lower _destruct>](self_ptr) } }
            }
        }
    };
}
pub(crate) use vec_wrapper;

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

macro_rules! forward_pin_mut_call {
    ($({$($gargs:tt)+})? $this:ty : $(#[$($attr:tt)+])* $name:ident $(<$($fargs:tt)+>)? (
        $($arg:ident: $argt:ty),*$(,)?
    ) -> $ret:ty [$($ok_expr:expr $(, $res:ident)?)?; $($err_expr:expr)?]) => {
        impl $(<$($gargs)+>)? $this {
            $(#[$($attr)+])*
            #[inline]
            pub fn $name $(<$($fargs)+>)? (&mut self, $($arg: $argt),*) -> $ret {
                if let Some(pin) = self.inner_mut() {
                    $($(let $res = )?)? pin.$name($($arg),*);
                    $($ok_expr)?
                } else {
                    $($err_expr)?
                }
            }
        }
    };
}
pub(crate) use forward_pin_mut_call;

macro_rules! ffi_unwrap_or {
    ($cname:path => $name:ident ( $($arg:ident: $argt:ty),*$(,)? ) -> $rtype:ident $(<$l:lifetime>)? ?? $out:ident -> $default_expr:block) => {
        #[doc = concat!("### Safety\n- `out` must point to properly allocated but uninitialized memory (will be overwritten, with no drop logic)\n- rest of the arguments must adhere to ", stringify!($cname),"'s invariants")]
        unsafe fn $name (
            out: *mut <nolt::nolt!($rtype $(<$l>)?) as crate::wrapper::Wrapper>::CSide,
            $($arg: <nolt::nolt!($argt) as crate::wrapper::Wrapper>::CSide,)*
        ) {
            let mut error = MaybeUninit::<CppException>::uninit();
            let error_ptr = error.as_mut_ptr().cast::<sdecay_sys::sdecay::Exception>();
            // SAFETY: ffi call with
            // - statically validated type representations
            // - correct pointer constness (as of bindgen, that is)
            // - `out` points to uninitialized memory (function invariant)
            // - `error_ptr` points to uninitialized memory by construction
            let tag = unsafe {
                $cname(
                    out,
                    error_ptr,
                    $($arg,)*
                )
            };
            if tag {
                // already written output
            } else {
                // drop the error
                // SAFETY: `tag == false` guarantees that exception occurred and written to `exception`
                unsafe { error.assume_init_drop() };
                // initialize default
                let $out = out;
                $default_expr;
            }
        }
    };
}
pub(crate) use ffi_unwrap_or;
