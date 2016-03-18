//! This crate provides (at this time) a single function, `take()`.
//!
//! `take()` allows for taking `T` out of a `&mut T`, doing anything with it including consuming it, and producing another `T` to put back in the `&mut T`.
//!
//! During `take()`, if a panic occurs, the entire process will be exited, as there's no valid `T` to put back into the `&mut T`.
//!
//! Contrast with `std::mem::replace()`, which allows for putting a different `T` into a `&mut T`, but requiring the new `T` to be available before being able to consume the old `T`.

mod exit_on_panic;

use exit_on_panic::exit_on_panic;

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
    exit_on_panic(|| {
        unsafe {
            let old_t = ptr::read(mut_ref);
            let new_t = closure(old_t);
            ptr::write(mut_ref, new_t);
        }
    });
}

use std::cell::Cell;

pub struct Scope {
    active_holes: Cell<usize>
}

impl Scope {
    pub fn scope<F>(f: F)
    where F: FnOnce(&Scope) {
        let aborter = AbortOnSuddenDrop::new();
        let this = Scope { active_holes: Cell::new(0) };
        f(&this);
        if this.active_holes.get() != 0 {
            panic!("There are still unfilled Holes!");
        }
        aborter.done();
    }
    // TODO: NEED TO GUARANTEE THAT MUTABLE OBJECT IN QUESTION IS NOT A SMALLER LIFETIME
    pub fn take<'s, 'm: 's, T: 'm>(&'s self, mut_ref: &'m mut T) -> (T, Hole<'s, 'm, T>) {
        use std::ptr;
        
        let num_holes = self.active_holes.get();
        if num_holes == std::usize::MAX {
            panic!("Failed to create new Hole, already usize::MAX unfilled holes.");
        }
        self.active_holes.set(num_holes + 1);
        let t: T;
        let hole: Hole<'s, 'm, T>;
        unsafe {
            t = ptr::read(mut_ref);
            hole = Hole { scope: self, hole: mut_ref };
        };
        (t, hole)
    }
}

pub struct Hole<'scope, 'm, T: 'm> {
    scope: &'scope Scope,
    hole: &'m mut T
}

impl<'scope, 'm, T: 'm> Hole<'scope, 'm, T> {
    pub fn fill(self, t: T) {
        use std::ptr;
        
        unsafe {
            ptr::write(self.hole, t);
        }
        
        self.scope.active_holes.set(self.scope.active_holes.get() - 1);
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
    take(&mut foo, |mut f| {
       drop(f);
       Foo::B
    });
    assert_eq!(&foo, &Foo::B);
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
    Scope::scope(|scope| {
        let (a, a_hole) = scope.take(&mut bar.a);
        let (b, b_hole) = scope.take(&mut bar.b);
        // Imagine consuming a and b
        a_hole.fill(Foo);
        b_hole.fill(Foo);
    });
    println!("{:?}", &bar);
}