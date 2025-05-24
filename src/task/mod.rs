use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct YieldNow {
    yielded: bool,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl YieldNow {
    fn new() -> Self {
        YieldNow { yielded: false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub async fn yield_now() {
    YieldNow::new().await
}

pub struct Sleep {
    target_ticks: u64,
}

impl Sleep {
    pub fn new(duration_ms: u64) -> Self {
        let current_ticks = crate::time::get_ticks();
        Sleep {
            target_ticks: current_ticks + duration_ms,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_ticks = crate::time::get_ticks();
        
        if current_ticks >= self.target_ticks {
            Poll::Ready(())
        } else {
            // Wake up on next timer interrupt
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn sleep_ms(duration_ms: u64) -> Sleep {
    Sleep::new(duration_ms)
}