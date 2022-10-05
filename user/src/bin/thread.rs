#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use user_lib::*;

pub const MAX_LEN: usize = 5;
pub const REQUEST: &str = "send request";   // 测试数据
pub const BUFFER_SIZE: usize = 4096;        // 缓冲区大小

static mut GFD1: [usize; 2] = [0usize; 2];
static mut GFD2: [usize; 2] = [0usize; 2];

pub fn client_thread() -> ! {
    // let pid = getpid() as usize;
    // let thread_id = gettid() as usize;
    // let tid = alloc_task_id();
    // unsafe {
    //     add_task_with_prority(Box::pin(client(GFD1[1], GFD2[0], tid, pid, thread_id, GFD1[0])),
    //                           0, pid, thread_id, tid);
    // }
    // user_thread_main(pid, thread_id);
    exit(1)
}

pub fn server_thread(read_fd: usize) -> ! {
    // let pid = getpid() as usize;
    // let thread_id = gettid() as usize;
    // let tid = alloc_task_id();
    // unsafe {
    //     add_task_with_prority(Box::pin(server(GFD1[0], GFD2[1], tid, pid, GFD2[0])),
    //                           1, pid, thread_id, tid);
    // }
    // user_thread_main(pid, thread_id);
    exit(2)
}

#[no_mangle]
pub fn main() -> i32 {
    // unsafe {
    //     pipe(&mut GFD1);
    //     pipe(&mut GFD2);
    // }
    // let v = vec![
    //     thread_create(server_thread as usize, 0),
    //     thread_create(client_thread as usize, 0),
    // ];
    // for tid in v.iter() {
    //     let exit_code = waittid(*tid as usize);
    //     println!("thread#{} exited with code {}", tid, exit_code);
    // }
    // println!("main thread exited.");
    0
}

// 服务端接收用户端的请求，从管道中读取内容，然后向客户端写响应内容
async fn server(fd1: usize, fd2: usize, tid: usize, pid: usize, read_fd: usize) {
    // println!("server read start---");
    // let mut buffer = [0u8; BUFFER_SIZE];
    // read(fd1, &mut buffer);
    // println!("server read end");
    //
    // println!("server write start---");
    // let resp = REQUEST;
    // async_write(fd2, resp.as_bytes().as_ptr() as usize, resp.len(), tid, pid, read_fd);
    // println!("server write end");
}

// 客户端发送请求，向管道中写请求内容，然后读取管道中服务器发送的响应内容
async fn client(fd1: usize, fd2: usize, tid: usize, pid: usize, thread_id: usize, read_fd: usize) {
    // 向一个管道中写入数据cd
    // println!("client write start---");
    // let req = REQUEST;
    // async_write(fd1, req.as_bytes().as_ptr() as usize, req.len(), tid, pid, read_fd);
    // println!("client write end");
    // // 从另一个管道中异步的读数据
    // println!("client read start---");
    // let buffer = [0u8; BUFFER_SIZE];
    // let ac_r = AsyncCall::new(ASYNC_SYSCALL_READ, fd2, buffer.as_ptr() as usize, buffer.len(), tid, pid, thread_id);
    // ac_r.await;
    // print!("------------------buffer: ");
    // for c in buffer {
    //     if c != 0 {
    //         print!("{}", c as char);
    //     }
    // }
    // println!("");
    // println!("client read end");
}
