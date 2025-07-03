//! Defines safe outer database types
//!
//! Unsafe: no

use core::{fmt::Debug, ops::Deref, pin::Pin};

use crate::{
    as_cpp_string::AsCppString,
    container::{Container, RefContainer},
    wrapper::{CppException, SandiaDecayDataBase},
};

/// `SandiaDecay`'s database with no info actually stored. Technically, it's already initialized, but I assume none of the calls would return meaningful info (so none are exposed)
///
/// To be used in any meaningful way, you need to obtain [`GenericDatabase`] using one of the following methods:
/// - [`GenericUninitDatabase::init`]
/// - [`GenericUninitDatabase::init_bytes`]
/// - [`GenericUninitDatabase::init_env`]
///
/// See respective docs for details
pub struct GenericUninitDatabase<C: Container<Inner = SandiaDecayDataBase>>(C);

/// Not initialized database stored in the [`alloc::boxed::Box`]
///
/// For more details, see [`GenericUninitDatabase`]
#[cfg(feature = "alloc")]
pub type UninitDatabase =
    GenericUninitDatabase<crate::container::BoxContainer<SandiaDecayDataBase>>;
/// Not initialized database stored in the [`std::sync::Arc`]
///
/// For more details, see [`GenericUninitDatabase`]
#[cfg(feature = "alloc")]
pub type UninitSharedDatabase =
    GenericUninitDatabase<crate::container::ArcContainer<SandiaDecayDataBase>>;
/// Not initialized database stored in wherever the `&`[`core::mem::MaybeUninit`] pointed to
///
/// For more details, see [`GenericUninitDatabase`]
pub type UninitLocalDatabase<'l> = GenericUninitDatabase<RefContainer<'l, SandiaDecayDataBase>>;

impl<C: Container<Inner = SandiaDecayDataBase>> Debug for GenericUninitDatabase<C> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("UninintDatabase")
    }
}

impl<C: Container<Inner = SandiaDecayDataBase>> Default for GenericUninitDatabase<C>
where
    C::Allocator: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Container<Inner = SandiaDecayDataBase>> GenericUninitDatabase<C> {
    /// Allocates empty database
    #[inline]
    pub fn new_in(allocator: C::Allocator) -> Self {
        Self(SandiaDecayDataBase::new(allocator))
    }

    /// Same as [`GenericUninitDatabase::new_in`], but allocator is created via [`Default::default`]
    #[inline]
    pub fn new() -> Self
    where
        C::Allocator: Default,
    {
        Self(SandiaDecayDataBase::new(C::Allocator::default()))
    }

    fn get_mut(&mut self) -> Pin<&mut SandiaDecayDataBase> {
        self.0
            .try_inner()
            .expect("Uninit database should not be in a shared container")
    }

    /// Attempts to initialize the database via path to the database `.xml` file
    ///
    /// ### Returns
    /// - [`Result::Ok`] indicates successfully initialized database
    /// - [`Result::Err`] indicates a failure to initialize a database. Actually returned value is a tuple of uninitialized database and exception thrown on C++ side
    ///
    /// ### Example
    /// An example using [`crate::container::BoxContainer`] for storage:
    /// ```rust,no_run
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::database::UninitDatabase;
    /// // assuming `database.xml` contains database data
    /// let database = UninitDatabase::new()
    ///     .init("database.xml")
    ///     .expect("`database.xml` should exist and contain a valid database data");
    /// # }
    /// ```
    ///
    /// Note, that path can be any [`AsCppString`] implementor - see it's doc to find the most convenient for you
    pub fn init(
        mut self,
        path: impl AsCppString,
    ) -> Result<GenericDatabase<C>, (GenericUninitDatabase<C>, CppException)> {
        match self.get_mut().init_path(path) {
            Ok(()) => Ok(GenericDatabase(self.0)),
            Err(exception) => Err((self, exception)),
        }
    }

    /// Attempts to initialize the database via `xml` data
    ///
    /// ### Returns
    /// - [`Result::Ok`] indicates successfully initialized database
    /// - [`Result::Err`] indicates a failure to initialize a database. Actually returned value is a tuple of uninitialized database and exception thrown on C++ side
    ///
    /// ### Example
    /// An example using [`crate::container::BoxContainer`] for storage:
    /// ```rust,no_run
    /// # #[cfg(feature = "alloc")] {
    /// let data: &[u8] = br#"<?xml version="1.0"?><document>...</document>"#; // assuming `data contains valid database data`
    /// # use sdecay::database::UninitDatabase;
    /// let database = UninitDatabase::new()
    ///     .init_bytes(data)
    ///     .expect("Should provide valid database data");
    /// # }
    /// ```
    pub fn init_bytes(
        mut self,
        bytes: impl AsRef<[u8]>,
    ) -> Result<GenericDatabase<C>, (GenericUninitDatabase<C>, CppException)> {
        match self.get_mut().init_bytes(bytes) {
            Ok(()) => Ok(GenericDatabase(self.0)),
            Err(exception) => Err((self, exception)),
        }
    }
}

/// Error while initializing database by path from environment variable
///
/// Returned by [`GenericUninitDatabase::init_env`] and [`GenericDatabase::from_env`]
#[derive(Debug, Error)]
pub enum EnvInitError {
    /// `SANDIA_DATABASE_PATH` is not present in the environment
    #[error("No `SANDIA_DATABASE_PATH` variable in the environment")]
    NoEnvVar,
    /// Envvar present, but exception thrown from C++ side
    #[error(transparent)]
    Exception(CppException),
}

impl<C: Container<Inner = SandiaDecayDataBase>> GenericUninitDatabase<C> {
    /// Attempts to initialize database by a path from `SANDIA_DATABASE_PATH` environment variable
    /// ### Returns
    /// - [`Result::Ok`] indicates successfully initialized database
    /// - [`Result::Err`] indicates a failure to initialize a database. Actually returned value is a tuple of uninitialized database and [`EnvInitError`]
    ///
    /// ### Example
    /// An example using [`crate::container::BoxContainer`] for storage:
    /// ```rust,no_run
    /// // assuming `SANDIA_DATABASE_PATH` envvar contains path to database file
    /// # use sdecay::database::UninitDatabase;
    /// let database = UninitDatabase::new()
    ///     .init_env()
    ///     .expect("`SANDIA_DATABASE_PATH` should contain path to data, and data should be valid");
    /// ```
    #[cfg(feature = "std")]
    #[inline]
    pub fn init_env(self) -> Result<GenericDatabase<C>, (GenericUninitDatabase<C>, EnvInitError)> {
        let Some(env_path) = std::env::var_os("SANDIA_DATABASE_PATH") else {
            return Err((self, EnvInitError::NoEnvVar));
        };
        self.init(env_path)
            .map_err(|(uninit, exception)| (uninit, EnvInitError::Exception(exception)))
    }
}

impl<C: Container<Inner = SandiaDecayDataBase>> GenericUninitDatabase<C> {
    /// Creates initialized database from embedded "default" database
    ///
    /// ### Example
    /// Using [`crate::container::BoxContainer`] for storage:
    /// ```rust
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::database::UninitDatabase;
    /// let database = UninitDatabase::new().init_vendor();
    /// # }
    /// ```
    #[cfg(feature = "database")]
    #[inline]
    pub fn init_vendor(self) -> GenericDatabase<C> {
        self.init_bytes(sdecay_sys::database::DATABASE)
            .expect("Embedded database should be valid")
    }

    /// Creates initialized database from embedded "min" database
    ///
    /// ### Example
    /// Using [`crate::container::BoxContainer`] for storage:
    /// ```rust
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::database::UninitDatabase;
    /// let database = UninitDatabase::new().init_vendor_min();
    /// # }
    /// ```
    #[cfg(feature = "database-min")]
    #[inline]
    pub fn init_vendor_min(self) -> GenericDatabase<C> {
        self.init_bytes(sdecay_sys::database::DATABASE_MIN)
            .expect("Embedded database should be valid")
    }

    /// Creates initialized database from embedded "nocoinc-min" database
    ///
    /// ### Example
    /// Using [`crate::container::BoxContainer`] for storage:
    /// ```rust
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::database::UninitDatabase;
    /// let database = UninitDatabase::new().init_vendor_nocoinc_min();
    /// # }
    /// ```
    #[cfg(feature = "database-nocoinc-min")]
    #[inline]
    pub fn init_vendor_nocoinc_min(self) -> GenericDatabase<C> {
        self.init_bytes(sdecay_sys::database::DATABASE_NOCOINC_MIN)
            .expect("Embedded database should be valid")
    }
}

/// Initialized and data-enabled `SandiaDecay` database. Can be created from [`GenericUninitDatabase`] (see it's doc), or directly via
/// - [`GenericDatabase::from_path`] ([`GenericDatabase::from_path_in`])
/// - [`GenericDatabase::from_bytes`] ([`GenericDatabase::from_bytes_in`])
/// - [`GenericDatabase::from_env`] ([`GenericDatabase::from_env_in`])
///
/// See functions below for usage examples
#[derive(Debug, Clone)]
pub struct GenericDatabase<C: Container<Inner = SandiaDecayDataBase>>(C);

/// Initialized database stored in the [`alloc::boxed::Box`]
///
/// For more details, see [`GenericDatabase`]
#[cfg(feature = "alloc")]
pub type Database = GenericDatabase<crate::container::BoxContainer<SandiaDecayDataBase>>;
/// Initialized database stored in the [`std::sync::Arc`]
///
/// For more details, see [`GenericDatabase`]
#[cfg(feature = "alloc")]
pub type SharedDatabase = GenericDatabase<crate::container::ArcContainer<SandiaDecayDataBase>>;
/// Initialized database stored in wherever the `&`[`core::mem::MaybeUninit`] pointed to
///
/// For more details, see [`GenericDatabase`]
pub type LocalDatabase<'l> = GenericDatabase<RefContainer<'l, SandiaDecayDataBase>>;

impl<C: Container<Inner = SandiaDecayDataBase>> GenericDatabase<C> {
    /// Attempts to create initialized database via path to the database `.xml` file
    ///
    /// This is the same as consequent [`UninitDatabase::new`] and [`UninitDatabase::init`] calls
    ///
    /// ### Returns
    /// - [`Result::Ok`] successfully initialized database
    /// - [`Result::Err`] contains a description of panic from C++ side
    ///
    /// ### Example
    /// An example using [`crate::container::BoxContainer`] for storage:
    /// ```rust,no_run
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::database::Database;
    /// // assuming `database.xml` contains database data
    /// let database = Database::from_path("database.xml")
    ///     .expect("`database.xml` should exist and contain a valid database data");
    /// # }
    /// ```
    ///
    /// Note, that path can be any [`AsCppString`] implementor - see it's doc to find the most convenient for you
    #[inline]
    pub fn from_path_in(
        allocator: C::Allocator,
        path: impl AsCppString,
    ) -> Result<Self, CppException> {
        match GenericUninitDatabase::new_in(allocator).init(path) {
            Ok(init) => Ok(init),
            Err((_, error)) => Err(error),
        }
    }

    /// Same as [`Self::from_path_in`], but uses `C::Allocator`'s [`Default`] implementation to obtain the allocator
    #[inline]
    pub fn from_path(path: impl AsCppString) -> Result<Self, CppException>
    where
        C::Allocator: Default,
    {
        Self::from_path_in(C::Allocator::default(), path)
    }

    /// Attempts to create initialized database via `xml` data
    ///
    /// This is the same as consequent [`UninitDatabase::new`] and [`UninitDatabase::init_bytes`] calls
    ///
    /// ### Returns
    /// - [`Result::Ok`] successfully initialized database
    /// - [`Result::Err`] contains a description of panic from C++ side
    ///
    /// ### Example
    /// An example using [`crate::container::BoxContainer`] for storage:
    /// ```rust,no_run
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::database::Database;
    /// let data: &[u8] = br#"<?xml version="1.0"?><document>...</document>"#; // assuming `data contains valid database data`
    /// let database = Database::from_bytes(data)
    ///     .expect("Should provide valid database data");
    /// # }
    /// ```
    #[inline]
    pub fn from_bytes_in(
        allocator: C::Allocator,
        bytes: impl AsRef<[u8]>,
    ) -> Result<Self, CppException> {
        match GenericUninitDatabase::new_in(allocator).init_bytes(bytes) {
            Ok(init) => Ok(init),
            Err((_, error)) => Err(error),
        }
    }

    /// Same as [`Self::from_bytes_in`], but uses `C::Allocator`'s [`Default`] implementation to obtain the allocator
    #[inline]
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, CppException>
    where
        C::Allocator: Default,
    {
        Self::from_bytes_in(C::Allocator::default(), bytes)
    }

    /// Attempts to create initialized database by a path from `SANDIA_DATABASE_PATH` environment variable
    ///
    /// This is the same as consequent [`UninitDatabase::new`] and [`UninitDatabase::init_bytes`] calls
    ///
    /// ### Returns
    /// - [`Result::Ok`] successfully initialized database
    /// - [`Result::Err`] contains a description of panic from C++ side
    ///
    /// ### Example
    /// An example using [`crate::container::BoxContainer`] for storage:
    /// ```rust,no_run
    /// # use sdecay::database::Database;
    /// // assuming `SANDIA_DATABASE_PATH` envvar contains path to database file
    /// let database = Database::from_env()
    ///     .expect("`SANDIA_DATABASE_PATH` should contain path to data, and data should be valid");
    /// ```
    #[cfg(feature = "std")]
    #[inline]
    pub fn from_env_in(allocator: C::Allocator) -> Result<Self, EnvInitError> {
        match GenericUninitDatabase::new_in(allocator).init_env() {
            Ok(init) => Ok(init),
            Err((_, error)) => Err(error),
        }
    }

    /// Same as [`Self::from_env_in`], but uses `C::Allocator`'s [`Default`] implementation to obtain the allocator
    #[cfg(feature = "std")]
    #[inline]
    pub fn from_env() -> Result<Self, EnvInitError>
    where
        C::Allocator: Default,
    {
        Self::from_env_in(C::Allocator::default())
    }

    /// Creates initialized database from embedded "default" database
    ///
    /// This is the same as consequent [`UninitDatabase::new`] and [`UninitDatabase::init_vendor`] calls
    ///
    /// ### Example
    /// Using [`crate::container::BoxContainer`] as storage:
    /// ```rust
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::database::Database;
    /// // assuming `database` feature is enabled
    /// let database = Database::vendor();
    /// # }
    /// ```
    #[cfg(feature = "database")]
    #[inline]
    pub fn vendor_in(allocator: C::Allocator) -> Self {
        GenericUninitDatabase::new_in(allocator).init_vendor()
    }

    /// Same as [`Self::vendor_in`], but uses `C::Allocator`'s [`Default`] implementation to obtain the allocator
    #[cfg(feature = "database")]
    #[inline]
    pub fn vendor() -> Self
    where
        C::Allocator: Default,
    {
        Self::vendor_in(C::Allocator::default())
    }

    /// Creates initialized database from embedded "min" database
    ///
    /// This is the same as consequent [`UninitDatabase::new`] and [`UninitDatabase::init_vendor_min`] calls
    ///
    /// ### Example
    /// Using [`crate::container::BoxContainer`] as storage:
    /// ```rust
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::database::Database;
    /// // assuming `database-min` feature is enabled
    /// let database = Database::vendor_min();
    /// # }
    /// ```
    #[cfg(feature = "database-min")]
    #[inline]
    pub fn vendor_min_in(allocator: C::Allocator) -> Self {
        GenericUninitDatabase::new_in(allocator).init_vendor_min()
    }

    /// Same as [`Self::vendor_min_in`], but uses `C::Allocator`'s [`Default`] implementation to obtain the allocator
    #[cfg(feature = "database-min")]
    #[inline]
    pub fn vendor_min() -> Self
    where
        C::Allocator: Default,
    {
        Self::vendor_min_in(C::Allocator::default())
    }

    /// Creates initialized database from embedded "nocoinc-min" database
    ///
    /// This is the same as consequent [`UninitDatabase::new`] and [`UninitDatabase::init_vendor_nocoinc_min`] calls
    ///
    /// ### Example
    /// Using [`crate::container::BoxContainer`] as storage:
    /// ```rust
    /// # #[cfg(feature = "alloc")] {
    /// # use sdecay::database::Database;
    /// let database = Database::vendor_nocoinc_min();
    /// # }
    /// ```
    #[cfg(feature = "database-nocoinc-min")]
    #[inline]
    pub fn vendor_nocoinc_min_in(allocator: C::Allocator) -> Self {
        GenericUninitDatabase::new_in(allocator).init_vendor_nocoinc_min()
    }

    /// Same as [`Self::vendor_nocoinc_min_in`], but uses `C::Allocator`'s [`Default`] implementation to obtain the allocator
    #[cfg(feature = "database-nocoinc-min")]
    #[inline]
    pub fn vendor_nocoinc_min() -> Self
    where
        C::Allocator: Default,
    {
        Self::vendor_nocoinc_min_in(C::Allocator::default())
    }

    /// Resets the database, returning it into uninitialized (empty) state
    ///
    /// Note, that this call **is not required** to properly drop the database - all of the resources are freed upon drop
    ///
    /// ### Returns
    /// [`Option::None`] represents failure to reset the database, due to it being shared by multiple containers
    ///
    /// ### Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// // database stored in the `Box` can always be reset:
    /// # use sdecay::database::Database;
    /// let database = Database::from_env().unwrap();
    /// let _uninit = database.reset().expect("Should be able to reset a database behind exclusive pointer");
    ///
    /// // database stored in the `Arc` can be reset while not shared:
    /// # use sdecay::database::SharedDatabase;
    /// let shared_database = SharedDatabase::from_env().unwrap();
    /// let _uninit = shared_database.reset().expect("Should be able to reset the database, while it is not shared");
    ///
    /// // once shared, databased can no longer be dropped:
    /// let shared_database = SharedDatabase::from_env().unwrap();
    /// let shared_database2 = shared_database.clone();
    /// let _ = shared_database.reset().expect_err("Should not reset database in a shared state");
    /// // once not shared, reset is possible once again:
    /// let _uninit = shared_database2.reset().expect("Database is not longer shared, should be able to reset");
    /// # }
    /// ```
    #[inline]
    pub fn reset(mut self) -> Result<GenericUninitDatabase<C>, Self> {
        let Some(pin) = self.0.try_inner() else {
            return Err(self);
        };
        pin.reset();
        Ok(GenericUninitDatabase(self.0))
    }
}

impl<C: Container<Inner = SandiaDecayDataBase>> Deref for GenericDatabase<C> {
    type Target = SandiaDecayDataBase;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
