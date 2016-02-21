//! This crate provides (at this time) a single function, `take()`.
//!
//! `take()` allows for taking `T` out of a `&mut T`, doing anything with it including consuming it, and producing another `T` to put back in the `&mut T`.
//!
//! During `take()`, if a panic occurs, the entire process will be exited, as there's no valid `T` to put back into the `&mut T`.
//!
//! Contrast with `std::mem::replace()`, which allows for putting a different `T` into a `&mut T`, but requiring the new `T` to be available before being able to consume the old `T`.

mod exit_on_panic;

use exit_on_panic::exit_on_panic;

/// Allows use of multiple values pointed to by `&mut T` as though they were owned, as long as the `T`'s are made available afterwards.
///
/// Currently, this macro is limited to a maximum of 5 bindings in a single block.
///
/// For more information, see `[take](fn.take.html)`.
///
/// # Example
/// ```
/// #[macro_use]
/// extern crate take_mut;
///
/// # fn main() {
/// struct Foo;
/// struct Bar;
/// let mut foo = Foo;
/// let mut bar = Bar;
/// take_multi!(&mut foo, &mut bar, |mut f, mut b| {
///     // Brackets and the mut on the closure arguments must be used.
///     // Can access both f and b, later providing a new value.
///     drop(f);
///     drop(b);
///     // Do more stuff
///     f = Foo;  // Values are taken out of the closure automatically.
///     b = Bar;  // Just set them to the new values.
/// });
/// # }
/// ```
#[macro_export]
macro_rules! take_multi {
    (to_expr, $e:expr) => {$e};

    ($mb1:expr, |mut $b1:ident| $body:block) => {
// For anyone wondering, this is a fake closure.
// It takes the syntax of a closure, but actually is placed into a different closure inline.
        $crate::take($mb1, |mut $b1| {
// Sadly, mut by default is nessary. It would be significantly more complicated to make it optional in the macro invocation.
            $body;
            take_multi!(to_expr, $b1)
        })
    };

    ($mb1:expr, $mb2:expr, |mut $b1:ident, mut $b2:ident| $body:block) => {
        take_multi!($mb1, |mut $b1| {
            $b1 = $crate::take_used_for_macros_1($mb2, |mut $b2| {
                $body;
                (take_multi!(to_expr, $b2), $b1)
            });
        })
    };

    ($mb1:expr, $mb2:expr, $mb3:expr, |mut $b1:ident, mut $b2:ident, mut $b3:ident| $body:block) => {
        take_multi!($mb1, $mb2, |mut $b1, mut $b2| {
            let (temp1, temp2) = $crate::take_used_for_macros_2($mb3, |mut $b3| {
                $body;
                (take_multi!(to_expr, $b3), $b1, $b2)
            });
            $b1 = temp1;
            $b2 = temp2;
        })
    };

    ($mb1:expr, $mb2:expr, $mb3:expr, $mb4:expr, |mut $b1:ident, mut $b2:ident, mut $b3:ident, mut $b4:ident| $body:block) => {
        take_multi!($mb1, $mb2, $mb3, |mut $b1, mut $b2, mut $b3| {
            let (temp1, temp2, temp3) = $crate::take_used_for_macros_3($mb4, |mut $b4| {
                $body;
                (take_multi!(to_expr, $b4), $b1, $b2, $b3)
            });
            $b1 = temp1;
            $b2 = temp2;
            $b3 = temp3;
        })
    };

    ($mb1:expr, $mb2:expr, $mb3:expr, $mb4:expr, $mb5:expr, |mut $b1:ident, mut $b2:ident, mut $b3:ident, mut $b4:ident, mut $b5:ident| $body:block) => {
        take_multi!($mb1, $mb2, $mb3, $mb4, |mut $b1, mut $b2, mut $b3, mut $b4| {
            let (temp1, temp2, temp3, temp4) = $crate::take_used_for_macros_4($mb5, |mut $b5| {
                $body;
                (take_multi!(to_expr, $b5), $b1, $b2, $b3, $b4)
            });
            $b1 = temp1;
            $b2 = temp2;
            $b3 = temp3;
            $b4 = temp4;
        })
    };
}

/// Allows use of a value pointed to by `&mut T` as though it was owned, as long as a `T` is made available afterwards.
///
/// The closure must return a valid T.
/// # Important
/// Will exit the program (with status code -1) if the closure panics.
///
/// # Example
/// ```
/// struct Foo;
/// let mut foo = Foo;
/// take_mut::take(&mut foo, |foo| {
///     // Can now consume the Foo, and provide a new value later
///     drop(foo);
///     // Do more stuff
///     Foo // Return new Foo from closure, which goes back into the &mut Foo
/// });
/// ```
pub fn take<T, F>(mut_ref: &mut T, closure: F)
    where F: FnOnce(T) -> T
{
    use std::ptr;
    exit_on_panic(|| {
        unsafe {
            let old_t = ptr::read(mut_ref);
            let new_t = closure(old_t);
            ptr::write(mut_ref, new_t);
        }
    });
}

/// Used internally in the take_multi! macro.
#[doc(hidden)]
pub fn take_used_for_macros_1<T, F, U>(mut_ref: &mut T, closure: F) -> U
    where F: FnOnce(T) -> (T, U)
{
    use std::ptr;
    exit_on_panic(|| {
        unsafe {
            let old_t = ptr::read(mut_ref);
            let (new_t, mv) = closure(old_t);
            ptr::write(mut_ref, new_t);
            mv
        }
    })
}

/// Used internally in the take_multi! macro.
#[doc(hidden)]
pub fn take_used_for_macros_2<T, F, U, V>(mut_ref: &mut T, closure: F) -> (U, V)
    where F: FnOnce(T) -> (T, U, V)
{
    use std::ptr;
    exit_on_panic(|| {
        unsafe {
            let old_t = ptr::read(mut_ref);
            let (new_t, mvu, mvv) = closure(old_t);
            ptr::write(mut_ref, new_t);
            (mvu, mvv)
        }
    })
}

/// Used internally in the take_multi! macro.
#[doc(hidden)]
pub fn take_used_for_macros_3<T, F, U, V, W>(mut_ref: &mut T, closure: F) -> (U, V, W)
    where F: FnOnce(T) -> (T, U, V, W)
{
    use std::ptr;
    exit_on_panic(|| {
        unsafe {
            let old_t = ptr::read(mut_ref);
            let (new_t, mvu, mvv, mvw) = closure(old_t);
            ptr::write(mut_ref, new_t);
            (mvu, mvv, mvw)
        }
    })
}

/// Used internally in the take_multi! macro.
#[doc(hidden)]
pub fn take_used_for_macros_4<T, F, U, V, W, X>(mut_ref: &mut T, closure: F) -> (U, V, W, X)
    where F: FnOnce(T) -> (T, U, V, W, X)
{
    use std::ptr;
    exit_on_panic(|| {
        unsafe {
            let old_t = ptr::read(mut_ref);
            let (new_t, mv_u, mv_v, mv_w, mv_x) = closure(old_t);
            ptr::write(mut_ref, new_t);
            (mv_u, mv_v, mv_w, mv_x)
        }
    })
}

#[cfg(test)]
mod test {
    use take as take_mut;

    #[derive(PartialEq, Eq, Debug)]
    enum Foo {
        A,
        B,
    }
    impl Drop for Foo {
        fn drop(&mut self) {
            match *self {
                Foo::A => println!("Foo::A dropped"),
                Foo::B => println!("Foo::B dropped"),
            }
        }
    }

    #[derive(PartialEq, Eq, Debug)]
    enum Bar {
        C,
        D,
    }
    impl Drop for Bar {
        fn drop(&mut self) {
            match *self {
                Bar::C => println!("Bar::C dropped"),
                Bar::D => println!("Bar::D dropped"),
            }
        }
    }

    #[test]
    fn take() {
        let mut foo = Foo::A;
        take_mut(&mut foo, |f| {
            drop(f);
            Foo::B
        });
        assert_eq!(&foo, &Foo::B);
    }

    #[test]
    fn take_multi_1() {
        let mut foo = Foo::A;
        take_multi!(&mut foo, |mut f| {
            drop(f);
            f = Foo::B;
        });
        assert_eq!(&foo, &Foo::B);
    }

    #[test]
    fn take_multi_2() {
        let mut foo = Foo::A;
        let mut bar = Bar::C;
        take_multi!(&mut foo, &mut bar, |mut f, mut b| {
            drop(f);
            drop(b);
            f = Foo::B;
            b = Bar::D;
        });
        assert_eq!(&foo, &Foo::B);
        assert_eq!(&bar, &Bar::D);
    }

    #[test]
    fn take_multi_3() {
        let mut foo1 = Foo::A;
        let mut foo2 = Foo::A;
        let mut bar = Bar::C;
        take_multi!(&mut foo1, &mut foo2, &mut bar, |mut f1, mut f2, mut b| {
            drop(f1);
            drop(f2);
            drop(b);
            f1 = Foo::B;
            f2 = Foo::B;
            b = Bar::D;
        });
        assert_eq!(&foo1, &Foo::B);
        assert_eq!(&foo2, &Foo::B);
        assert_eq!(&bar, &Bar::D);
    }

    #[test]
    fn take_multi_4() {
        let mut foo1 = Foo::A;
        let mut foo2 = Foo::A;
        let mut bar1 = Bar::C;
        let mut bar2 = Bar::C;
        take_multi!(&mut foo1, &mut foo2, &mut bar1, &mut bar2, |mut f1, mut f2, mut b1, mut b2| {
            drop(f1);
            drop(f2);
            drop(b1);
            drop(b2);
            f1 = Foo::B;
            f2 = Foo::B;
            b1 = Bar::D;
            b2 = Bar::D;
        });
        assert_eq!(&foo1, &Foo::B);
        assert_eq!(&foo2, &Foo::B);
        assert_eq!(&bar1, &Bar::D);
        assert_eq!(&bar2, &Bar::D);
    }

    #[test]
    fn take_multi_5() {
        let mut foo1 = Foo::A;
        let mut foo2 = Foo::A;
        let mut bar1 = Bar::C;
        let mut bar2 = Bar::C;
        let mut bar3 = Bar::C;
        take_multi!(&mut foo1, &mut foo2, &mut bar1, &mut bar2, &mut bar3, |mut f1, mut f2, mut b1, mut b2, mut b3| {
            drop(f1);
            drop(f2);
            drop(b1);
            drop(b2);
            drop(b3);
            f1 = Foo::B;
            f2 = Foo::B;
            b1 = Bar::D;
            b2 = Bar::D;
            b3 = Bar::D;
        });
        assert_eq!(&foo1, &Foo::B);
        assert_eq!(&foo2, &Foo::B);
        assert_eq!(&bar1, &Bar::D);
        assert_eq!(&bar2, &Bar::D);
        assert_eq!(&bar3, &Bar::D);
    }
}
