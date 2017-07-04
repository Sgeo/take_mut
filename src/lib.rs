//! This crate provides (at this time) a single function, `take()`.
//!
//! `take()` allows for taking `T` out of a `&mut T`, doing anything with it including consuming it, and producing another `T` to put back in the `&mut T`.
//!
//! During `take()`, if a panic occurs, the entire process will be exited, as there's no valid `T` to put back into the `&mut T`.
//! Use `take_or_recover()` to replace the `&mut T` with a recovery value before continuing the panic.
//!
//! Contrast with `std::mem::replace()`, which allows for putting a different `T` into a `&mut T`, but requiring the new `T` to be available before being able to consume the old `T`.

use std::panic;

/// Allows use of a value pointed to by `&mut T` as though it was owned, as long as a `T` is made available afterwards.
///
/// The closure must return a valid T.
/// # Important
/// Will exit the program (with status code 101) if the closure panics.
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
  where F: FnOnce(T) -> T {
    use std::ptr;

    unsafe {
        let old_t = ptr::read(mut_ref);
        let new_t = panic::catch_unwind(panic::AssertUnwindSafe(|| closure(old_t)))
            .unwrap_or_else(|_| ::std::process::exit(101));
        ptr::write(mut_ref, new_t);
    }
}

#[test]
fn it_works() {
    #[derive(PartialEq, Eq, Debug)]
    enum Foo {A, B};
    impl Drop for Foo {
        fn drop(&mut self) {
            match *self {
                Foo::A => println!("Foo::A dropped"),
                Foo::B => println!("Foo::B dropped")
            }
        }
    }
    let mut foo = Foo::A;
    take(&mut foo, |f| {
       drop(f);
       Foo::B
    });
    assert_eq!(&foo, &Foo::B);
}


/// Allows use of a value pointed to by `&mut T` as though it was owned, as long as a `T` is made available afterwards.
///
/// The closure must return a valid T.
/// # Important
/// Will replace `&mut T` with `recover` if the closure panics, then continues the panic.
///
/// # Example
/// ```
/// struct Foo;
/// let mut foo = Foo;
/// take_mut::take_or_recover(&mut foo, || Foo, |foo| {
///     // Can now consume the Foo, and provide a new value later
///     drop(foo);
///     // Do more stuff
///     Foo // Return new Foo from closure, which goes back into the &mut Foo
/// });
/// ```
pub fn take_or_recover<T, F, R>(mut_ref: &mut T, recover: R, closure: F)
  where F: FnOnce(T) -> T, R: FnOnce() -> T {
    use std::ptr;
    unsafe {
        let old_t = ptr::read(mut_ref);
        let new_t = panic::catch_unwind(panic::AssertUnwindSafe(|| closure(old_t)));
        match new_t {
            Err(err) => {
                let r = panic::catch_unwind(panic::AssertUnwindSafe(|| recover()))
                    .unwrap_or_else(|_| ::std::process::exit(101));
                ptr::write(mut_ref, r);
                panic::resume_unwind(err);
            }
            Ok(new_t) => ptr::write(mut_ref, new_t),
        }
    }
}


use std::cell::Cell;
use std::marker::PhantomData;

pub struct Scope<'s> {
    active_holes: Cell<usize>,
    marker: PhantomData<Cell<&'s mut ()>>
}

impl<'s> Scope<'s> {

    
    pub fn take_and_recover<'c, 'm: 's, T: 'm, F: FnOnce() -> T>(&'c self, mut_ref: &'m mut T, recovery: F) -> (T, Hole<'c, 'm, T, F>) {
        use std::ptr;
        
        let t: T;
        let hole: Hole<'c, 'm, T, F>;
        let num_of_holes = self.active_holes.get();
        if num_of_holes == std::usize::MAX {
            panic!("Too many holes!");
        }
        self.active_holes.set(num_of_holes + 1);
        unsafe {
            t = ptr::read(mut_ref);
            hole = Hole {
                active_holes: &self.active_holes,
                hole: mut_ref,
                recovery: Some(recovery)
            };
        };
        (t, hole)
    }
    
    pub fn take<'c, 'm: 's, T: 'm>(&'c self, mut_ref: &'m mut T) -> (T, Hole<'c, 'm, T, fn() -> T>) {
        use std::ptr;
        
        let t: T;
        let hole: Hole<'c, 'm, T, fn() -> T>;
        let num_of_holes = self.active_holes.get();
        if num_of_holes == std::usize::MAX {
            panic!("Too many holes!");
        }
        self.active_holes.set(num_of_holes + 1);
        unsafe {
            t = ptr::read(mut_ref);
            hole = Hole {
                active_holes: &self.active_holes,
                hole: mut_ref,
                recovery: None
            };
        };
        (t, hole)
    }
}

pub fn scope<'s, F, R>(f: F) -> R
    where F: FnOnce(&Scope<'s>) -> R {
    let this = Scope { active_holes: Cell::new(0), marker: PhantomData };
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        f(&this)
    }));
    if this.active_holes.get() != 0 {
        std::process::exit(101);
    }
    match result {
        Ok(r) => r,
        Err(p) => panic::resume_unwind(p),
    }
    
}

#[must_use]
pub struct Hole<'c, 'm, T: 'm, F> {
    active_holes: &'c Cell<usize>,
    hole: &'m mut T,
    recovery: Option<F>,
}

impl<'c, 'm, T: 'm, F> Hole<'c, 'm, T, F> {
    pub fn fill(mut self, t: T) {
        use std::ptr;
        use std::mem;
        
        unsafe {
            ptr::write(self.hole, t);
        }
        self.active_holes.take();
        mem::forget(self);
    }
}

impl<'c, 'm, T: 'm, F> Drop for Hole<'c, 'm, T, F> {
    fn drop(&mut self) {
        panic!("An unfilled Hole was destructed!");
    }
}





#[test]
fn it_works_recover() {
    #[derive(PartialEq, Eq, Debug)]
    enum Foo {A, B};
    impl Drop for Foo {
        fn drop(&mut self) {
            match *self {
                Foo::A => println!("Foo::A dropped"),
                Foo::B => println!("Foo::B dropped")
            }
        }
    }
    let mut foo = Foo::A;
    take_or_recover(&mut foo, || Foo::A, |f| {
       drop(f);
       Foo::B
    });
    assert_eq!(&foo, &Foo::B);
}

#[test]
fn it_works_recover_panic() {
    #[derive(PartialEq, Eq, Debug)]
    enum Foo {A, B, C};
    impl Drop for Foo {
        fn drop(&mut self) {
            match *self {
                Foo::A => println!("Foo::A dropped"),
                Foo::B => println!("Foo::B dropped"),
                Foo::C => println!("Foo::C dropped")
            }
        }
    }
    let mut foo = Foo::A;

    let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        take_or_recover(&mut foo, || Foo::C, |f| {
            drop(f);
            panic!("panic");
            Foo::B
        });
    }));

    assert!(res.is_err());
    assert_eq!(&foo, &Foo::C);
}

#[test]
fn scope_based_take() {
    #[derive(Debug)]
    struct Foo;
    
    #[derive(Debug)]
    struct Bar {
        a: Foo,
        b: Foo
    }
    let mut bar = Bar { a: Foo, b: Foo };
    scope(|scope| {
        let (a, a_hole) = scope.take(&mut bar.a);
        let (b, b_hole) = scope.take(&mut bar.b);
        // Imagine consuming a and b
        a_hole.fill(Foo);
        b_hole.fill(Foo);
    });
    println!("{:?}", &bar);
}

