use alloc::{vec::Vec, collections::VecDeque};
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::coroutine::config::{CAP, PRIO_NUM};


#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct TaskId {
    pub tid: usize,
    pub prio: usize,
}

impl TaskId {
    pub(crate) fn generate() -> usize {
        // 任务编号计数器，任务编号自增
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        if id >= PRIO_NUM * CAP {
            // TODO: 不让系统 Panic
            panic!("too many tasks!")
        }
        id
    }
}

pub trait Scheduler: 'static {
    // 如果 tid 不存在，表明将一个新线程加入线程调度
    // 否则表明一个已有的线程要继续运行
    fn push(&mut self, tid: TaskId);

    // 从若干可运行线程中选择一个运行
    fn pop(&mut self) -> Option<TaskId>;

    fn empty(&self) -> bool;

    fn high_prio(&self) -> usize;
}

pub struct PrioScheduler {
    queue: Vec<VecDeque<TaskId>>,
    high_prio: usize,
    task_num: usize,
}

impl PrioScheduler {
    pub fn new() -> Self {
        Self {
            queue: (0..PRIO_NUM).map(|_| VecDeque::new() ).collect::<Vec<VecDeque<TaskId>>>(),
            high_prio: 0,
            task_num: 0,
        }
    }
}

impl Scheduler for PrioScheduler {
    fn push(&mut self,  tid: TaskId) {
        let prio = tid.prio;
        self.queue[prio].push_back(tid);
        self.task_num += 1;
        if prio > self.high_prio {
            self.high_prio = prio;
        }
    }

    fn pop(&mut self) -> Option<TaskId> {
        if self.queue[self.high_prio].is_empty() {
            self.high_prio = 0;
            for i in 0..PRIO_NUM {
                if !self.queue[PRIO_NUM - i - 1].is_empty() {
                    self.high_prio = PRIO_NUM - i - 1;
                }
            }
            println!("prio changed");
            return None;
        }
        self.task_num -= 1;
        let ret = self.queue[self.high_prio].pop_front();
        ret
    }

    fn empty(&self) -> bool {
        self.task_num == 0
    }

    fn high_prio(&self) -> usize {
        self.high_prio
    }
}