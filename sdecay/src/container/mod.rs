//! Handling C++ types requires caution, because of possibly-defined move constructor. Since Rust types are always allowed to be moved, unless it's an `Unpin` type behind pinned pointer, representations of C++ types must contain `PhantomPinned` field and handled indirectly - behind a pinned smart pointer.
//!
//! This module defines [`Container`] trait, as well as multiple it's implementations, allowing us to handle C++ types without causing trouble.
//!
//! Unsafe: **YES**

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc};
use core::{mem::MaybeUninit, ops::Deref, pin::Pin};
#[cfg(feature = "std")]
use std::sync::Arc;

/// Defines move constructor for the type
///
/// ### Copy types
///
/// All copy types have a trivial implementation via [`core::ptr::copy`]
///
/// Copy types are intended to have trivial move semantics, so in case your type DOES have special move constructor, make sure to make it `!Copy` - I'd probably want that anyway
///
/// ### Safety
/// [`Moveable::mv`] function MUST respect following specification:
/// - at function call, `dst` points to properly aligned, but initialized memory
/// - at function call, `src` points to a live, valid version of the type
/// - after function call, `dst` must contain live, valid version of the type
pub unsafe trait Moveable {
    /// Moves value from `src` to `dst`
    ///
    /// ### Safety
    /// - both pointers must be aligned
    /// - both pointers must be valid for reads and writes
    /// - `src` must point to live, valid version of the type
    unsafe fn mv(dst: *mut Self, src: *mut Self);
}

// SAFETY: Copy types have trivial move semantics
unsafe impl<T: Copy> Moveable for T {
    unsafe fn mv(dst: *mut Self, src: *mut Self) {
        // SAFETY:
        // - src is valid for reads of T
        // - dst is valid for writes of T
        // - by definition, Copy types can be copied around as a bunch of bytes
        unsafe { core::ptr::copy(src, dst, 1) };
    }
}

/// Represents container that can be used to safely handle types with non-trivial move semantics
///
/// ### Examples
/// Container types usually end in "Container": [`BoxContainer`], [`RcContainer`], [`ArcContainer`], [`RefContainer`], etc. You may implement your own, and most methods would accept it just fine.
///
/// Typically, container lifecycle involves:
///
/// 1. Creation of uninit container:
/// ```rust
/// # use sdecay::container::Container;
/// struct S(i32);
///
/// fn use_container<C: Container<Inner = S>>(allocator: C::Allocator) {
///     let mut uninit = C::uninit(allocator);
///     // (...)
/// }
/// ```
/// 2. Container initialization:
/// ```rust
/// # use sdecay::container::Container;
/// # struct S(i32);
/// # fn use_container<C: Container<Inner = S>>(allocator: C::Allocator) {
/// #    let mut uninit = C::uninit(allocator);
///     // get pointer
///     let ptr = C::uninit_inner_ptr(&mut uninit);
///     // initialize (for example, call a C++ constructor)
///     unsafe { core::ptr::write(ptr, S(42)) };
///     let container = unsafe { C::init(uninit) };
/// # }
/// ```
/// Alternatively, with single unsafe block:
/// ```rust
/// # use sdecay::container::Container;
/// # struct S(i32);
/// # fn use_container<C: Container<Inner = S>>(allocator: C::Allocator) {
///     let container = unsafe {
///         C::init_ptr(allocator, |ptr| unsafe { core::ptr::write(ptr, S(42)) })
///     };
/// # }
/// ```
/// 3. Container use:
/// ```rust
///# struct S(i32);
/// impl S {
///     fn use_ref(&self) { /* use behind shared reference */ }
///     fn use_mut(self: Pin<&mut Self>) { /* use behind exclusive reference */ }
/// }
/// // fn
/// # use core::pin::Pin;
/// # use sdecay::container::Container;
/// # fn use_container<C: Container<Inner = S>>(allocator: C::Allocator) {
/// #    let mut uninit = C::uninit(allocator);
/// #    // get pointer
/// #    let ptr = C::uninit_inner_ptr(&mut uninit);
/// #    // initialize (for example, call a C++ constructor)
/// #    unsafe { core::ptr::write(ptr, S(42)) };
/// #    let mut container = unsafe { C::init(uninit) };
///     container.use_ref(); // &T can be used right away, since `Container` requires `Deref`
///
///     // while getting exclusive reference, Option is returned, None indicating non-exclusive access to data
///     container.try_inner().unwrap().use_mut();
///     // note function signature - exclusive reference is always pinned
/// # }
/// ```
/// If you would want your container to always have unique access to data, use [`ExclusiveContainer`]:
/// ```rust
/// # use core::pin::Pin;
/// # use sdecay::container::ExclusiveContainer;
/// # struct S(i32);
/// # impl S {
/// #    fn use_ref(&self) { /* use behind shared reference */ }
/// #
/// #    fn use_mut(self: Pin<&mut Self>) { /* use behind exclusive reference */ }
/// # }
///fn use_container<C: ExclusiveContainer<Inner = S>>(allocator: C::Allocator) {
/// #    let mut uninit = C::uninit(allocator);
/// #    // get pointer
/// #    let ptr = C::uninit_inner_ptr(&mut uninit);
/// #    // initialize (for example, call a C++ constructor)
/// #    unsafe { core::ptr::write(ptr, S(42)) };
/// #    let mut container = unsafe { C::init(uninit) };
///     // (usual initialization)
///     container.inner().use_mut(); // no unwrap required, no panic possible
/// }
/// ```
/// 4. Drop or move out:
/// - drop
/// ```rust
/// # use sdecay::container::Container;
/// # struct S(i32);
/// # fn use_container<C: Container<Inner = S>>(allocator: C::Allocator) {
/// #    let mut uninit = C::uninit(allocator);
/// #    // get pointer
/// #    let ptr = C::uninit_inner_ptr(&mut uninit);
/// #    // initialize (for example, call a C++ constructor)
/// #    unsafe { core::ptr::write(ptr, S(42)) };
/// #    let mut container = unsafe { C::init(uninit) };
///     core::mem::drop(container); // (containers are intended to have drop impl)
/// # }
/// ```
/// - move out
/// ```rust
/// # use sdecay::container::Container;
/// # #[derive(Clone, Copy)]
/// # struct S(i32);
/// # fn use_container<C: Container<Inner = S> + core::fmt::Debug>(allocator: C::Allocator) {
/// #   let mut uninit = C::uninit(allocator);
/// #   // get pointer
/// #   let ptr = C::uninit_inner_ptr(&mut uninit);
/// #   // initialize (for example, call a C++ constructor)
/// #   unsafe { core::ptr::write(ptr, S(42)) };
/// #   let mut container = unsafe { C::init(uninit) };
///     let i42 = container.try_move_out(|ptr| unsafe { ptr.read() }).unwrap();
/// #   assert_eq!(i42.0, 42);
/// # }
/// ```
/// - move to different container
/// ```rust
/// # use core::mem::MaybeUninit;
/// # use sdecay::container::{Container, ExclusiveContainer, RefContainer};
/// # #[derive(Clone, Copy)]
/// # struct S(i32);
/// # fn use_container<C: Container<Inner = S> + core::fmt::Debug>(allocator: C::Allocator) {
/// #    let mut uninit = C::uninit(allocator);
/// #    // get pointer
/// #    let ptr = C::uninit_inner_ptr(&mut uninit);
/// #    // initialize (for example, call a C++ constructor)
/// #    unsafe { core::ptr::write(ptr, S(42)) };
/// #    let mut container = unsafe { C::init(uninit) };
///     // while moving, `Result<NewContainer, OldContainer>` is returned, Err indicating non-exclusive access
///     let mut tmp = MaybeUninit::uninit();
///     let new_container = container.try_mv::<RefContainer<'_, _>>(&mut tmp).unwrap();
/// #    let i42 = new_container.move_out(|ptr| unsafe { ptr.read() });
/// #    assert_eq!(i42.0, 42);
/// # }
/// ```
/// Alternatively, use [`ExclusiveContainer`]:
/// ```rust
/// # use core::mem::MaybeUninit;
/// # use sdecay::container::{Container, ExclusiveContainer, RefContainer};
/// # #[derive(Clone, Copy)]
/// # struct S(i32);
/// fn use_container<C: ExclusiveContainer<Inner = S>>(allocator: C::Allocator) {
/// #   let mut uninit = C::uninit(allocator);
/// #   // get pointer
/// #   let ptr = C::uninit_inner_ptr(&mut uninit);
/// #   // initialize (for example, call a C++ constructor)
/// #   unsafe { core::ptr::write(ptr, S(42)) };
/// #   let mut container = unsafe { C::init(uninit) };
///     // (usual initialization)
///     let mut tmp = MaybeUninit::uninit();
///     let new_container = container.mv::<RefContainer<'_, _>>(&mut tmp); // NOTE: no unwrap
/// #   let i42 = new_container.move_out(|ptr| unsafe { ptr.read() });
/// #   assert_eq!(i42.0, 42);
/// }
/// ```
/// ### Safety
/// - [`Container::Uninit`] *must not* allow any form of shared ownership. If the container itself or inner type do implement shared ownership, they must be sufficiently encapsulated to not allow it until [`Container::init`] call
/// - Moving this type should NOT move referenced [`Container::Inner`]
/// - Dropping this type is expected to drop [`Container::Inner`] (i.e. all resources should be freed appropriately by just dropping the container, no extra calls required)
pub unsafe trait Container:
    Deref<Target = Self::Inner> + AsRef<Self::Inner> + Sized
{
    /// Type used to create uninitialized version of the container
    ///
    /// Usually this is a `()`; at the moment, [`RefContainer`] is the only container having a different type -- `&mut [MaybeUninit]`
    type Allocator;
    /// Contained type
    type Inner;
    /// Container in the uninitialized state. Usually somewhat similar to container itself
    ///
    /// Notably, these types should not be
    type Uninit;

    /// Creates container in the uninitialized state
    ///
    /// ### Example
    /// Note, that argument type is defined by the container. Here are some examples:
    ///
    /// - box container:
    /// ```rust
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::container::{BoxContainer, Container};
    /// BoxContainer::<i32>::uninit(());
    /// # }
    /// ```
    /// - ref container:
    /// ```rust
    /// # use core::mem::MaybeUninit;
    /// # use sdecay::container::{RefContainer, Container};
    /// let mut tmp = MaybeUninit::uninit();
    /// RefContainer::<'_, i32>::uninit(&mut tmp);
    /// ```
    fn uninit(allocator: Self::Allocator) -> Self::Uninit;

    /// Retrieves pointer to the contents of uninit container
    fn uninit_inner_ptr(uninit: &mut Self::Uninit) -> *mut Self::Inner;

    /// ### Safety
    /// Memory managed by the pointer must contain a valid [`Container::Inner`]. To initialize the value, use pointer returned by [`Container::uninit_inner_ptr`]
    unsafe fn init(uninit: Self::Uninit) -> Self;

    /// Gets exclusive reference to inner value. Note that exclusive reference is always pinned
    ///
    /// ### Returns
    /// [`Option::None`] variant indicates non-exclusive memory access, for example, function was called on [`ArcContainer`] having other [`ArcContainer`] (s) pointing to the same value
    fn try_inner(&mut self) -> Option<Pin<&mut Self::Inner>>;

    /// Moves value out of the container. `action` is called *at most once* (i.e. 1 or 0 times) and provided with a pointer to *contained* value (i.e. it is valid for reads and points to valid [`Container::Inner`])
    ///
    /// ### UB warning
    /// *NOTE*: you may try doing this manually, like this:
    /// ```rust,ignore
    /// let container: impl Container<Inner = T> = /* */;
    /// let ptr: *const T = core::ptr::from_ref(&*container).cast_mut();
    /// // move value out of `ptr`
    /// ```
    ///
    /// This implementation is flawed, since [`Container`] will drop contained value again at it's drop point
    ///
    /// However, even adding
    /// ```rust,ignore
    /// core::mem::forget(container);
    /// ```
    /// Will still most likely be a UB, since it assumes valid container as argument of [`core::mem::forget`], which would likely imply valid `T` still contained inside
    ///
    /// This by itself IS safe, since it's call can only ever **leak** the resources, for example, like this:
    /// ```rust,ignore
    /// let container: impl Container<Inner = T> = /* */;
    /// container.try_move_out(|ptr: *mut T| {
    ///     // w-what is this scary `*mut T`-thing?
    ///     // I'm scared of it, I guess let's just ignore it
    ///     let _ = ptr;
    /// }).expect("Should have exclusive memory access to leak resources");
    /// // resources managed by `T` were leaked at this point
    /// ```
    ///
    /// Considering examples above, please take care, while implementing this function.
    fn try_move_out<O>(self, action: impl FnOnce(*mut Self::Inner) -> O) -> Result<O, Self>;

    /// Provided helper function creating and initializing the container in a single call
    ///
    /// ### Example
    ///
    /// ```rust
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::container::{BoxContainer, Container};
    /// let container = unsafe { BoxContainer::init_ptr((), |ptr: *mut i32| unsafe { core::ptr::write(ptr, 42) }) };
    /// # }
    /// ```
    ///
    /// ### Safety
    ///
    /// `initializer` must *actually initialize* the value behind a pointer. It is a UB to not do that:
    /// ```rust,no_run
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::container::{Container, BoxContainer};
    /// # type T = Box<()>;
    /// let container = unsafe { BoxContainer::<T>::init_ptr((), |_ptr: *mut T| {}) };
    /// // `container` assumes T is init, while it is not, technically this is a UB
    /// # }
    /// ```
    #[inline]
    unsafe fn init_ptr(
        allocator: Self::Allocator,
        initializer: impl FnOnce(*mut Self::Inner),
    ) -> Self {
        let mut uninit = Self::uninit(allocator);
        let ptr = Self::uninit_inner_ptr(&mut uninit);
        initializer(ptr);
        // SAFETY: `initializer` should have initialized contained data. It is a UB to not do so, that is why this function is `unsafe`
        unsafe { Self::init(uninit) }
    }

    /// Tries moving value out of the container into a different container
    ///
    /// ### Returns
    /// - [`Result::Ok`] variant indicates successful move into a new container
    /// - [`Result::Err`] variant indicates fail to move due to non-exclusive access
    ///
    /// ### Bound
    /// This method has a "[`Container::Inner`]: [`Moveable`]" bound to respect possible move constructors.
    ///
    /// ### Example
    /// Moving from [`BoxContainer`] into [`ArcContainer`]:
    ///
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::container::{BoxContainer, ArcContainer, Container};
    /// let box_container = unsafe { BoxContainer::init_ptr((), |ptr| unsafe { core::ptr::write(ptr, 42) }) };
    /// let arc_container = box_container.try_mv::<ArcContainer<_>>(()).expect("Should always be able to move from Box");
    /// # }
    /// ```
    ///
    /// In fact, when moving from box (or any other exclusive container), `expect`ing can be avoided by using [`ExclusiveContainer::mv`]:
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::container::{Container, ExclusiveContainer, BoxContainer, ArcContainer};
    /// # let box_container = unsafe { BoxContainer::init_ptr((), |ptr| unsafe { core::ptr::write(ptr, 42) }) };
    /// let arc_container = box_container.mv::<ArcContainer<_>>(());
    /// # }
    /// ```
    ///
    /// Note that reversed operation can fail:
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use sdecay::container::{Container, BoxContainer, ArcContainer};
    /// let arc_container = unsafe { ArcContainer::init_ptr((), |ptr| unsafe { core::ptr::write(ptr, 42) }) };
    /// let box_container = arc_container.try_mv::<BoxContainer<_>>(()).expect("Should move from exclusive arc container");
    /// // but,
    /// let arc_container = unsafe { ArcContainer::init_ptr((), |ptr| unsafe { core::ptr::write(ptr, 42) }) };
    /// let arc_container2 = arc_container.clone();
    /// let box_container = arc_container.try_mv::<BoxContainer<_>>(()).expect_err("Should not move from shared arc container");
    /// # core::mem::drop(arc_container2);
    /// # }
    /// ```
    #[inline]
    fn try_mv<C: Container<Inner = Self::Inner>>(self, allocator: C::Allocator) -> Result<C, Self>
    where
        Self::Inner: Moveable,
    {
        self.try_move_out(|src| {
            // NOTE: `src` is valid for reads and points to a valid instance of `T`
            let mv = move |dst: *mut Self::Inner| {
                // NOTE: assuming `dst` is valid for writes sized as `T`
                // SAFETY: (see notes above)
                // - `src` is valid for reads
                // - `src` points to valid T
                // - `dst` is valid writes
                unsafe { C::Inner::mv(dst, src) }
            };
            // SAFETY: `mv` does initialize the value by calling `Moveable::mv`
            unsafe { C::init_ptr(allocator, mv) }
        })
    }

    /// Provided helper function initializing the [`Container`] by moving a value into it
    ///
    /// To be honest, this kinda defeat the purpose, so it's here just for easier initialization in examples, I guess
    #[inline]
    fn init_value(allocator: Self::Allocator, value: Self::Inner) -> Self {
        let w = |dst| {
            // SAFETY:
            // - `dst` is valid for writes of `T`
            unsafe {
                core::ptr::write(dst, value);
            }
        };
        // SAFETY: `w` writes a valid value into `dst`, initializing it
        unsafe { Self::init_ptr(allocator, w) }
    }
}

/// Extension of [`Container`] trait implemented for containers always having a unique data access
///
/// Note that this trait is safe, as it does not impose any implicit contract over [`Container`]
pub trait ExclusiveContainer: Container {
    /// Gets exclusive reference to inner value. Note that exclusive reference is always pinned
    fn inner(&mut self) -> Pin<&mut Self::Inner>;

    /// Moves value out of the container. `action` is called *exactly once*, and provided with a pointer to *contained* value
    ///
    /// ### UB warning
    /// See [`Container::try_move_out`] doc
    ///
    /// Please take care, while implementing this function.
    fn move_out<O>(self, action: impl FnOnce(*mut Self::Inner) -> O) -> O;

    /// Moves value out of the container into a different container
    ///
    /// ### Bound
    /// This method has a "[`Container::Inner`]: [`Moveable`]" bound to respect possible move constructors.
    ///
    /// ### Example
    /// Moving from [`BoxContainer`] into [`ArcContainer`]:
    ///
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// # use core::pin::Pin;
    /// # use sdecay::container::{ArcContainer, BoxContainer, Container, ExclusiveContainer};
    /// let box_container = unsafe { BoxContainer::init_ptr((), |ptr| unsafe { core::ptr::write(ptr, 42) }) };
    /// let arc_container = box_container.mv::<ArcContainer<_>>(());
    /// # }
    /// ```
    ///
    /// Note that reverse operation is not implemented, since [`ArcContainer`] is not an [`ExclusiveContainer`]:
    /// ```rust,compile_fail
    /// # #[cfg(feature = "std")] {
    /// # use core::pin::Pin;
    /// # use sdecay::container::{ArcContainer, BoxContainer};
    /// let arc_container = unsafe { ArcContainer::init_ptr((), |ptr| unsafe { core::ptr::write(ptr, 42) }) };
    /// let box_container = arc_container.mv::<BoxContainer<_>>(());
    /// # }
    /// # #[cfg(not(feature = "std"))] {
    /// # compile_fail!("Dummy compile fail, if `no_std`");
    /// # }
    /// ```
    fn mv<C: Container<Inner = Self::Inner>>(self, allocator: C::Allocator) -> C
    where
        Self::Inner: Moveable,
    {
        self.move_out(|src| {
            // NOTE: `src` is valid for reads and points to a valid instance of `T`
            let mv = move |dst: *mut Self::Inner| {
                // NOTE: assuming `dst` is valid for writes sized as `T`
                // SAFETY: (see notes above)
                // - `src` is valid for reads
                // - `src` points to valid T
                // - `dst` is valid writes
                unsafe { C::Inner::mv(dst, src) }
            };
            // SAFETY: `mv` does initialize the value by calling `Moveable::mv`
            unsafe { C::init_ptr(allocator, mv) }
        })
    }
}

macro_rules! impl_container_traits {
    ($c:ty $(| <$($args:tt),+> $(lt: <$($l:lifetime),+>)?)?) => {
        impl $(<$($args),+>)? AsRef<<$c as Container>::Inner> for $c {
            #[inline]
            fn as_ref(&self) -> &<$c as Container>::Inner {
                &*self
            }
        }

        impl $(<$($args),+>)? $c {
            #[doc = concat!("Same as `<&Self as IntoIterator>::into_iter(self)`")]
            #[inline]
            pub fn iter<'r0>(&'r0 self) -> <&'r0 Self as IntoIterator>::IntoIter
            where
                $($($($l : 'r0,)+)?)?
                for<'r1> &'r1 T: IntoIterator,
            {
                self.into_iter()
            }
        }

        impl <'r0 $(,$($args),+)?> IntoIterator for & 'r0 $c
        where
            $($($($l : 'r0,)+)?)?
            for<'r1> &'r1 T: IntoIterator,
        {
            type Item = <&'r0 T as IntoIterator>::Item;

            type IntoIter = <&'r0 T as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                <&'r0 T as IntoIterator>::into_iter(&*self)
            }
        }
    };
}

#[derive(Debug)]
#[doc(hidden)]
#[cfg(feature = "alloc")]
pub struct UninitBoxContainer<T>(Box<MaybeUninit<T>>);

/// [`Container`] implementation via [`Box`]
///
/// Implements [`ExclusiveContainer`]
///
/// ### Example
/// ```rust
/// # use core::pin::Pin;
/// # use sdecay::container::{BoxContainer, Container, ExclusiveContainer};
/// let mut container = BoxContainer::init_value((), 42);
/// let shared: &i32 = &container;
/// let exclusive: Pin<&mut i32> = container.inner();
/// let container2 = container.mv::<BoxContainer<_>>(());
/// ```
#[derive(Debug)]
#[cfg(feature = "alloc")]
pub struct BoxContainer<T>(Pin<Box<T>>);

#[cfg(feature = "alloc")]
impl<T> Deref for BoxContainer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "alloc")]
impl<T: core::fmt::Display> core::fmt::Display for BoxContainer<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        T::fmt(&self.0, f)
    }
}

#[cfg(feature = "alloc")]
// SAFETY:
// - `UninitBoxContainer` does not introduce shared ownership
// - Moving `BoxContainer` does not move `T`
// - Dropping `BoxContainer` drops the `T`
unsafe impl<T> Container for BoxContainer<T> {
    type Allocator = ();
    type Inner = T;
    type Uninit = UninitBoxContainer<T>;

    #[inline]
    fn uninit(_allocator: ()) -> Self::Uninit {
        UninitBoxContainer(Box::new_uninit())
    }

    #[inline]
    fn uninit_inner_ptr(uninit: &mut Self::Uninit) -> *mut Self::Inner {
        uninit.0.as_mut_ptr()
    }

    #[inline]
    unsafe fn init(uninit: Self::Uninit) -> Self {
        // SAFETY: value contained in the box must be init (function requirement)
        let init_ptr = unsafe { uninit.0.assume_init() };
        // SAFETY:
        // - exclusive reference is only ever exposed as `Pin<&mut T>`
        // - memory is unpinned only after drop call, or `Container::move_out` (`core::mem::forget` at most)
        let pin = unsafe { Pin::new_unchecked(init_ptr) };
        Self(pin)
    }

    #[inline]
    fn try_inner(&mut self) -> Option<Pin<&mut Self::Inner>> {
        Some(self.inner())
    }

    #[inline]
    fn try_move_out<O>(self, action: impl FnOnce(*mut Self::Inner) -> O) -> Result<O, Self> {
        Ok(self.move_out(action))
    }
}

#[cfg(feature = "alloc")]
impl<T> ExclusiveContainer for BoxContainer<T> {
    #[inline]
    fn inner(&mut self) -> Pin<&mut Self::Inner> {
        self.0.as_mut()
    }

    #[inline]
    fn move_out<O>(self, action: impl FnOnce(*mut Self::Inner) -> O) -> O {
        // SAFETY: none of the operations below will expose `&mut T` to the caller, or move the data directly. Any possible movement logic is handled by the `action` closure, and it's up to it to call any sort of destructor and such
        //
        // After this call, contained `T` is assumed to be dropped or moved in a way respecting all `T`s invariants. Regardless, data is not read or assumed to be a valid `T` again
        let bx = unsafe { Pin::into_inner_unchecked(self.0) };
        let ptr = Box::into_raw(bx);
        let res = action(ptr);
        let uptr = ptr.cast::<MaybeUninit<T>>(); // `ptr`'s pointee was invalidated by `action`
        // SAFETY:
        // - `uptr` is derived from `ptr`, obtained from call to `Box::into_raw`
        // - `MaybeUninit<T>` has the same layout as `T`
        let ubx = unsafe { Box::from_raw(uptr) };
        // free the box allocation
        core::mem::drop(ubx);
        res
    }
}

#[cfg(feature = "alloc")]
impl_container_traits!(BoxContainer<T> | <T>);

// This is a helper function specifically designed to help calling `Ptr`'s functions. Care should be taken to uphold pin invariants while doing so
#[cfg(feature = "alloc")]
unsafe fn pin_inner_mut<Ptr>(pin: &mut Pin<Ptr>) -> &mut Ptr {
    let ptr = core::ptr::from_mut(pin).cast();
    // SAFETY:
    // - ptr validity: `Pin<Ptr>` has the same layout as `Ptr`
    // - pin invariant: see function doc
    unsafe { &mut *ptr }
}

#[derive(Debug)]
#[doc(hidden)]
#[cfg(feature = "alloc")]
pub struct UninitRcContainer<T>(Rc<MaybeUninit<T>>);

/// [`Container`] implementation via [`Rc`]
///
/// ### Example
/// ```rust
/// # use core::pin::Pin;
/// # use sdecay::container::{RcContainer, Container};
/// let mut container = RcContainer::init_value((), 42);
///
/// let shared: &i32 = &container;
/// let exclusive: Pin<&mut i32> = container.try_inner().expect("Single Rc should have exclusive access");
/// let mut container2 = container.try_mv::<RcContainer<_>>(()).expect("Single Rc should be always successfully moved out of");
///
/// let container3 = container2.clone(); // container is not longer exclusive
///
/// assert!(container2.try_inner().is_none(), "Non-excluive Rc should not be able to get &mut");
/// let shared: &i32 = &container2; // shared reference is still ok
/// let _ = container2.try_mv::<RcContainer<_>>(()).expect_err("Non-exclusive Rc should not be able to move the value");
/// ```
#[derive(Debug)]
#[cfg(feature = "alloc")]
pub struct RcContainer<T>(Pin<Rc<T>>);

#[cfg(feature = "alloc")]
impl<T> Clone for RcContainer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(feature = "alloc")]
impl<T> Deref for RcContainer<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "alloc")]
impl<T: core::fmt::Display> core::fmt::Display for RcContainer<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        T::fmt(&self.0, f)
    }
}

#[cfg(feature = "alloc")]
// SAFETY:
// - `UninitRcContainer` does not introduce shared ownership (it does contain `Rc`, but it's SO capability is encapsulated)
// - Moving `RcContainer` does not move `T`
// - Dropping all `RcContainer`s referring to the same `T`, drops the `T`
unsafe impl<T> Container for RcContainer<T> {
    type Allocator = ();
    type Inner = T;
    type Uninit = UninitRcContainer<T>;

    #[inline]
    fn uninit(_allocator: ()) -> Self::Uninit {
        UninitRcContainer(Rc::new_uninit())
    }

    #[inline]
    fn uninit_inner_ptr(uninit: &mut Self::Uninit) -> *mut Self::Inner {
        Rc::get_mut(&mut uninit.0).unwrap().as_mut_ptr()
    }

    #[inline]
    unsafe fn init(uninit: Self::Uninit) -> Self {
        // SAFETY: value contained in the rc must be init (function invariant)
        let init_ptr = unsafe { uninit.0.assume_init() };
        // SAFETY:
        // - exclusive reference is only ever exposed as `Pin<&mut T>`
        // - memory is unpinned only after drop call, or `Container::move_out` (`core::mem::forget` at least)
        let pin = unsafe { Pin::new_unchecked(init_ptr) };
        Self(pin)
    }

    #[inline]
    fn try_inner(&mut self) -> Option<Pin<&mut Self::Inner>> {
        // SAFETY:
        // - I'll only use `&mut Rc` to call `Rc::get_mut` here
        // - `Rc::get_mut` is not interacting with `T` at all (apart from creating reference to it)
        // - obtained reference is pinned again shortly below, and only returned as pinned
        let rc = unsafe { pin_inner_mut(&mut self.0) };
        if let Some(refm) = Rc::get_mut(rc) {
            // SAFETY: `&mut T` refers to already-pinned `T`, that was unpinned right above to use `Rc::get_mut`
            Some(unsafe { Pin::new_unchecked(refm) })
        } else {
            None
        }
    }

    fn try_move_out<O>(mut self, action: impl FnOnce(*mut Self::Inner) -> O) -> Result<O, Self> {
        // NOTE: I could've used `Rc::strong_count` here, but it would require `unsafe` code anyways, so I opted out for uniformity with implementation for `Arc`

        // SAFETY: obtained reference will only be used for `Rc::get_mut`, which does not interact with the value in any way
        let rc = unsafe { pin_inner_mut(&mut self.0) };
        if Rc::get_mut(rc).is_none() {
            return Err(self);
        }
        // SAFETY: so, this is a tricky part, but here's a quick rundown of why is this ok:
        // - `*mut T` to inner value is obtained, but no reads and/or writes are performed here
        // - then, `*mut T` is given to `action` for potential reads and/or writes, and not assumed to contain valid `T` after that. Essentially, `action` is supposed to act as `T`'s destructor
        let rc = unsafe { Pin::into_inner_unchecked(self.0) };
        let ptr_c = Rc::into_raw(rc); // disassemble the `Rc`, basically
        let ptr_m = ptr_c.cast_mut(); // currently we have an exclusive access to data part of `Rc`'s allocation
        let res = action(ptr_m);
        let uptr_c = ptr_c.cast::<MaybeUninit<T>>(); // contained `T` is not valid anymore - treat it as `MaybeUninit`
        // SAFETY:
        // - `uptr_C` is derived from `ptr_c`, obtained from call to `Rc::into_raw`
        // - `MaybeUninit<T>` has the same layout as `T`
        let urc = unsafe { Rc::from_raw(uptr_c) };
        // free Rc's allocation
        // (not really; there potentially can be `Weak`s keeping the allocation alive, which this is in no way intended or supported - this drop still will not be an issue)
        core::mem::drop(urc);
        Ok(res)
    }
}

#[cfg(feature = "alloc")]
impl_container_traits!(RcContainer<T> | <T>);

#[derive(Debug)]
#[doc(hidden)]
#[cfg(feature = "std")]
pub struct UninitArcContainer<T>(Arc<MaybeUninit<T>>);

/// [`Container`] implementation via [`Arc`]
///
/// ### Example
/// ```rust
/// # use core::pin::Pin;
/// # use sdecay::container::{Container, ArcContainer};
/// let mut container = ArcContainer::init_value((), 42);
///
/// let shared: &i32 = &container;
/// let exclusive: Pin<&mut i32> = container.try_inner().expect("Single Arc should have exclusive access");
/// let mut container2 = container.try_mv::<ArcContainer<_>>(()).expect("Single Arc should be always successfully moved out of");
///
/// let container3 = container2.clone(); // container is not longer exclusive
///
/// assert!(container2.try_inner().is_none(), "Non-excluive Arc should not be able to get &mut");
/// let shared: &i32 = &container2; // shared reference is still ok
/// let _ = container2.try_mv::<ArcContainer<_>>(()).expect_err("Non-exclusive Arc should not be able to move the value");
/// ```
#[derive(Debug)]
#[cfg(feature = "std")]
pub struct ArcContainer<T>(Pin<Arc<T>>);

#[cfg(feature = "std")]
impl<T> Clone for ArcContainer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(feature = "std")]
impl<T> Deref for ArcContainer<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "std")]
impl<T: core::fmt::Display> core::fmt::Display for ArcContainer<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        T::fmt(&self.0, f)
    }
}

#[cfg(feature = "std")]
// SAFETY:
// - `UninitArcContainer` does not introduce shared ownership (it does contain `Arc`, but it's SO capability is encapsulated)
// - Moving `ArcContainer` does not move `T`
// - Dropping all `ArcContainer`s referring to the same `T`, drops the `T`
unsafe impl<T> Container for ArcContainer<T> {
    type Allocator = ();
    type Inner = T;
    type Uninit = UninitArcContainer<T>;

    #[inline]
    fn uninit(_allocator: ()) -> Self::Uninit {
        UninitArcContainer(Arc::new_uninit())
    }

    #[inline]
    fn uninit_inner_ptr(uninit: &mut Self::Uninit) -> *mut Self::Inner {
        Arc::get_mut(&mut uninit.0).unwrap().as_mut_ptr()
    }

    #[inline]
    unsafe fn init(uninit: Self::Uninit) -> Self {
        // SAFETY: value contained in the arc must be init (function invariant)
        let init_ptr = unsafe { uninit.0.assume_init() };
        // SAFETY:
        // - exclusive reference is only ever exposed as `Pin<&mut T>`
        // - memory is unpinned only after drop call, or `Container::move_out` (`core::mem::forget` at least)
        let pin = unsafe { Pin::new_unchecked(init_ptr) };
        Self(pin)
    }

    #[inline]
    fn try_inner(&mut self) -> Option<Pin<&mut Self::Inner>> {
        // SAFETY:
        // - I'll only use `&mut Arc` to call `Arc::get_mut` here
        // - `Arc::get_mut` is not interacting with `T` at all (apart from creating reference to it)
        // - obtained reference is pinned again shortly below, and only returned as pinned
        let rc = unsafe { pin_inner_mut(&mut self.0) };
        if let Some(refm) = Arc::get_mut(rc) {
            // SAFETY: `&mut T` refers to already-pinned `T`, that was unpinned right above to use `Arc::get_mut`
            Some(unsafe { Pin::new_unchecked(refm) })
        } else {
            None
        }
    }

    fn try_move_out<O>(mut self, action: impl FnOnce(*mut Self::Inner) -> O) -> Result<O, Self> {
        // NOTE: `Arc::strong_count == 1` does not guarantee proper memory ordering, since it loads counter with `Relaxed` ordering. `Arc` does have proper `is_unique` function, but the only way to use it is indirectly through `Arc::get_mut` call.

        // SAFETY: obtained reference will only be used for `Arc::get_mut`, which does not interact with the value in any way
        let rc = unsafe { pin_inner_mut(&mut self.0) };
        if Arc::get_mut(rc).is_none() {
            return Err(self);
        }

        // SAFETY: so, this is a tricky part, but here's a quick rundown of why is this ok:
        // - `*mut T` to inner value is obtained, but no reads and/or writes are performed here
        // - then, `*mut T` is given to `action` for potential reads and/or writes, and not assumed to contain valid `T` after that. Essentially, `action` is supposed to act as `T`'s destructor
        let arc = unsafe { Pin::into_inner_unchecked(self.0) };
        let ptr_c = Arc::into_raw(arc); // disassemble the `Arc`, basically
        let ptr_m = ptr_c.cast_mut(); // currently we have an exclusive access to data part of `Arc`'s allocation
        let res = action(ptr_m);
        let uptr_c = ptr_c.cast::<MaybeUninit<T>>(); // contained `T` is not valid anymore - treat it as `MaybeUninit`
        // SAFETY:
        // - `uptr_C` is derived from `ptr_c`, obtained from call to `Arc::into_raw`
        // - `MaybeUninit<T>` has the same layout as `T`
        let urc = unsafe { Arc::from_raw(uptr_c) };
        // free Arc's allocation
        // (not really; there potentially can be `Weak`s keeping the allocation alive, which this is in no way intended or supported - this drop still will not be an issue)
        core::mem::drop(urc);
        Ok(res)
    }
}

#[cfg(feature = "std")]
impl_container_traits!(ArcContainer<T> | <T>);

#[derive(Debug)]
#[doc(hidden)]
pub struct UninitRefContainer<'r, T>(&'r mut MaybeUninit<T>);

/// [`Container`] implementation via exclusive reference to [`MaybeUninit`]
///
/// Implements [`ExclusiveContainer`]
///
/// ### Example
/// ```rust
/// # use core::{mem::MaybeUninit, pin::Pin};
/// # use sdecay::container::{RefContainer, Container, ExclusiveContainer};
/// let mut tmp = MaybeUninit::uninit();
/// let mut container = RefContainer::init_value(&mut tmp, 42);
/// let shared: &i32 = &container;
/// let exclusive: Pin<&mut i32> = container.inner();
/// let mut tmp = MaybeUninit::uninit();
/// let container2 = container.mv::<RefContainer<_>>(&mut tmp);
/// ```
/// Note that this container has a non-`()` allocator, and you might want to explicitly allocate `MaybeUninit`:
/// ```rust
/// # use core::mem::MaybeUninit;
/// # use sdecay::container::{RefContainer, Container};
/// let mut tmp = MaybeUninit::uninit();
/// let container = RefContainer::init_value(&mut tmp, 42);
/// ```
/// This works just fine too, although a bit verbose
#[derive(Debug)]
pub struct RefContainer<'r, T>(Option<Pin<&'r mut T>>);

impl<T> Deref for RefContainer<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> Drop for RefContainer<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // I'd like to move out of option (no good reason for that, really)
        let Some(pin): Option<Pin<&mut T>> = self.0.take() else {
            // pin was taken away by `Container::move_out` method. Nothing to do: data was dropped already
            return;
        };
        // now I need to manually drop the value behind the exclusive reference
        //
        // although exclusive reference provides an exclusive access to data, it can't be alive at all, after data is invalidated - we need to use raw `*mut T` and `core::mem::drop_in_place` call
        // SAFETY: data will not be moved out, and will be dropped in a couple lines
        let rf = unsafe { Pin::into_inner_unchecked(pin) };
        // pin no longer exists
        let ptr = core::ptr::from_mut(rf);
        // reference no longer exists
        // SAFETY:
        // - `ptr` points to valid `T`, as it was derived from a `&mut T`
        // - at this point, data is only referred to as `*mut T` (and possibly as `&mut MaybeUninit<T>` from the outer scope)
        unsafe { core::ptr::drop_in_place(ptr) };
    }
}

impl<T: core::fmt::Display> core::fmt::Display for RefContainer<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        T::fmt(self.0.as_ref().unwrap(), f)
    }
}

// SAFETY:
// - `UninitRefContainer` does not introduce shared ownership
// - Moving `RefContainer` does not move `T`
// - Dropping `RefContainer` drops the `T`
unsafe impl<'r, T> Container for RefContainer<'r, T> {
    type Allocator = &'r mut MaybeUninit<T>;
    type Inner = T;
    type Uninit = UninitRefContainer<'r, T>;

    #[inline]
    fn uninit(allocator: Self::Allocator) -> Self::Uninit {
        UninitRefContainer(allocator)
    }

    #[inline]
    fn uninit_inner_ptr(uninit: &mut Self::Uninit) -> *mut Self::Inner {
        uninit.0.as_mut_ptr()
    }

    #[inline]
    unsafe fn init(uninit: Self::Uninit) -> Self {
        // SAFETY: value pointed by the `&mut MaybeUninit` must be init (function invariant)
        let ref_mut = unsafe { uninit.0.assume_init_mut() };
        // SAFETY:
        // - exclusive reference is only ever exposed as `Pin<&mut T>`
        // - memory is unpinned only after drop call, or `Container::move_out` (`core::mem::forget` at most)
        let pin = unsafe { Pin::new_unchecked(ref_mut) };
        Self(Some(pin))
    }

    #[inline]
    fn try_inner(&mut self) -> Option<Pin<&mut Self::Inner>> {
        Some(self.inner())
    }

    #[inline]
    fn try_move_out<O>(self, action: impl FnOnce(*mut Self::Inner) -> O) -> Result<O, Self> {
        Ok(self.move_out(action))
    }
}

impl<T> ExclusiveContainer for RefContainer<'_, T> {
    #[inline]
    fn inner(&mut self) -> Pin<&mut Self::Inner> {
        self.0.as_mut().unwrap().as_mut()
    }

    #[inline]
    fn move_out<O>(mut self, action: impl FnOnce(*mut Self::Inner) -> O) -> O {
        // SAFETY: none of the operations below will expose `&mut T` to the caller, or move the data directly. Any possible movement logic is handled by the `action` closure, and it's up to it to call any sort of destructor and such
        //
        // After this call, contained `T` is assumed to be dropped or moved in a way respecting all `T`s invariants. Regardless, data is not read or assumed to be a valid `T` again
        let pin = self
            .0
            .take()
            .expect("Should contain the pin, since it's only ever taken out by this function and a destructor (both can only be called once)");
        // SAFETY: extracted `&mut T` will only be passed to `action`, which is assumed to call value's destructor
        let rf = unsafe { Pin::into_inner_unchecked(pin) };
        // pin no longer exists
        let ptr = core::ptr::from_mut(rf);
        // `&mut T` no longer exists
        action(ptr) // value is dropped, result returned
    }
}

impl_container_traits!(RefContainer<'r, T> | <'r, T> lt: <'r>);

#[cfg(test)]
mod tests;
