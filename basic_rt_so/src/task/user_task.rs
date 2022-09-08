use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Waker, Poll, Context};
use spin::Mutex;

use super::{task_queue::TaskQueue, task_waker::TaskWaker};


#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct TaskId(usize);

impl TaskId {
    pub(crate) fn generate() -> TaskId {
        // 任务编号计数器，任务编号自增
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        if id > usize::MAX / 2 {
            // TODO: 不让系统 Panic
            panic!("too many tasks!")
        }
        TaskId(id)
    }

    pub fn get_tid_by_usize(v: usize) -> Self {
        Self(v)
    }

    pub fn get_val(&self) -> usize {
        self.0
    } 
}



// Task包装协程
pub struct UserTask{
    /// 任务编号
    pub tid: TaskId,
    /// future
    pub future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>, 
    /// 优先级
    pub prio: usize,
    /// waker
    pub waker: Arc<Waker>,

}

impl UserTask{
    //创建一个协程
    pub fn new(future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>, prio: usize, task_queue: Arc<Mutex<Box<TaskQueue>>>) -> Self{
        let tid = TaskId::generate();
        UserTask{
            tid,
            future,
            prio,
            waker: Arc::new(TaskWaker::new(tid, prio, task_queue)),
        }
    }

    pub fn execute(& self) -> Poll<()> {
        let waker = self.waker.clone();
        let mut context = Context::from_waker(&*waker);
        self.future.lock().as_mut().poll(&mut context)
    }
}

