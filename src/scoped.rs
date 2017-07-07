use std;
use std::panic;
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
        fn panic<T>() -> T {
            panic!("Failed to recover a Hole!")
        }
        self.take_and_recover(mut_ref, panic)
    }
}

pub fn scope<'s, F, R>(f: F) -> R
    where F: FnOnce(&Scope<'s>) -> R {
    let this = Scope { active_holes: Cell::new(0), marker: PhantomData };
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        f(&this)
    }));
    if this.active_holes.get() != 0 {
        std::process::abort();
    }
    match result {
        Ok(r) => r,
        Err(p) => panic::resume_unwind(p),
    }
    
}

#[must_use]
pub struct Hole<'c, 'm, T: 'm, F: FnOnce() -> T> {
    active_holes: &'c Cell<usize>,
    hole: &'m mut T,
    recovery: Option<F>,
}

impl<'c, 'm, T: 'm, F: FnOnce() -> T> Hole<'c, 'm, T, F> {
    pub fn fill(mut self, t: T) {
        use std::ptr;
        use std::mem;
        
        unsafe {
            ptr::write(self.hole, t);
        }
        let num_holes = self.active_holes.get();
        self.active_holes.set(num_holes - 1);
        mem::forget(self);
    }
}

impl<'c, 'm, T: 'm, F: FnOnce() -> T> Drop for Hole<'c, 'm, T, F> {
    fn drop(&mut self) {
        use std::ptr;
        use std::mem;
        
        let t = (self.recovery.take().expect("No recovery function in Hole!"))();
        unsafe {
            ptr::write(self.hole, t);
        }
        let num_holes = self.active_holes.get();
        self.active_holes.set(num_holes - 1);
    }
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