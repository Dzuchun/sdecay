use core::ffi::CStr;

type S = crate::wrapper::StdString;

const TEXT: &CStr = c"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

mod stack_ref {
    use core::mem::MaybeUninit;

    use crate::container::{
        Container, ExclusiveContainer,
        tests::{S, TEXT},
    };

    type C<'l> = crate::container::RefContainer<'l, S>;

    #[test]
    fn create() {
        let mut tmp = MaybeUninit::uninit();
        let container = S::from_cstr_in::<C<'_>>(&mut tmp, TEXT);
        drop(container);
    }

    #[test]
    fn try_mv() {
        let mut tmp = MaybeUninit::uninit();
        let container = S::from_cstr_in::<C<'_>>(&mut tmp, TEXT);
        let mut tmp = MaybeUninit::uninit();
        let container2 = container.try_mv::<C<'_>>(&mut tmp).unwrap();
        drop(container2);
    }

    #[test]
    fn mv() {
        let mut tmp = MaybeUninit::uninit();
        let container = S::from_cstr_in::<C<'_>>(&mut tmp, TEXT);
        let mut tmp = MaybeUninit::uninit();
        let container2 = container.mv::<C<'_>>(&mut tmp);
        drop(container2);
    }
}

#[cfg(feature = "alloc")]
mod alloc_box {
    use crate::container::{
        ArcContainer, Container, ExclusiveContainer,
        tests::{S, TEXT},
    };

    type C = crate::container::BoxContainer<S>;

    #[test]
    fn create() {
        let container = S::from_cstr_in::<C>((), TEXT);
        drop(container);
    }

    #[test]
    fn try_mv() {
        let container = S::from_cstr_in::<C>((), TEXT);
        let container2 = container.try_mv::<C>(()).unwrap();
        drop(container2);
    }

    #[test]
    fn mv() {
        let container = S::from_cstr_in::<C>((), TEXT);
        let container2 = container.mv::<C>(());
        drop(container2);
    }

    #[test]
    fn mv_to_arc() {
        let container = S::from_cstr_in::<C>((), TEXT);
        let container2 = container.mv::<ArcContainer<S>>(());
        let container3 = container2.clone();
        drop(container2);
        drop(container3);
    }
}

// #[cfg(feature = "alloc")]
// mod alloc_rc {
//     use crate::container::{
//         BoxContainer, Container,
//         tests::{S, TEXT},
//     };
//
//     type C = crate::container::RcContainer<S>;
//
//     #[test]
//     fn create() {
//         let container = S::from_cstr_in::<C>((), TEXT);
//         drop(container);
//     }
//
//     #[test]
//     fn create_clone() {
//         let container = S::from_cstr_in::<C>((), TEXT);
//         let container2 = container.clone();
//         drop(container);
//         drop(container2);
//     }
//
//     #[test]
//     fn try_mv_ok() {
//         let container = S::from_cstr_in::<C>((), TEXT);
//         let container2 = container.try_mv::<C>(()).unwrap();
//         drop(container2);
//     }
//
//     #[test]
//     fn try_mv_err() {
//         let container = S::from_cstr_in::<C>((), TEXT);
//         let container2 = container.clone();
//         let container = container.try_mv::<C>(()).unwrap_err();
//         drop(container);
//         let container3 = container2.try_mv::<C>(()).unwrap();
//         drop(container3);
//     }
//
//     #[test]
//     fn try_mv_to_box() {
//         let container = S::from_cstr_in::<C>((), TEXT);
//         let container2 = container.clone();
//         let _ = container.try_mv::<BoxContainer<S>>(()).unwrap_err();
//         let container3 = container2.try_mv::<BoxContainer<S>>(()).unwrap();
//         drop(container3);
//     }
// }

#[cfg(feature = "alloc")]
mod alloc_arc {
    use core::hint::black_box;
    use std::{sync::Arc, thread::spawn};

    use crate::container::{
        ArcContainer, BoxContainer, Container,
        tests::{S, TEXT},
    };

    type C = crate::container::ArcContainer<S>;

    #[test]
    fn create() {
        let container = S::from_cstr_in::<C>((), TEXT);
        drop(container);
    }

    #[test]
    fn create_clone() {
        let container = S::from_cstr_in::<C>((), TEXT);
        let container2 = container.clone();
        drop(container);
        drop(container2);
    }

    #[test]
    fn try_mv_ok() {
        let container = S::from_cstr_in::<C>((), TEXT);
        let container2 = container.try_mv::<C>(()).unwrap();
        drop(container2);
    }

    #[test]
    fn try_mv_err() {
        let container = S::from_cstr_in::<C>((), TEXT);
        let container2 = container.clone();
        assert!(container.try_mv::<C>(()).is_none());
        let container3 = container2.try_mv::<C>(()).unwrap();
        drop(container3);
    }

    #[test]
    fn try_mv_to_box() {
        let container = S::from_cstr_in::<C>((), TEXT);
        let container2 = container.clone();
        assert!(container.try_mv::<BoxContainer<S>>(()).is_none());
        let container3 = container2.try_mv::<BoxContainer<S>>(()).unwrap();
        drop(container3);
    }

    #[test]
    fn mt_clone() {
        const THREADS: usize = 10;
        const ITERATIONS: usize = 20;
        let container = S::from_cstr_in::<C>((), TEXT);
        let sem = Arc::new(std::sync::Barrier::new(THREADS));
        let handles = (0..THREADS)
            .map(|i| {
                spawn({
                    let mut container = container.clone();
                    let sem = sem.clone();
                    move || {
                        for _ in 0..ITERATIONS {
                            sem.wait();
                            println!("Run thread {i}");
                            assert_eq!(container.as_cstr(), TEXT);
                            black_box(container.try_inner());
                        }
                        drop(container);
                    }
                })
            })
            .collect::<Vec<_>>();
        drop(container);
        handles
            .into_iter()
            .for_each(|h| h.join().expect("Should successfully join thread"));
    }

    #[test]
    fn crate_get_mut() {
        let mut ucontainer = ArcContainer::<i32>::uninit(());
        let ptr = ArcContainer::<i32>::uninit_inner_ptr(&mut ucontainer);
        unsafe { core::ptr::write(ptr, 42) };
        let mut container = unsafe { ArcContainer::init(ucontainer) };
        let _refmut = container
            .try_inner()
            .expect("This is the only container now; should get a reference");
    }

    #[test]
    fn mt_move() {
        for threads in 1..=16 {
            let sem = Arc::new(std::sync::Barrier::new(threads + 1));
            for _ in 0..30 {
                let container = S::from_cstr_in::<C>((), TEXT);
                let handles = (0..threads)
                    .map(|_| {
                        spawn({
                            let container = container.clone();
                            let sem = sem.clone();
                            move || {
                                sem.wait();
                                assert_eq!(container.as_cstr(), TEXT);
                                container.try_mv::<BoxContainer<_>>(())
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                drop(container);
                sem.wait();
                let container: BoxContainer<_> = handles
                    .into_iter()
                    .fold(None, |acc, res| {
                        match (
                            acc,
                            res.join().expect("Should successfully join the thread"),
                        ) {
                            (None, None) => None,
                            (Some(b), None) | (None, Some(b)) => Some(b),
                            (Some(_), Some(_)) => unreachable!("Cannot produce two box containers"),
                        }
                    })
                    .expect("One of the threads should get the value");
                // there was a bug at this point with incorrect `try_move_out` implementation
                drop(container);
            }
        }
    }
}
