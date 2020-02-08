use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread,
    thread::Thread,
};

/// Simple executor that puts the current thread to sleep until the given future
/// is ready.
pub fn block_on<F: Future>(mut future: F) -> F::Output {
    fn create_raw_waker(thread: Thread) -> RawWaker {
        RawWaker::new(
            Box::into_raw(Box::new(thread)) as *const _,
            &RawWakerVTable::new(
                |ptr| unsafe { create_raw_waker((&*(ptr as *const Thread)).clone()) },
                |ptr| unsafe {
                    Box::from_raw(ptr as *mut Thread).unpark();
                },
                |ptr| unsafe {
                    (&*(ptr as *const Thread)).unpark();
                },
                |ptr| unsafe {
                    Box::from_raw(ptr as *mut Thread);
                },
            ),
        )
    }

    let waker = unsafe { Waker::from_raw(create_raw_waker(thread::current())) };
    let mut context = Context::from_waker(&waker);
    let mut future = unsafe { Pin::new_unchecked(&mut future) };

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::park(),
        }
    }
}

pub fn kill<O: Into<Option<i32>>>(pid: O) {
    match pid.into() {
        Some(p) => unsafe {
            libc::kill(p, 2);
        },

        None => unsafe {
            libc::kill(libc::getpid(), 2);
        },
    }
}
