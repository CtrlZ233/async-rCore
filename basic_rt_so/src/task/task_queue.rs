use alloc::{vec::Vec, collections::VecDeque};
use super::{bitmap::BitMap, user_task::TaskId};
use crate::config::PRIO_NUM;
use crate::task::manager::BITMAPS;

pub struct TaskQueue {
    queue: Vec<VecDeque<TaskId>>,
    // pub bitmap: BitMap,
    pid: usize,
    thread_id: usize,
    pub task_num: usize,
}

impl TaskQueue {
    pub fn new(pid: usize, thread_id: usize) -> Self {
        Self {
            queue: (0..PRIO_NUM).map(|_| VecDeque::new() ).collect::<Vec<VecDeque<TaskId>>>(),
            // bitmap: BitMap::new(),
            pid,
            thread_id,
            task_num: 0,
        }
    }

    pub fn add_tid(&mut self,  tid: TaskId, prio: usize) {
        self.queue[prio].push_back(tid);
        // self.bitmap.update(prio, true);\
        BITMAPS[self.pid][self.thread_id].update(prio, true);
        self.task_num += 1;
    }

    pub fn pop_tid(&mut self) -> Option<TaskId> {
        if self.task_num == 0 {
            return None;
        }
        // let prio = self.bitmap.get_priority();
        let prio = BITMAPS[self.pid][self.thread_id].get_priority();
        if prio == PRIO_NUM {
            None
        } else {
            self.task_num -= 1;
            let ret = self.queue[prio].pop_front();
            if self.queue[prio].len() == 0 { BITMAPS[self.pid][self.thread_id].update(prio, false); }
            ret
        }
    }

    pub fn is_empty(&self) -> bool {
        self.task_num == 0
    }
}