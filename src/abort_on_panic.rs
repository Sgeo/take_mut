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
            ::std::process::exit(-1);
        }
    }
}


/// Calls its closure.
/// If the closure panics, kill the process.
pub fn abort_on_panic<R, F: FnOnce() -> R>(f: F) -> R {
    let aborter = AbortOnSuddenDrop::new();
    let result = f();
    aborter.done();
    result
}