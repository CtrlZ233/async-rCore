use alloc::sync::Arc;
use spin::Mutex;
use alloc::boxed::Box;
use super::task_queue::TaskQueue;
use core::task::Waker;
use alloc::task::Wake;
use super::user_task::TaskId;

pub struct TaskWaker {
    tid: TaskId,
    prio: usize,
    queue: Arc<Mutex<Box<TaskQueue>>>,
}

impl TaskWaker {
    pub fn new(tid: TaskId, prio: usize, queue: Arc<Mutex<Box<TaskQueue>>>) -> Waker {
        Waker::from(
            Arc::new(Self {
                    tid,
                    prio,
                    queue,
                }
            )
        )
    }

    fn wake_task(&self) {
        self.queue.lock().add_tid(self.tid, self.prio);
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