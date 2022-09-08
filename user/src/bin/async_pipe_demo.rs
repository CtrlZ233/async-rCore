#![no_std]
#![no_main]

extern crate alloc;

extern crate user_lib;
use user_lib::*;
use alloc::boxed::Box;

#[no_mangle]
pub fn main() -> i32 {
    let pid = getpid() as usize;
    println!("pid is {}, user test>>>>>>>", pid);
    let mut pipe_fd = [0usize; 2];
    pipe(&mut pipe_fd);
    let read_end = pipe_fd[0];
    let write_end = pipe_fd[1];
    // 先添加读的协程，再添加写的协程，两个协程的优先级相同
    add_task_with_prority(Box::pin(server_read(read_end, 0, pid, 333)), 0, pid);
    add_task_with_prority(Box::pin(client_write(write_end, 1, pid, 333)), 1, pid);

    user_thread_main(pid);
    0
}

pub const REQUEST: &str = "send request";
pub const BUFFER_SIZE: usize = 40;

// 服务端接收用户端的请求，从管道中读取内容
async fn server_read(fd: usize, tid: usize, pid: usize, key: usize) {
    println!("server read start");
    let buffer = [0u8; BUFFER_SIZE];
    let read_corotine = AsyncCall::new(ASYNC_SYSCALL_READ, fd, buffer.as_ptr() as usize, buffer.len(), tid, pid, key);
    read_corotine.await;
    print!("buffer: ");
    for c in buffer {
        if c != 0 {
            print!("{}", c as char);
        }
    }
    println!("");
    println!("server read end");
}

// 客户端发送请求，向管道中写请求内容
async fn client_write(fd: usize, tid: usize, pid: usize, key: usize) {
    println!("client write start");
    let req = REQUEST;
    async_write(fd, req.as_bytes().as_ptr() as usize, req.len(), tid, pid, key);
    println!("client write end");
}