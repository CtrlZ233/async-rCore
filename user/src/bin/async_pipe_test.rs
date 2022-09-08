#![no_std]
#![no_main]

extern crate alloc;

extern crate user_lib;
use user_lib::*;
use alloc::boxed::Box;


pub const PAIR_NUM: usize = 1;              //
pub const MAX_LEN: usize = 5;            //
pub const REQUEST: &str = "send request";   // 测试数据
pub const BUFFER_SIZE: usize = 4096;        // 缓冲区大小

#[no_mangle]
pub fn main() -> i32 {
    let pid = getpid() as usize;
    let mut key: usize = 1;
    for i in 0..PAIR_NUM {
        // 先创建一个管道，客户端先写请求
        let mut fd1 = [0usize; 2];
        pipe(&mut fd1);
        let first_write = fd1[1];
        let mut readi = fd1[0];

        let first_key = key;
        
        for j in 0..MAX_LEN - 1 {
            let mut fd2 = [0usize; 2];
            pipe(&mut fd2);
            let writei = fd2[1];
        
            add_task_with_prority(Box::pin(server(readi, writei, i * MAX_LEN + j, pid, key, key + 1)), 1, pid);
            readi = fd2[0];
            key += 1;
        }
        add_task_with_prority(Box::pin(client(first_write, readi, i * MAX_LEN + MAX_LEN - 1, pid, first_key, key)), 0, pid);
        key += 2;
    }
    user_thread_main(pid);
    0
}



// 服务端接收用户端的请求，从管道中读取内容，然后向客户端写响应内容
async fn server(fd1: usize, fd2: usize, tid: usize, pid: usize, _key1: usize, key2: usize) {
    println!("server read start---");
    let mut buffer = [0u8; BUFFER_SIZE];
    read(fd1, &mut buffer);
    println!("server read end");

    println!("server write start---");
    let resp = REQUEST;
    async_write(fd2, resp.as_bytes().as_ptr() as usize, resp.len(), tid, pid, key2);
    println!("server write end");
}

// 客户端发送请求，向管道中写请求内容，然后读取管道中服务器发送的响应内容
async fn client(fd1: usize, fd2: usize, tid: usize, pid: usize, key1: usize, key2: usize) {
    // 向一个管道中写入数据cd
    println!("client write start---");
    let req = REQUEST;
    async_write(fd1, req.as_bytes().as_ptr() as usize, req.len(), tid, pid, key1);
    println!("client write end");
    // 从另一个管道中异步的读数据
    println!("client read start---");
    let buffer = [0u8; BUFFER_SIZE];
    let ac_r = AsyncCall::new(ASYNC_SYSCALL_READ, fd2, buffer.as_ptr() as usize, buffer.len(), tid, pid, key2);
    ac_r.await;
    print!("------------------buffer: ");
    for c in buffer {
        if c != 0 {
            print!("{}", c as char);
        }
    }
    println!("");
    // println!("client read end");
}