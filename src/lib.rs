
mod abort_on_panic;

use abort_on_panic::abort_on_panic;

/// Allows use of a value inside a `&mut T` as though it was owned by the closure
///
/// The closure must return a valid T
/// # Aborts
/// Will abort the program (exiting with status code -1) if the closure panics.
///
/// # Example
/// ```
/// struct Foo;
/// let mut foo = Foo;
/// take_mut::take(&mut foo, |foo| {
///     // Can now consume from the reference, and provide a new value later
///     drop(foo);
///     // Do more stuff
///     Foo // Return new Foo from closure
/// });
pub fn take<T, F>(mut_ref: &mut T, closure: F)
  where F: FnOnce(T) -> T {
    use std::ptr;
    abort_on_panic(|| {
        unsafe {
            let old_t = ptr::read(mut_ref);
            let new_t = closure(old_t);
            ptr::write(mut_ref, new_t);
        }
    });
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