use alloc::sync::Arc;
use spin::Mutex;
use alloc::boxed::Box;
use core::task::Waker;
use alloc::task::Wake;
use crate::task::task_queue::{PrioScheduler, Scheduler};
use super::task_queue::TaskId;

pub struct TaskWaker {
    tid: TaskId,
    scheduler: Arc<Mutex<Box<PrioScheduler>>>,
}

impl TaskWaker {
    pub fn new(tid: TaskId, prio: usize, scheduler: Arc<Mutex<Box<PrioScheduler>>>) -> Waker {
        Waker::from(
            Arc::new(Self {
                    tid,
                    scheduler
                }
            )
        )
    }

    fn wake_task(&self) {
        self.scheduler.lock().push(self.tid);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}