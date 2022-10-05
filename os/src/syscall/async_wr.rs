use super::fs::{sys_write, sys_close};
use crate::task::{current_process, current_user_token};
use crate::mm::{translated_byte_buffer, UserBuffer};

// tid 表示当前用户进程执行的写协程， rtid 表示对应的读协程
// 向文件中写完之后，应该唤醒对应的 read 协程
pub fn async_sys_write(fd: usize, buf: *const u8, len: usize, _tid: usize, _pid: usize, read_fd: usize) -> isize {
    sys_write(fd, buf, len);
    sys_close(fd);
    0
}

pub fn async_sys_read(fd: usize, buf: *const u8, len: usize, tid: usize, pid: usize, thread_id: usize) -> isize {
    -1
}