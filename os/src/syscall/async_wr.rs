
use super::fs::{sys_write, sys_close};
use crate::task::{current_process, current_user_token};
use crate::mm::{translated_byte_buffer, UserBuffer};

// tid 表示当前用户进程执行的写协程， rtid 表示对应的读协程
// 向文件中写完之后，应该唤醒对应的 read 协程
pub fn async_sys_write(fd: usize, buf: *const u8, len: usize, _tid: usize, _pid: usize, key: usize) -> isize {
    sys_write(fd, buf, len);
    sys_close(fd);
    // 向文件中写完数据之后，需要唤醒内核当中的协程，将管道中的数据写到缓冲区中，因此 pid 固定为 0
    crate::lkm::wake_kernel_tid(0, key);
    // log::debug!("async_sys_write do nothing");
    0
}

pub fn async_sys_read(fd: usize, buf: *const u8, len: usize, tid: usize, pid: usize, key: usize) -> isize {
    // log::debug!("async_sys_read do nothing");
    let token = current_user_token();
    let process = current_process();
    // let task = current_task().unwrap();
    // let inner = task.acquire_inner_lock();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release Task lock manually to avoid deadlock
        drop(inner);
        //file.read(
        //    UserBuffer::new(translated_byte_buffer(token, buf, len))
        //) as isize
        let work = file.aread(UserBuffer::new(translated_byte_buffer(token, buf, len)), tid, pid, key);
        crate::lkm::add_task_with_prority(work, 0, 0);
        0
    } else {
        -1
    }    
}