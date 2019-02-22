// vim: tw=80

/// Common methods of the Common struct
// Ideally methods like with and withf that are identical for all
// Expectation types would be defined here, too.  But that's not possible for
// methods that need any of the variable-length argument fields due to this Rust
// bug:
// https://github.com/rust-lang/rust/issues/35853
#[macro_export]
#[doc(hidden)]
macro_rules! common_methods {
    () => {
        fn in_sequence(&mut self, seq: &mut $crate::Sequence)
            -> &mut Self
        {
            assert!(self.times.is_exact(),
                "Only Expectations with an exact call count have sequences");
            self.seq_handle = Some(seq.next());
            self
        }

        fn is_done(&self) -> bool {
            self.times.is_done()
        }

        /// Forbid this expectation from ever being called
        fn never(&mut self) {
            self.times.never();
        }

        fn satisfy_sequence(&self) {
            if let Some(handle) = &self.seq_handle {
                handle.satisfy()
            }
        }

        /// Require this expectation to be called exactly `n` times.
        fn times(&mut self, n: usize) {
            self.times.n(n);
        }

        /// Allow this expectation to be called any number of times
        fn times_any(&mut self) {
            self.times.any();
        }

        /// Allow this expectation to be called any number of times within a
        /// given range
        fn times_range(&mut self, range: Range<usize>) {
            self.times.range(range);
        }

        fn verify_sequence(&self) {
            if let Some(handle) = &self.seq_handle {
                handle.verify()
            }
        }
    }
}

/// Common methods of the Expectation structs
// Rust bug 35853 applies here, too.
#[macro_export]
#[doc(hidden)]
macro_rules! expectation_methods {
    () => {
        /// Add this expectation to a [`Sequence`](struct.Sequence.html).
        pub fn in_sequence(&mut self, seq: &mut $crate::Sequence) -> &mut Self {
            self.common.in_sequence(seq);
            self
        }

        pub fn is_done(&self) -> bool {
            self.common.is_done()
        }

        /// Forbid this expectation from ever being called
        pub fn never(&mut self) -> &mut Self {
            self.common.never();
            self
        }

        /// Expect this expectation to be called exactly once.  Shortcut for
        /// [`times(1)`](#method.times).
        pub fn once(&mut self) -> &mut Self {
            self.times(1)
        }

        /// Require this expectation to be called exactly `n` times.
        pub fn times(&mut self, n: usize) -> &mut Self {
            self.common.times(n);
            self
        }

        /// Allow this expectation to be called any number of times
        ///
        /// This behavior is the default, but the method is provided in case the
        /// default behavior changes.
        pub fn times_any(&mut self) -> &mut Self {
            self.common.times_any();
            self
        }

        /// Allow this expectation to be called any number of times within a
        /// given range
        pub fn times_range(&mut self, range: Range<usize>)
            -> &mut Self
        {
            self.common.times_range(range);
            self
        }
    }
}

/// Common methods of the Expectations, RefExpectations, and RefMutExpectations
/// structs
#[macro_export]
#[doc(hidden)]
macro_rules! expectations_methods {
    ($expectation:ty) => {
        /// Verify that all current expectations are satisfied and clear them.
        pub fn checkpoint(&mut self) {
            self.0.drain(..);
        }

        pub fn new() -> Self {
            Self::default()
        }
    }
}

/// Generate Expectation and Expectations types for a single method.
///
/// This macro can mock most method types, whether they take `self`, `&self`, or
/// `&mut self` references, `'static` arguments or reference arguments, and
/// `'static` or reference return types.
///
/// Reference arguments will be given independent anonymous lifetimes.  This is
/// the usual behavior for methods that don't specify explicit lifetimes.
///
/// Methods that return references will use `self`'s lifetime for the lifetime
/// of the returned value.
///
/// Generic methods are allowed, as long as the generic parameters are `'static`
/// (but concrete types don't need to be).  Don't repeat the generic bounds in
/// the macro invocation.
///
/// # Arguments
///
/// * `module`:         Name of the module to create
/// * `generics`:       Comma-delimited sequence of generic parameters, sans
///                     bounds.
/// * `o`:              Owned version of the output type.  Must be a `'static`.
///                     The real output type will be a reference to this one.
///                     Returning references, both mutable and immutable, is
///                     allowed, but note that the `&` or `&mut` is technically
///                     not part of the macro argument.
/// * `argty`:          Comma-delimited sequence of arguments types for the
///                     method being mocked.
/// * `matchcall`:      Comma-delimited sequence of expressions that produce
///                     values of type `matchty` from values of type `argty`.
/// * `args`:           comma-delimited sequence of argument names for each
///                     argument.  Ideally this wouldn't need to be specified,
///                     but it is due to Rust's macro hygiene rules.
/// * `altargs`:        Comma-delimited sequence of identifiers of the same
///                     length as `args`, but distinct.
/// * `matchty`:        comma-delimited sequence of types for each match
///                     argument.  Must all be `'static`.
///
/// # TODO: document generated methods
///
/// # Examples
///
/// Mock a method with a `'static` return type like
/// `foo(&self, x: u32, y: &i16) -> u32`
/// ```no_run
/// # use mockall::*;
/// expectation! {
///     fn foo<>(&self, x: u32, y: &i16) -> u32 {
///         let (px: &u32, py: &i16) = (&x, y);
///     }
/// }
/// ```
///
/// Mock a generic method with a `'static` return type like
/// `foo<D: Clone>(d: D, x: &u32) -> bool`
/// ```no_run
/// # use mockall::*;
/// expectation! {
///     fn foo<D>(&self, d: D, x: &u32) -> bool {
///         let (pd: &D, px: &u32) = (&d, x);
///     }
/// }
/// ```
///
/// Mock a method returning a reference like
/// `foo(&self, x: u32, y: &i16) -> &u32`
/// ```no_run
/// # use mockall::*;
/// expectation!{
///     fn foo<>(&self, i0: u32, i1: &i16) -> &u32 {
///         let (p0: &u32, p1: &i16) = (&i0, i1);
///     }
/// }
/// ```
///
/// Mock a method returning a mutable reference like
/// `foo(&mut self, x: u32, y: &i16) -> &mut u32`
/// ```no_run
/// # use mockall::*;
/// expectation!{
///     fn foo<>(&mut self, i0: u32, i1: &i16) -> &mut u32 {
///         let (p0: &u32, p1: &i16) = (&i0, i1);
///     }
/// }
/// ```
#[macro_export]
macro_rules! expectation {
    (
        // First pattern, for references taking &self and returning immutable
        // references.
        fn $module:ident
        < $( $generics:ident ),* >
        (&self, $( $args:ident : $argty:ty ),* ) -> & $o:ty
        {
            let ( $( $altargs:ident : &$matchty:ty ),* ) =
                ( $( $matchcall:expr ),* );
        }
    ) => {
        mod $module {
        use ::downcast::*;
        use ::fragile::Fragile;
        use ::predicates_tree::CaseTreeExt;
        use ::std::{
            collections::hash_map::HashMap,
            marker::PhantomData,
            mem,
            ops::{DerefMut, Range},
            sync::Mutex
        };
        use super::*;

        enum Matcher<$($generics: 'static,)*> {
            Func(Box<Fn($( &$matchty, )* ) -> bool + Send>),
            Pred( $( Box<$crate::Predicate<$matchty> + Send>, )* ),
            // Prevent "unused type parameter" errors
            _Phantom(PhantomData<($($generics,)*)>)
        }

        impl<$($generics: 'static,)*> Matcher<$($generics,)*> {
            fn matches(&self, $( $args: &$matchty, )*) -> bool {
                match self {
                    Matcher::Func(f) => f($( $args, )*),
                    Matcher::Pred($( $altargs, )*) =>
                        [$( $altargs.eval($args), )*]
                        .into_iter()
                        .all(|x| *x),
                    _ => unreachable!()
                }
            }

            fn verify(&self, $( $args: &$matchty, )* ) {
                match self {
                    Matcher::Func(f) => assert!(f($( $args, )*),
                        "Expectation didn't match arguments"),
                    Matcher::Pred($( $altargs, )*) => {
                        $(if let Some(c) = $altargs.find_case(false, $args) {
                            panic!("Expectation didn't match arguments:\n{}",
                                   c.tree());
                        })*
                    },
                    _ => unreachable!()
                }
            }
        }

        impl<$($generics: 'static,)*> Default for Matcher<$($generics,)*> {
            #[allow(unused_variables)]
            fn default() -> Self {
                Matcher::Func(Box::new(|$( $args, )*| true))
            }
        }

        /// Holds the stuff that is independent of the output type
        struct Common<$($generics: 'static,)*> {
            matcher: Mutex<Matcher<$($generics,)*>>,
            seq_handle: Option<$crate::SeqHandle>,
            times: $crate::Times
        }

        impl<$($generics: 'static,)*> std::default::Default for Common<$($generics,)*>
        {
            fn default() -> Self {
                Common {
                    matcher: Mutex::new(Matcher::default()),
                    seq_handle: None,
                    times: $crate::Times::default()
                }
            }
        }

        impl<$($generics: 'static,)*> Common<$($generics,)*> {
            fn call(&self, $( $args: &$matchty, )* ) {
                self.matcher.lock().unwrap().verify($( $args, )*);
                self.times.call();
                self.verify_sequence();
                if self.times.is_satisfied() {
                    self.satisfy_sequence()
                }
            }

            fn matches(&self, $( $args: &$matchty, )*) -> bool {
                self.matcher.lock().unwrap().matches($( $args, )*)
            }

            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*)
            {
                let mut guard = self.matcher.lock().unwrap();
                let m = Matcher::Pred($( Box::new($args), )*);
                mem::replace(guard.deref_mut(), m);
            }

            fn withf<F>(&mut self, f: F)
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                let mut guard = self.matcher.lock().unwrap();
                let m = Matcher::Func(Box::new(f));
                mem::replace(guard.deref_mut(), m);
            }

            $crate::common_methods!{}
        }

        pub struct Expectation<$($generics: 'static,)*> {
            common: Common<$($generics,)*>,
            result: Option<$o>,
        }

        impl<$($generics: 'static,)*> Expectation<$($generics,)*> {
            pub fn call(&self, $( $args: $argty, )* ) -> &$o {
                self.common.call($( $matchcall, )*);
                &self.result.as_ref()
                    .expect("Must set return value with return_const")
            }

            /// Validate this expectation's matcher.
            pub fn matches(&self, $( $args: &$matchty, )*) -> bool {
                self.common.matches($( $args, )*)
            }

            pub fn new() -> Self {
                Self::default()
            }

            /// Return a reference to a constant value from the `Expectation`
            pub fn return_const(&mut self, o: $o) -> &mut Self {
                self.result = Some(o);
                self
            }

            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            pub fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*) -> &mut Self
            {
                self.common.with($( $args, )*);
                self
            }

            pub fn withf<F>(&mut self, f: F) -> &mut Self
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                self.common.withf(f);
                self
            }

            $crate::expectation_methods!{}
        }

        impl<$($generics: 'static,)*> Default for Expectation<$($generics,)*>
        {
            fn default() -> Self {
                Expectation {
                    common: Common::default(),
                    result: None
                }
            }
        }

        pub struct Expectations<$($generics: 'static,)*>(
            Vec<Expectation<$($generics,)*>>
        );
        impl<$($generics: 'static,)*> Expectations<$($generics,)*> {
            /// Simulating calling the real method.  Every current expectation
            /// will be checked in FIFO order and the first one with matching
            /// arguments will be used.
            pub fn call(&self, $( $args: $argty, )* ) -> &$o {
                let n = self.0.len();
                match self.0.iter()
                    .find(|e| e.matches($( $matchcall, )*) &&
                          (!e.is_done() || n == 1))
                {
                    None => panic!("No matching expectation found"),
                    Some(e) => e.call($( $args, )*)
                }
            }

            /// Create a new expectation for this method.
            pub fn expect(&mut self) -> &mut Expectation<$($generics,)*>
            {
                let e = Expectation::<$($generics,)*>::default();
                self.0.push(e);
                let l = self.0.len();
                &mut self.0[l - 1]
            }

            $crate::expectations_methods!{Expectation}
        }
        impl<$($generics: 'static,)*> Default for Expectations<$($generics,)*>
        {
            fn default() -> Self {
                Expectations(Vec::new())
            }
        }
        impl<$($generics: Send + Sync + 'static,)*>
            $crate::AnyExpectations for Expectations<$($generics,)*>
        {}

        #[derive(Default)]
        pub struct GenericExpectations{
            store: ::std::collections::hash_map::HashMap<$crate::Key,
                Box<dyn $crate::AnyExpectations>>
        }
        impl GenericExpectations {
            /// Simulating calling the real method.
            pub fn call<$($generics: Send + Sync + 'static,)*>
                (&self, $( $args: $argty, )* ) -> &$o
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let e: &Expectations<$($generics,)*> = self.store.get(&key)
                    .expect("No matching expectation found")
                    .downcast_ref()
                    .unwrap();
                e.call($( $args, )*)
            }

            /// Verify that all current expectations are satisfied and clear
            /// them.  This applies to all sets of generic parameters!
            pub fn checkpoint(&mut self) {
                self.store.clear();
            }

            /// Create a new Expectation.
            pub fn expect<'e, $($generics: Send + Sync + 'static,)*>
                (&'e mut self)
                -> &'e mut Expectation<$($generics,)*>
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.store.entry(key)
                    .or_insert_with(||
                        Box::new(Expectations::<$($generics,)*>::new())
                    ).downcast_mut()
                    .unwrap();
                ee.expect()
            }

            pub fn new() -> Self {
                Self::default()
            }
        }
        }
    };

    (
        // Second pattern, for methods taking &mut self and returning mutable or
        // immutable references.
        fn $module:ident
        < $( $generics:ident ),* >
        (&mut self, $( $args:ident : $argty:ty ),* ) -> & $(mut)? $o:ty
        {
            let ( $( $altargs:ident : &$matchty:ty ),* ) =
                ( $( $matchcall:expr ),* );
        }
    ) => {
        mod $module {
        use ::downcast::*;
        use ::predicates_tree::CaseTreeExt;
        use ::fragile::Fragile;
        use ::std::{
            collections::hash_map::HashMap,
            marker::PhantomData,
            mem,
            ops::{DerefMut, Range},
            sync::{Mutex, MutexGuard}
        };
        use super::*;

        enum Matcher<$($generics: 'static,)*> {
            Func(Box<Fn($( &$matchty, )* ) -> bool + Send>),
            Pred( $( Box<$crate::Predicate<$matchty> + Send>, )* ),
            // Prevent "unused type parameter" errors
            _Phantom(PhantomData<($($generics,)*)>)
        }

        impl<$($generics: 'static,)*> Matcher<$($generics,)*> {
            fn matches(&self, $( $args: &$matchty, )*) -> bool {
                match self {
                    Matcher::Func(f) => f($( $args, )*),
                    Matcher::Pred($( $altargs, )*) =>
                        [$( $altargs.eval($args), )*]
                        .into_iter()
                        .all(|x| *x),
                    _ => unreachable!()
                }
            }

            fn verify(&self, $( $args: &$matchty, )* ) {
                match self {
                    Matcher::Func(f) => assert!(f($( $args, )*),
                        "Expectation didn't match arguments"),
                    Matcher::Pred($( $altargs, )*) => {
                        $(if let Some(c) = $altargs.find_case(false, $args) {
                            panic!("Expectation didn't match arguments:\n{}",
                                   c.tree());
                        })*
                    },
                    _ => unreachable!()
                }
            }
        }

        impl<$($generics: 'static,)*> Default for Matcher<$($generics,)*> {
            #[allow(unused_variables)]
            fn default() -> Self {
                Matcher::Func(Box::new(|$( $args, )*| true))
            }
        }

        /// Holds the stuff that is independent of the output type
        struct Common<$($generics: 'static,)*> {
            matcher: Mutex<Matcher<$($generics,)*>>,
            seq_handle: Option<$crate::SeqHandle>,
            times: $crate::Times
        }

        impl<$($generics: 'static,)*> Common<$($generics,)*> {
            fn call(&self, $( $args: &$matchty, )* ) {
                self.matcher.lock().unwrap().verify($( $args, )*);
                self.times.call();
                self.verify_sequence();
                if self.times.is_satisfied() {
                    self.satisfy_sequence()
                }
            }

            fn matches(&self, $( $args: &$matchty, )*) -> bool {
                self.matcher.lock().unwrap().matches($( $args, )*)
            }

            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*)
            {
                let mut guard = self.matcher.lock().unwrap();
                let m = Matcher::Pred($( Box::new($args), )*);
                mem::replace(guard.deref_mut(), m);
            }

            fn withf<F>(&mut self, f: F)
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                let mut guard = self.matcher.lock().unwrap();
                let m = Matcher::Func(Box::new(f));
                mem::replace(guard.deref_mut(), m);
            }

            $crate::common_methods!{}
        }
        impl<$($generics: 'static,)*> std::default::Default for Common<$($generics,)*>
        {
            fn default() -> Self {
                Common {
                    matcher: Mutex::new(Matcher::default()),
                    seq_handle: None,
                    times: $crate::Times::default()
                }
            }
        }

        pub struct Expectation<$($generics: 'static,)*> {
            common: Common<$($generics,)*>,
            result: Option<$o>,
            rfunc: Option<Box<dyn FnMut($( $argty, )*) -> $o + Send + Sync>>,
        }

        impl<$($generics: 'static,)*> Expectation<$($generics,)*> {
            /// Simulating calling the real method for this expectation
            pub fn call_mut(&mut self, $( $args: $argty, )*) -> &mut $o {
                self.common.call($( $matchcall, )*);
                if let Some(ref mut f) = self.rfunc {
                    self.result = Some(f($( $args, )*));
                }
                self.result.as_mut()
                    .expect("Must first set return function with returning or return_var")
            }

            /// Validate this expectation's matcher.
            pub fn matches(&self, $( $args: &$matchty, )*) -> bool {
                self.common.matches($( $args, )*)
            }

            pub fn new() -> Self {
                Self::default()
            }

            /// Convenience method that can be used to supply a return value for
            /// a `Expectation`.  The value will be returned by mutable
            /// reference.
            pub fn return_var(&mut self, o: $o) -> &mut Self
            {
                self.result = Some(o);
                self
            }

            /// Supply a closure that the `Expectation` will use to create its
            /// return value.  The return value will be returned by mutable
            /// reference.
            pub fn returning<F>(&mut self, f: F) -> &mut Self
                where F: FnMut($( $argty, )*) -> $o + Send + Sync + 'static
            {
                mem::replace(&mut self.rfunc, Some(Box::new(f)));
                self
            }

            /// Single-threaded version of [`returning`](#method.returning).
            /// Can be used when the argument or return type isn't `Send`.
            pub fn returning_st<F>(&mut self, f: F) -> &mut Self
                where F: FnMut($( $argty, )*) -> $o + 'static
            {
                let mut fragile = Fragile::new(f);
                let fmut = move |$( $args: $argty, )*| {
                    (fragile.get_mut())($( $args, )*)
                };
                mem::replace(&mut self.rfunc, Some(Box::new(fmut)));
                self
            }

            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            pub fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*) -> &mut Self
            {
                self.common.with($( $args, )*);
                self
            }

            pub fn withf<F>(&mut self, f: F) -> &mut Self
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                self.common.withf(f);
                self
            }

            $crate::expectation_methods!{}
        }
        impl<$($generics: 'static,)*> Default for Expectation<$($generics,)*>
        {
            fn default() -> Self {
                Expectation {
                    common: Common::default(),
                    result: None,
                    rfunc: None
                }
            }
        }
        pub struct Expectations<$($generics: 'static,)*>(
            Vec<Expectation<$($generics,)*>>
        );
        impl<$($generics: 'static,)*> Expectations<$($generics,)*> {
            /// Simulating calling the real method.  Every current expectation
            /// will be checked in FIFO order and the first one with matching
            /// arguments will be used.
            pub fn call_mut(&mut self, $( $args: $argty, )* ) -> &mut $o {
                let n = self.0.len();
                match self.0.iter_mut()
                    .find(|e| e.matches($( $matchcall, )*) &&
                          (!e.is_done() || n == 1))
                {
                    None => panic!("No matching expectation found"),
                    Some(e) => e.call_mut($( $args, )*)
                }
            }

            /// Create a new expectation for this method.
            pub fn expect(&mut self) -> &mut Expectation<$($generics,)*>
            {
                let e = Expectation::default();
                self.0.push(e);
                let l = self.0.len();
                &mut self.0[l - 1]
            }

            $crate::expectations_methods!{Expectation}
        }
        impl<$($generics: 'static,)*> Default for Expectations<$($generics,)*>
        {
            fn default() -> Self {
                Expectations(Vec::new())
            }
        }
        impl<$($generics: Send + Sync + 'static,)*>
            $crate::AnyExpectations for Expectations<$($generics,)*>
        {}

        #[derive(Default)]
        pub struct GenericExpectations{
            store: ::std::collections::hash_map::HashMap<$crate::Key,
                Box<dyn $crate::AnyExpectations>>
        }
        impl GenericExpectations {
            /// Simulating calling the real method.
            pub fn call_mut<$($generics: Send + Sync + 'static,)*>
                (&mut self, $( $args: $argty, )* ) -> &mut $o
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let e: &mut Expectations<$($generics,)*> = self.store
                    .get_mut(&key)
                    .expect("No matching expectation found")
                    .downcast_mut()
                    .unwrap();
                e.call_mut($( $args, )*)
            }

            /// Verify that all current expectations are satisfied and clear
            /// them.  This applies to all sets of generic parameters!
            pub fn checkpoint(&mut self) {
                self.store.clear();
            }

            /// Create a new Expectation.
            pub fn expect<'e, $($generics: Send + Sync + 'static,)*>
                (&'e mut self)
                -> &'e mut Expectation<$($generics,)*>
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.store.entry(key)
                    .or_insert_with(||
                        Box::new(Expectations::<$($generics,)*>::new())
                    ).downcast_mut()
                    .unwrap();
                ee.expect()
            }

            pub fn new() -> Self {
                Self::default()
            }
        }
        }
    };

    (
        // Third pattern, for methods returning 'static values
        fn $module:ident
        < $( $generics:ident ),* >
        ($(&)?$(mut)?self, $( $args:ident : $argty:ty ),* ) -> $o:ty
        {
            let ( $( $altargs:ident : &$matchty:ty ),* ) =
                ( $( $matchcall:expr ),* );
        }
    ) => {
        mod $module {
        use ::downcast::*;
        use ::fragile::Fragile;
        use ::predicates_tree::CaseTreeExt;
        use ::std::{
            collections::hash_map::HashMap,
            marker::PhantomData,
            mem,
            ops::{DerefMut, Range},
            sync::{Mutex, MutexGuard}
        };
        use super::*;   // Import types from the calling environment

        enum Rfunc<$($generics: 'static,)*> {
            Default,
            // Indicates that a `return_once` expectation has already returned
            Expired,
            Mut(Box<dyn FnMut($( $argty, )*) -> $o + Send>),
            // Should be Box<dyn FnOnce> once that feature is stabilized
            // https://github.com/rust-lang/rust/issues/28796
            Once(Box<dyn FnMut($( $argty, )*) -> $o + Send>),
        }

        impl<$($generics,)*>  Rfunc<$($generics,)*> {
            fn call_mut(&mut self, $( $args: $argty, )* ) -> $o {
                match self {
                    Rfunc::Default => {
                        use $crate::ReturnDefault;
                        $crate::DefaultReturner::<$o>::return_default()
                    },
                    Rfunc::Expired => {
                        panic!("Called a method twice that was expected only once")
                    },
                    Rfunc::Mut(f) => {
                        f( $( $args, )* )
                    },
                    Rfunc::Once(_) => {
                        let fo = mem::replace(self, Rfunc::Expired);
                        if let Rfunc::Once(mut f) = fo {
                            f( $( $args, )* )
                        } else {
                            unreachable!()
                        }
                    },
                }
            }
        }

        impl<$($generics,)*>
            std::default::Default for Rfunc<$($generics,)*>
        {
            fn default() -> Self {
                Rfunc::Default
            }
        }

        enum Matcher<$($generics: 'static,)*> {
            Func(Box<Fn($( &$matchty, )* ) -> bool + Send>),
            Pred( $( Box<$crate::Predicate<$matchty> + Send>, )* ),
            // Prevent "unused type parameter" errors
            _Phantom(PhantomData<($($generics,)*)>)
        }

        impl<$($generics,)*> Matcher<$($generics,)*> {
            fn matches(&self, $( $args: &$matchty, )*) -> bool {
                match self {
                    Matcher::Func(f) => f($( $args, )*),
                    Matcher::Pred($( $altargs, )*) =>
                        [$( $altargs.eval($args), )*]
                        .into_iter()
                        .all(|x| *x),
                    _ => unreachable!()
                }
            }

            fn verify(&self, $( $args: &$matchty, )* ) {
                match self {
                    Matcher::Func(f) => assert!(f($( $args, )*),
                        "Expectation didn't match arguments"),
                    Matcher::Pred($( $altargs, )*) => {
                        $(if let Some(c) = $altargs.find_case(false, $args) {
                            panic!("Expectation didn't match arguments:\n{}",
                                   c.tree());
                        })*
                    },
                    _ => unreachable!()
                }
            }
        }

        impl<$($generics,)*> Default for Matcher<$($generics,)*> {
            #[allow(unused_variables)]
            fn default() -> Self {
                Matcher::Func(Box::new(|$( $args, )*| true))
            }
        }

        /// Holds the stuff that is independent of the output type
        struct Common<$($generics: 'static,)*> {
            matcher: Mutex<Matcher<$($generics,)*>>,
            seq_handle: Option<$crate::SeqHandle>,
            times: $crate::Times
        }

        impl<$($generics,)*> Common<$($generics,)*> {
            fn call(&self, $( $args: &$matchty, )* ) {
                self.matcher.lock().unwrap().verify($( $args, )*);
                self.times.call();
                self.verify_sequence();
                if self.times.is_satisfied() {
                    self.satisfy_sequence()
                }
            }

            fn matches(&self, $( $args: &$matchty, )*) -> bool {
                self.matcher.lock().unwrap().matches($( $args, )*)
            }

            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*)
            {
                let mut guard = self.matcher.lock().unwrap();
                let m = Matcher::Pred($( Box::new($args), )*);
                mem::replace(guard.deref_mut(), m);
            }

            fn withf<F>(&mut self, f: F)
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                let mut guard = self.matcher.lock().unwrap();
                let m = Matcher::Func(Box::new(f));
                mem::replace(guard.deref_mut(), m);
            }

            $crate::common_methods!{}
        }

        impl<$($generics,)*> std::default::Default for Common<$($generics,)*>
        {
            fn default() -> Self {
                Common {
                    matcher: Mutex::new(Matcher::default()),
                    seq_handle: None,
                    times: $crate::Times::default()
                }
            }
        }

        pub struct Expectation<$($generics: 'static,)*> {
            common: Common<$($generics,)*>,
            rfunc: Mutex<Rfunc<$($generics,)*>>,
        }

        impl<$($generics,)*> Expectation<$($generics,)*> {
            pub fn call(&self, $( $args: $argty, )* ) -> $o
            {
                self.common.call($( $matchcall, )*);
                self.rfunc.lock().unwrap().call_mut($( $args, )*)
            }

            /// Validate this expectation's matcher.
            pub fn matches(&self, $( $args: &$matchty, )*) -> bool {
                self.common.matches($( $args, )*)
            }

            /// Return a constant value from the `Expectation`
            ///
            /// The output type must be `Clone`.  The compiler can't always
            /// infer the proper type to use with this method; you will usually
            /// need to specify it explicitly.  i.e. `return_const(42u32)`
            /// instead of `return_const(42)`.
            // We must use Into<$o> instead of $o because where clauses don't
            // accept equality constraints.
            // https://github.com/rust-lang/rust/issues/20041
            #[allow(unused_variables)]
            pub fn return_const<MockallOutput>(&mut self, c: MockallOutput) -> &mut Self
                where MockallOutput: Clone + Into<$o> + Send + 'static
            {
                let f = move |$( $args: $argty, )*| c.clone().into();
                self.returning(f)
            }

            /// Supply an `FnOnce` closure that will provide the return value
            /// for this Expectation.  This is useful for return types that
            /// aren't `Clone`.  It will be an error to call this Expectation
            /// multiple times.
            pub fn return_once<F>(&mut self, f: F) -> &mut Self
                where F: FnOnce($( $argty, )*) -> $o + Send + 'static
            {
                let mut fopt = Some(f);
                let fmut = move |$( $args: $argty, )*| {
                    if let Some(f) = fopt.take() {
                        f($( $args, )*)
                    } else {
                        panic!("Called a method twice that was expected only once")
                    }
                };
                {
                    let mut guard = self.rfunc.lock().unwrap();
                    mem::replace(guard.deref_mut(),
                                 Rfunc::Once(Box::new(fmut)));
                }
                self
            }

            /// Single-threaded version of [`return_once`](#method.return_once).
            /// This is useful for return types that are neither `Send` nor
            /// `Clone`.
            ///
            /// It is a runtime error to call the mock method from a different
            /// thread than the one that originally called this method.  It is
            /// also a runtime error to call the method more than once.
            pub fn return_once_st<F>(&mut self, f: F) -> &mut Self
                where F: FnOnce($( $argty, )*) -> $o + 'static
            {
                let mut fragile = Some(::fragile::Fragile::new(f));
                let fmut = Box::new(move |$( $args: $argty, )*| {
                    match fragile.take() {
                        Some(frag) => (frag.into_inner())($( $args, )*),
                        None => panic!(
                            "Called a method twice that was expected only once")
                    }
                });
                {
                    let mut guard = self.rfunc.lock().unwrap();
                    mem::replace(guard.deref_mut(), Rfunc::Once(fmut));
                }
                self
            }

            pub fn returning<F>(&mut self, f: F) -> &mut Self
                where F: FnMut($( $argty, )*) -> $o + Send + 'static
            {
                {
                    let mut guard = self.rfunc.lock().unwrap();
                    mem::replace(guard.deref_mut(), Rfunc::Mut(Box::new(f)));
                }
                self
            }

            /// Single-threaded version of [`returning`](#method.returning).
            /// Can be used when the argument or return type isn't `Send`.
            ///
            /// It is a runtime error to call the mock method from a different
            /// thread than the one that originally called this method.
            pub fn returning_st<F>(&mut self, f: F) -> &mut Self
                where F: FnMut($( $argty, )*) -> $o + 'static
            {
                let mut fragile = Fragile::new(f);
                let fmut = move |$( $args: $argty, )*| {
                    (fragile.get_mut())($( $args, )*)
                };
                {
                    let mut guard = self.rfunc.lock().unwrap();
                    mem::replace(guard.deref_mut(), Rfunc::Mut(Box::new(fmut)));
                }
                self
            }

            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            pub fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*) -> &mut Self
            {
                self.common.with($( $args, )*);
                self
            }

            pub fn withf<F>(&mut self, f: F) -> &mut Self
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                self.common.withf(f);
                self
            }

            $crate::expectation_methods!{}
        }
        impl<$($generics,)*> Default for Expectation<$($generics,)*>
        {
            fn default() -> Self {
                Expectation {
                    common: Common::default(),
                    rfunc: Mutex::new(Rfunc::default())
                }
            }
        }

        pub struct Expectations<$($generics: 'static,)*>(
            Vec<Expectation<$($generics,)*>>
        );
        impl<$($generics,)*> Expectations<$($generics,)*> {
            /// Simulating calling the real method.  Every current expectation
            /// will be checked in FIFO order and the first one with matching
            /// arguments will be used.
            pub fn call(&self, $( $args: $argty, )* ) -> $o
            {
                let n = self.0.len();
                match self.0.iter()
                    .find(|e| e.matches($( $matchcall, )*) &&
                          (!e.is_done() || n == 1))
                {
                    None => panic!("No matching expectation found"),
                    Some(e) => e.call($( $args, )*)
                }
            }

            /// Create a new expectation for this method.
            pub fn expect(&mut self) -> &mut Expectation<$($generics,)*>
            {
                let e = Expectation::<$($generics,)*>::default();
                self.0.push(e);
                let l = self.0.len();
                &mut self.0[l - 1]
            }

            $crate::expectations_methods!{Expectation}
        }
        impl<$($generics,)*> Default for Expectations<$($generics,)*>
        {
            fn default() -> Self {
                Expectations(Vec::new())
            }
        }
        impl<$($generics: Send + Sync + 'static,)*>
            $crate::AnyExpectations for Expectations<$($generics,)*>
        {}

        #[derive(Default)]
        pub struct GenericExpectations{
            store: HashMap<$crate::Key, Box<dyn $crate::AnyExpectations>>
        }
        impl GenericExpectations {
            /// Simulating calling the real method.
            pub fn call<$($generics: Send + Sync + 'static,)*>
                (&self, $( $args: $argty, )* ) -> $o
            {
                // TODO: remove the 2nd type argument from Key after the old API
                // is fully replaced.
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let e: &Expectations<$($generics,)*> = self.store.get(&key)
                    .expect("No matching expectation found")
                    .downcast_ref()
                    .unwrap();
                e.call($( $args, )*)
            }

            /// Verify that all current expectations are satisfied and clear
            /// them.  This applies to all sets of generic parameters!
            pub fn checkpoint(&mut self) {
                self.store.clear();
            }

            /// Create a new Expectation.
            pub fn expect<'e, $($generics: Send + Sync + 'static,)*>
                (&'e mut self)
                -> &'e mut Expectation<$($generics,)*>
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.store.entry(key)
                    .or_insert_with(||
                        Box::new(Expectations::<$($generics,)*>::new())
                    ).downcast_mut()
                    .unwrap();
                ee.expect()
            }

            pub fn new() -> Self {
                Self::default()
            }
        }
        }
    };
    (
        // Fourth pattern, for static methods
        fn $module:ident
        < $( $generics:ident ),* >
        ($( $args:ident : $argty:ty ),* ) -> $o:ty
        {
            let ( $( $altargs:ident : &$matchty:ty ),* ) =
                ( $( $matchcall:expr ),* );
        }
    ) => {
        mod $module {
        use ::downcast::*;
        use ::fragile::Fragile;
        use ::predicates_tree::CaseTreeExt;
        use ::std::{
            collections::hash_map::HashMap,
            marker::PhantomData,
            mem,
            ops::{DerefMut, Range},
            sync::{Mutex, MutexGuard}
        };
        use super::*;   // Import types from the calling environment

        enum Rfunc<$($generics: 'static,)*> {
            Default,
            // Indicates that a `return_once` expectation has already returned
            Expired,
            Mut(Box<dyn FnMut($( $argty, )*) -> $o + Send>),
            // Should be Box<dyn FnOnce> once that feature is stabilized
            // https://github.com/rust-lang/rust/issues/28796
            Once(Box<dyn FnMut($( $argty, )*) -> $o + Send>),
        }

        impl<$($generics,)*>  Rfunc<$($generics,)*> {
            fn call_mut(&mut self, $( $args: $argty, )* ) -> $o {
                match self {
                    Rfunc::Default => {
                        use $crate::ReturnDefault;
                        $crate::DefaultReturner::<$o>::return_default()
                    },
                    Rfunc::Expired => {
                        panic!("Called a method twice that was expected only once")
                    },
                    Rfunc::Mut(f) => {
                        f( $( $args, )* )
                    },
                    Rfunc::Once(_) => {
                        let fo = mem::replace(self, Rfunc::Expired);
                        if let Rfunc::Once(mut f) = fo {
                            f( $( $args, )* )
                        } else {
                            unreachable!()
                        }
                    },
                }
            }
        }

        impl<$($generics,)*>
            std::default::Default for Rfunc<$($generics,)*>
        {
            fn default() -> Self {
                Rfunc::Default
            }
        }

        enum Matcher<$($generics: 'static,)*> {
            Func(Box<Fn($( &$matchty, )* ) -> bool + Send>),
            Pred( $( Box<$crate::Predicate<$matchty> + Send>, )* ),
            // Prevent "unused type parameter" errors
            _Phantom(PhantomData<($($generics,)*)>)
        }

        impl<$($generics,)*> Matcher<$($generics,)*> {
            fn matches(&self, $( $args: &$matchty, )*) -> bool {
                match self {
                    Matcher::Func(f) => f($( $args, )*),
                    Matcher::Pred($( $altargs, )*) =>
                        [$( $altargs.eval($args), )*]
                        .into_iter()
                        .all(|x| *x),
                    _ => unreachable!()
                }
            }

            fn verify(&self, $( $args: &$matchty, )* ) {
                match self {
                    Matcher::Func(f) => assert!(f($( $args, )*),
                        "Expectation didn't match arguments"),
                    Matcher::Pred($( $altargs, )*) => {
                        $(if let Some(c) = $altargs.find_case(false, $args) {
                            panic!("Expectation didn't match arguments:\n{}",
                                   c.tree());
                        })*
                    },
                    _ => unreachable!()
                }
            }
        }

        impl<$($generics,)*> Default for Matcher<$($generics,)*> {
            #[allow(unused_variables)]
            fn default() -> Self {
                Matcher::Func(Box::new(|$( $args, )*| true))
            }
        }

        /// Holds the stuff that is independent of the output type
        struct Common<$($generics: 'static,)*> {
            matcher: Mutex<Matcher<$($generics,)*>>,
            seq_handle: Option<$crate::SeqHandle>,
            times: $crate::Times
        }

        impl<$($generics,)*> Common<$($generics,)*> {
            fn call(&self, $( $args: &$matchty, )* ) {
                self.matcher.lock().unwrap().verify($( $args, )*);
                self.times.call();
                self.verify_sequence();
                if self.times.is_satisfied() {
                    self.satisfy_sequence()
                }
            }

            fn matches(&self, $( $args: &$matchty, )*) -> bool {
                self.matcher.lock().unwrap().matches($( $args, )*)
            }

            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*)
            {
                let mut guard = self.matcher.lock().unwrap();
                let m = Matcher::Pred($( Box::new($args), )*);
                mem::replace(guard.deref_mut(), m);
            }

            fn withf<F>(&mut self, f: F)
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                let mut guard = self.matcher.lock().unwrap();
                let m = Matcher::Func(Box::new(f));
                mem::replace(guard.deref_mut(), m);
            }

            $crate::common_methods!{}
        }

        impl<$($generics,)*> std::default::Default for Common<$($generics,)*>
        {
            fn default() -> Self {
                Common {
                    matcher: Mutex::new(Matcher::default()),
                    seq_handle: None,
                    times: $crate::Times::default()
                }
            }
        }

        pub struct Expectation<$($generics: 'static,)*> {
            common: Common<$($generics,)*>,
            rfunc: Mutex<Rfunc<$($generics,)*>>,
        }

        impl<$($generics,)*> Expectation<$($generics,)*> {
            pub fn call(&self, $( $args: $argty, )* ) -> $o
            {
                self.common.call($( $matchcall, )*);
                self.rfunc.lock().unwrap().call_mut($( $args, )*)
            }

            /// Validate this expectation's matcher.
            pub fn matches(&self, $( $args: &$matchty, )*) -> bool {
                self.common.matches($( $args, )*)
            }

            /// Return a constant value from the `Expectation`
            ///
            /// The output type must be `Clone`.  The compiler can't always
            /// infer the proper type to use with this method; you will usually
            /// need to specify it explicitly.  i.e. `return_const(42u32)`
            /// instead of `return_const(42)`.
            // We must use Into<$o> instead of $o because where clauses don't
            // accept equality constraints.
            // https://github.com/rust-lang/rust/issues/20041
            #[allow(unused_variables)]
            pub fn return_const<MockallOutput>(&mut self, c: MockallOutput) -> &mut Self
                where MockallOutput: Clone + Into<$o> + Send + 'static
            {
                let f = move |$( $args: $argty, )*| c.clone().into();
                self.returning(f)
            }

            /// Supply an `FnOnce` closure that will provide the return value
            /// for this Expectation.  This is useful for return types that
            /// aren't `Clone`.  It will be an error to call this Expectation
            /// multiple times.
            pub fn return_once<F>(&mut self, f: F) -> &mut Self
                where F: FnOnce($( $argty, )*) -> $o + Send + 'static
            {
                let mut fopt = Some(f);
                let fmut = move |$( $args: $argty, )*| {
                    if let Some(f) = fopt.take() {
                        f($( $args, )*)
                    } else {
                        panic!("Called a method twice that was expected only once")
                    }
                };
                {
                    let mut guard = self.rfunc.lock().unwrap();
                    mem::replace(guard.deref_mut(),
                                 Rfunc::Once(Box::new(fmut)));
                }
                self
            }

            /// Single-threaded version of [`return_once`](#method.return_once).
            /// This is useful for return types that are neither `Send` nor
            /// `Clone`.
            ///
            /// It is a runtime error to call the mock method from a different
            /// thread than the one that originally called this method.  It is
            /// also a runtime error to call the method more than once.
            pub fn return_once_st<F>(&mut self, f: F) -> &mut Self
                where F: FnOnce($( $argty, )*) -> $o + 'static
            {
                let mut fragile = Some(::fragile::Fragile::new(f));
                let fmut = Box::new(move |$( $args: $argty, )*| {
                    match fragile.take() {
                        Some(frag) => (frag.into_inner())($( $args, )*),
                        None => panic!(
                            "Called a method twice that was expected only once")
                    }
                });
                {
                    let mut guard = self.rfunc.lock().unwrap();
                    mem::replace(guard.deref_mut(), Rfunc::Once(fmut));
                }
                self
            }

            pub fn returning<F>(&mut self, f: F) -> &mut Self
                where F: FnMut($( $argty, )*) -> $o + Send + 'static
            {
                {
                    let mut guard = self.rfunc.lock().unwrap();
                    mem::replace(guard.deref_mut(), Rfunc::Mut(Box::new(f)));
                }
                self
            }

            /// Single-threaded version of [`returning`](#method.returning).
            /// Can be used when the argument or return type isn't `Send`.
            ///
            /// It is a runtime error to call the mock method from a different
            /// thread than the one that originally called this method.
            pub fn returning_st<F>(&mut self, f: F) -> &mut Self
                where F: FnMut($( $argty, )*) -> $o + 'static
            {
                let mut fragile = Fragile::new(f);
                let fmut = move |$( $args: $argty, )*| {
                    (fragile.get_mut())($( $args, )*)
                };
                {
                    let mut guard = self.rfunc.lock().unwrap();
                    mem::replace(guard.deref_mut(), Rfunc::Mut(Box::new(fmut)));
                }
                self
            }

            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            pub fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*) -> &mut Self
            {
                self.common.with($( $args, )*);
                self
            }

            pub fn withf<F>(&mut self, f: F) -> &mut Self
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                self.common.withf(f);
                self
            }

            $crate::expectation_methods!{}
        }
        impl<$($generics,)*> Default for Expectation<$($generics,)*>
        {
            fn default() -> Self {
                Expectation {
                    common: Common::default(),
                    rfunc: Mutex::new(Rfunc::default())
                }
            }
        }

        pub struct Expectations<$($generics: 'static,)*>(
            Vec<Expectation<$($generics,)*>>
        );
        impl<$($generics,)*> Expectations<$($generics,)*> {
            /// Simulating calling the real method.  Every current expectation
            /// will be checked in FIFO order and the first one with matching
            /// arguments will be used.
            pub fn call(&self, $( $args: $argty, )* ) -> $o
            {
                let n = self.0.len();
                match self.0.iter()
                    .find(|e| e.matches($( $matchcall, )*) &&
                          (!e.is_done() || n == 1))
                {
                    None => panic!("No matching expectation found"),
                    Some(e) => e.call($( $args, )*)
                }
            }

            /// Create a new expectation for this method.
            pub fn expect(&mut self) -> &mut Expectation<$($generics,)*>
            {
                let e = Expectation::<$($generics,)*>::default();
                self.0.push(e);
                let l = self.0.len();
                &mut self.0[l - 1]
            }

            $crate::expectations_methods!{Expectation}
        }
        impl<$($generics,)*> Default for Expectations<$($generics,)*>
        {
            fn default() -> Self {
                Expectations(Vec::new())
            }
        }
        impl<$($generics: Send + Sync + 'static,)*>
            $crate::AnyExpectations for Expectations<$($generics,)*>
        {}

        #[derive(Default)]
        pub struct GenericExpectations{
            store: HashMap<$crate::Key, Box<dyn $crate::AnyExpectations>>
        }
        impl GenericExpectations {
            /// Simulating calling the real method.
            pub fn call<$($generics: Send + Sync + 'static,)*>
                (&self, $( $args: $argty, )* ) -> $o
            {
                // TODO: remove the 2nd type argument from Key after the old API
                // is fully replaced.
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let e: &Expectations<$($generics,)*> = self.store.get(&key)
                    .expect("No matching expectation found")
                    .downcast_ref()
                    .unwrap();
                e.call($( $args, )*)
            }

            /// Verify that all current expectations are satisfied and clear
            /// them.  This applies to all sets of generic parameters!
            pub fn checkpoint(&mut self) {
                self.store.clear();
            }

            /// Create a new Expectation.
            pub fn expect<'e, $($generics: Send + Sync + 'static,)*>
                (&'e mut self)
                -> &'e mut Expectation<$($generics,)*>
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.store.entry(key)
                    .or_insert_with(||
                        Box::new(Expectations::<$($generics,)*>::new())
                    ).downcast_mut()
                    .unwrap();
                ee.expect()
            }

            pub fn new() -> Self {
                Self::default()
            }
        }

        /// Like an [`&Expectation`](struct.Expectation.html) but protected by a
        /// Mutex guard.  Useful for mocking static methods.  Forwards accesses
        /// to an `Expectation` object.
        // We must return the MutexGuard to the caller so he can configure the
        // expectation.  But we can't bundle both the guard and the &Expectation
        // into the same structure; the borrow checker won't let us.  Instead
        // we'll record the expectation's position within the Expectations
        // vector so we can proxy its methods.
        //
        // ExpectationGuard is only defined for expectations that return 'static
        // return types.
        pub struct ExpectationGuard<'guard, $($generics: 'static,)*>{
            guard: MutexGuard<'guard, Expectations<$($generics,)*>>,
            i: usize
        }

        impl<'guard, $($generics,)*> ExpectationGuard<'guard, $($generics,)*> {
            /// Just like
            /// [`Expectation::in_sequence`](struct.Expectation.html#method.in_sequence)
            pub fn in_sequence(&mut self, seq: &mut $crate::Sequence)
                -> &mut Expectation<$($generics,)*>
            {
                self.guard.0[self.i].in_sequence(seq)
            }

            /// Just like
            /// [`Expectation::never`](struct.Expectation.html#method.never)
            pub fn never(&mut self) -> &mut Expectation<$($generics,)*> {
                self.guard.0[self.i].never()
            }

            // Should only be called from the mockall_derive generated code
            #[doc(hidden)]
            pub fn new(mut guard: MutexGuard<'guard, Expectations<$($generics,)*>>)
                -> Self
            {
                guard.expect(); // Drop the &Expectation
                let i = guard.0.len() - 1;
                ExpectationGuard{guard, i}
            }

            /// Just like [`Expectation::once`](struct.Expectation.html#method.once)
            pub fn once(&mut self) -> &mut Expectation<$($generics,)*> {
                self.guard.0[self.i].once()
            }

            /// Just like
            /// [`Expectation::returning`](struct.Expectation.html#method.returning)
            pub fn returning<F>(&mut self, f: F)
                -> &mut Expectation<$($generics,)*>
                where F: FnMut($( $argty, )*) -> $o + Send + 'static
            {
                self.guard.0[self.i].returning(f)
            }

            /// Just like
            /// [`Expectation::return_once`](struct.Expectation.html#method.return_once)
            pub fn return_once<F>(&mut self, f: F)
                -> &mut Expectation<$($generics,)*>
                where F: FnOnce($( $argty, )*) -> $o + Send + 'static
            {
                self.guard.0[self.i].return_once(f)
            }

            /// Just like
            /// [`Expectation::returning_st`](struct.Expectation.html#method.returning_st)
            pub fn returning_st<F>(&mut self, f: F)
                -> &mut Expectation<$($generics,)*>
                where F: FnMut($( $argty, )*) -> $o + 'static
            {
                self.guard.0[self.i].returning_st(f)
            }

            /// Just like
            /// [`Expectation::times`](struct.Expectation.html#method.times)
            pub fn times(&mut self, n: usize)
                -> &mut Expectation<$($generics,)*> {
                self.guard.0[self.i].times(n)
            }

            /// Just like
            /// [`Expectation::times_any`](struct.Expectation.html#method.times_any)
            pub fn times_any(&mut self) -> &mut Expectation<$($generics,)*> {
                self.guard.0[self.i].times_any()
            }

            /// Just like
            /// [`Expectation::times_range`](struct.Expectation.html#method.times_range)
            pub fn times_range(&mut self, range: Range<usize>)
                -> &mut Expectation<$($generics,)*>
            {
                self.guard.0[self.i].times_range(range)
            }

            /// Just like
            /// [`Expectation::with`](struct.Expectation.html#method.with)
            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            pub fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*)
                -> &mut Expectation<$($generics,)*>
            {
                self.guard.0[self.i].with($($args,)*)
            }

            /// Just like
            /// [`Expectation::withf`](struct.Expectation.html#method.withf)
            pub fn withf<F>(&mut self, f: F) -> &mut Expectation<$($generics,)*>
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                self.guard.0[self.i].withf(f)
            }
        }

        pub struct GenericExpectationGuard<'guard, $($generics: Sync + 'static,)*> {
            guard: MutexGuard<'guard, GenericExpectations>,
            i: usize,
            _phantom: PhantomData<((), $($generics,)*)>,
        }

        impl<'guard, $($generics: Send + Sync + 'static,)*>
            GenericExpectationGuard<'guard, $($generics,)*>
        {
            /// Just like
            /// [`Expectation::in_sequence`](struct.Expectation.html#method.in_sequence)
            pub fn in_sequence(&mut self, seq: &mut $crate::Sequence)
                -> &mut Expectation<$($generics,)*>
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].in_sequence(seq)
            }

            /// Just like
            /// [`Expectation::never`](struct.Expectation.html#method.never)
            pub fn never(&mut self) -> &mut Expectation<$($generics,)*> {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].never()
            }

            // Should only be called from the mockall_derive generated code
            #[doc(hidden)]
            pub fn new(mut guard: MutexGuard<'guard, GenericExpectations>)
                -> Self
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    guard.store.entry(key)
                    .or_insert_with(||
                        Box::new(Expectations::<$($generics,)*>::new()))
                    .downcast_mut()
                    .unwrap();
                ee.expect();    // Drop the &Expectation
                let i = ee.0.len() - 1;
                GenericExpectationGuard{guard, i, _phantom: PhantomData}
            }

            /// Just like
            /// [`Expectation::once`](struct.Expectation.html#method.once)
            pub fn once(&mut self) -> &mut Expectation<$($generics,)*> {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].once()
            }

            /// Just like
            /// [`Expectation::returning`](struct.Expectation.html#method.returning)
            pub fn returning<F>(&mut self, f: F) -> &mut Expectation<$($generics,)*>
                where F: FnMut($( $argty, )*) -> $o + Send + 'static
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].returning(f)
            }

            /// Just like
            /// [`Expectation::return_once`](struct.Expectation.html#method.return_once)
            pub fn return_once<F>(&mut self, f: F) -> &mut Expectation<$($generics,)*>
                where F: FnOnce($( $argty, )*) -> $o + Send + 'static
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].return_once(f)
            }

            /// Just like
            /// [`Expectation::returning_st`](struct.Expectation.html#method.returning_st)
            pub fn returning_st<F>(&mut self, f: F) -> &mut Expectation<$($generics,)*>
                where F: FnMut($( $argty, )*) -> $o + 'static
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].returning_st(f)
            }

            /// Just like
            /// [`Expectation::times`](struct.Expectation.html#method.times)
            pub fn times(&mut self, n: usize) -> &mut Expectation<$($generics,)*> {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].times(n)
            }

            /// Just like
            /// [`Expectation::times_any`](struct.Expectation.html#method.times_any)
            pub fn times_any(&mut self) -> &mut Expectation<$($generics,)*> {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].times_any()
            }

            /// Just like
            /// [`Expectation::times_range`](struct.Expectation.html#method.times_range)
            pub fn times_range(&mut self, range: Range<usize>) -> &mut Expectation<$($generics,)*>
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].times_range(range)
            }

            /// Just like
            /// [`Expectation::with`](struct.Expectation.html#method.with)
            #[allow(non_camel_case_types)]  // Repurpose $altargs for generics
            pub fn with<$( $altargs: $crate::Predicate<$matchty> + Send + 'static,)*>
                (&mut self, $( $args: $altargs,)*)
                -> &mut Expectation<$($generics,)*>
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].with($($args,)*)
            }

            /// Just like
            /// [`Expectation::withf`](struct.Expectation.html#method.withf)
            pub fn withf<F>(&mut self, f: F) -> &mut Expectation<$($generics,)*>
                where F: Fn($( &$matchty, )* ) -> bool + Send + 'static
            {
                let key = $crate::Key::new::<($($argty,)*), ()>();
                let ee: &mut Expectations<$($generics,)*> =
                    self.guard.store.get_mut(&key).unwrap()
                    .downcast_mut().unwrap();
                ee.0[self.i].withf(f)
            }
        }
        }
    };

}
