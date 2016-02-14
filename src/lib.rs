

/// Allows use of a value inside a `&mut T` as though it was owned by the closure
///
/// The closure must return a valid T
/// # Aborts
/// Will abort the program (exiting with status code -1) if the closure panics.
pub fn take<T, F>(mut_ref: &mut T, closure: F)
  where F: FnOnce(T) -> T {
    use std::ptr;
    let aborter = AbortOnSuddenDrop::new();
    unsafe {
        let old_t = ptr::read(mut_ref);
        let new_t = closure(old_t);
        ptr::write(mut_ref, new_t);
    }
    aborter.done();
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