/// ccmap -- 表示协程与协程之间的对应关系 
/// 当前是通过 (key, value) 这种形式，一旦完成某个协程，则会唤醒对应的另一个协程

use lazy_static::*;
use alloc::collections::BTreeMap;
use spin::Mutex;
use alloc::sync::Arc;
use crate::add_callback;
use super::TaskId;


lazy_static! {
    pub static ref WRMAP: Arc<Mutex<WRMap>> = Arc::new(Mutex::new(WRMap::new()));
}

/// key -> r_id, write coroutine can use WRMAP to find the corresponding read coroutine id 
pub struct WRMap {
    map: BTreeMap<usize, usize>,
}

impl WRMap {
    pub fn new() -> Self {
        Self { map: BTreeMap::new(), }
    }

    pub fn register(&mut self, k: usize, rid: usize) {
        self.map.insert(k, rid);
    }

    pub fn get_rid(&mut self, k: usize) -> Option<usize> {
        let mut ret = None;
        if self.map.get(&k).is_some() {
            ret = Some(*self.map.get(&k).unwrap());
            self.map.remove(&k);
        }
        ret
    }
}

/// 唤醒 pid 进程内 key 对应的 kernel 协程
pub fn wake_kernel_tid(pid: usize, key: usize) {
    let kernel_tid = WRMAP.lock().get_rid(key);
    if kernel_tid.is_some() {
        add_callback(0, 0, kernel_tid.unwrap());
    }
}

/// 向 WRMAP 中注册 (write_tid, read_tid)，read_tid 通过执行协程时会获取的 CUR_COROTINE 获取
pub fn wrmap_register(key: usize, kernel_tid: usize) {
    unsafe {
        // crate::kprintln!("register write_tid {} ~ read_tid {}", key, kernel_tid);
        WRMAP.lock().register(key, kernel_tid);
    }
}