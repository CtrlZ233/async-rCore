use alloc::sync::Arc;
use spin::Mutex;
use alloc::boxed::Box;
use core::task::Waker;
use alloc::task::Wake;
use crate::task::schduler::{PrioScheduler, Scheduler};
use super::schduler::TaskId;

pub struct TaskWaker {}

// 仅用于存储执行上下文，我们手动唤醒任务
impl TaskWaker {
    pub fn new() -> Waker {
        Waker::from(
            Arc::new(Self {}
            )
        )
    }

    fn wake_task(&self) {}
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}