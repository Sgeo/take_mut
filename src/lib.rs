

/// Used to ensure an abort on panic.
/// Call .done() to consume without aborting
#[derive(Debug)]
struct AbortOnSuddenDrop {
    finished: bool
}

impl AbortOnSuddenDrop {

    pub fn new() -> Self {
        AbortOnSuddenDrop { finished: false }
    }
    /// Consume `self` without aborting
    pub fn done(mut self) {
        self.finished = true;
    }
}

impl Drop for AbortOnSuddenDrop {
    fn drop(&mut self) {
        if !self.finished {
            std::process::exit(-1);
        }
    }
}

/// Allows use of a value inside a `&mut T` as though it was owned by the closure
///
/// The closure must return a valid T
/// # Aborts
/// Will abort the program (exiting with status code -1) if the closure panics.
pub fn take<T, F>(mut_ref: &mut T, closure: F)
  where F: FnOnce(T) -> T {
    use std::mem;
    let aborter = AbortOnSuddenDrop::new();
    unsafe {
        let old_t = mem::replace(mut_ref, mem::uninitialized());
        let new_t = closure(old_t);
        let garbage = mem::replace(mut_ref, new_t);
        mem::forget(garbage);
    }
    aborter.done();
}

use std::cell;

pub struct Scope {
    active_holes: cell::Cell<usize>
}

impl Scope {
    pub fn scope<F>(f: F)
    where F: FnOnce(&Scope) {
        let aborter = AbortOnSuddenDrop::new();
        let this = Scope { active_holes: cell::Cell::new(0) };
        f(&this);
        aborter.done();
    }
    // TODO: NEED TO GUARANTEE THAT MUTABLE OBJECT IN QUESTION IS NOT A SMALLER LIFETIME
    pub fn take<'s, 'm: 's, T: 'm>(&'s self, mut_ref: &'m mut T) -> (T, Hole<'s, 'm, T>) {
        self.active_holes.set(self.active_holes.get() + 1);
    }
}

pub struct Hole<'scope, 'm, T: 'm> {
    scope: &'scope Scope,
    hole: &'m mut T
}




#[test]
fn it_works() {
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
}
