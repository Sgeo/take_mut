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

use std::rc::Rc;
use std::marker::PhantomData;

pub struct Scope<'s> {
    active_holes: Rc<()>,
    marker: PhantomData<&'s mut ()>
}

impl<'s> Scope<'s> {

    // Guarantees break if this is &self instead of &mut self, and I don't know why
    // Reason to use Rcs is because can't return a & from a &mut self nicely
    pub fn take<'m: 's, T: 'm>(&mut self, mut_ref: &'m mut T) -> (T, Hole<'m, T>) {
        use std::ptr;
        
        let t: T;
        let hole: Hole<'m, T>;
        unsafe {
            t = ptr::read(mut_ref);
            hole = Hole { active_holes: Some(self.active_holes.clone()), hole: mut_ref };
        };
        (t, hole)
    }
}

pub fn scope<'s, F, R>(f: F) -> R
    where F: FnOnce(&mut Scope<'s>) -> R {
    exit_on_panic(|| {
        let mut this = Scope { active_holes: Rc::new(()), marker: PhantomData };
        let r = f(&mut this);
        if Rc::strong_count(&this.active_holes) != 1 {
            panic!("There are still unfilled Holes at the end of the scope!");
        }
        r
    })
}

#[must_use]
pub struct Hole<'m, T: 'm> {
    active_holes: Option<Rc<()>>,
    hole: &'m mut T
}

impl<'m, T: 'm> Hole<'m, T> {
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

impl<'m, T: 'm> Drop for Hole<'m, T> {
    fn drop(&mut self) {
        panic!("An unfilled Hole was destructed!");
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
    scope(|scope| {
        let (a, a_hole) = scope.take(&mut bar.a);
        let (b, b_hole) = scope.take(&mut bar.b);
        // Imagine consuming a and b
        a_hole.fill(Foo);
        b_hole.fill(Foo);
    });
    println!("{:?}", &bar);
}