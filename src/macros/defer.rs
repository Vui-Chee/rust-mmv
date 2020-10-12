//! This file contains `defer!` macro for executing deferred logic
//! once function is out of scope.
//!
//! For reference, see
//! https://stackoverflow.com/questions/29963449/golang-like-defer-in-rust.

pub struct ScopeCall<F: FnOnce()> {
    pub c: Option<F>,
}

impl<F: FnOnce()> Drop for ScopeCall<F> {
    fn drop(&mut self) {
        self.c.take().unwrap()()
    }
}

macro_rules! expr {
    ($e: expr) => {
        $e
    };
} // tt hack

#[macro_export]
macro_rules! defer {
    ($($data: tt)*) => (
        let _scope_call = ScopeCall {
            c: Some(|| -> () { expr!({ $($data)* }) })
        };
    )
}
