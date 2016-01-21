

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
        use std::mem;
        
        let num_holes = self.active_holes.get();
        if num_holes == std::usize::MAX {
            panic!("Failed to create new Hole, already usize::MAX unfilled holes.");
        }
        self.active_holes.set(num_holes + 1);
        let t: T;
        let hole: Hole<'s, 'm, T>;
        unsafe {
            t = mem::replace(mut_ref, mem::uninitialized());
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