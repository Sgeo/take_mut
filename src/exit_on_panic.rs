/// Used to ensure an exit on panic.
/// Call .done() to consume without exiting
#[derive(Debug)]
struct ExitOnSuddenDrop;

impl ExitOnSuddenDrop {

    pub fn new() -> Self {
        ExitOnSuddenDrop
    }
    /// Consume `self` without exiting
    pub fn done(self) {
        ::std::mem::forget(self);
    }
}

impl Drop for ExitOnSuddenDrop {
    fn drop(&mut self) {
        ::std::process::exit(-1);
    }
}


/// Calls its closure.
/// If the closure panics, kill the process.
pub fn exit_on_panic<R, F: FnOnce() -> R>(f: F) -> R {
    let exiter = ExitOnSuddenDrop::new();
    let result = f();
    exiter.done();
    result
}