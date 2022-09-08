mod inode;
mod pipe;
mod stdio;

use crate::mm::UserBuffer;

use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;

    fn aread(&self, buf: UserBuffer, tid: usize, pid: usize, key: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>;

}

pub use inode::{list_apps, open_file, OSInode, OpenFlags};
pub use pipe::{make_pipe, Pipe};
pub use stdio::{Stdin, Stdout};
