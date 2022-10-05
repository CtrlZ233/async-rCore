use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::ptr::null;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Waker, Poll, Context, RawWaker, RawWakerVTable};
use spin::Mutex;
use crate::console::print;
use crate::coroutine::scheduler::TaskId;

use super::{task_waker::TaskWaker};


pub fn alloc_task_id() -> usize {
    TaskId::generate()
}


// Task包装协程
pub struct UserTask{
    /// 任务编号
    pub tid: TaskId,
    /// future
    pub future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>,
    /// waker
    pub waker: Arc<Waker>,

}

impl UserTask{
    //创建一个协程
    pub fn new(future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>, prio: usize) -> Self{
        let task_id = TaskId{
            tid: TaskId::generate(),
            prio,
        };
        unsafe {
            UserTask{
                tid: task_id,
                future,
                waker: Arc::new(TaskWaker::new()),
            }
        }

    }

    pub fn execute(& self) -> Poll<()> {
        let waker = self.waker.clone();
        let mut context = Context::from_waker(&*waker);
        self.future.lock().as_mut().poll(&mut context)
    }
}

