use std::path::Path;
use std::time::SystemTime;

pub use crate::error::{Error, Result};
use crate::session::Session;

mod error;
mod mounter;
mod session;
mod socket;

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

pub struct FileAttr {
    pub ino: u64,
    pub size: u64,
    pub blocks: u64,
    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
    pub kind: FileType,
    pub perm: u16,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    RegularFile,
    Directory,
    Symlink,
    BlockDevice,
    CharDevice,
    NamedPipe,
    Socket,
}

pub trait Filesystem {
    fn init(&mut self) -> Result<()>;

    fn lookup(&self, parent: &str, name: &str) -> Result<FileAttr>;

    fn getattr(&self, path: &str) -> Result<FileAttr>;

    fn setattr(&mut self, path: &str, attr: FileAttr) -> Result<()>;

    fn mkdir(&mut self, path: &str, mode: u32) -> Result<()>;

    fn unlink(&mut self, path: &str) -> Result<()>;

    fn rmdir(&mut self, path: &str) -> Result<()>;

    fn rename(&mut self, old: &str, new: &str) -> Result<()>;

    fn open(&self, path: &str, flags: i32) -> Result<()>;

    fn read(&self, path: &str, offset: u64, size: u32) -> Result<Vec<u8>>;

    fn write(&mut self, path: &str, offset: u64, data: &[u8]) -> Result<u32>;

    fn flush(&mut self, path: &str) -> Result<()>;

    fn readdir(&self, path: &str) -> Result<Vec<(String, FileType)>>;

    fn destroy(&mut self) -> Result<()>;
}

/// Mount the given filesystem to the given mountpoint. This function spawns
/// a background thread to handle filesystem operations while being mounted.
/// The returned handle should be stored to reference the mounted filesystem.
/// If it's dropped, the filesystem will be unmounted.
pub async fn mount<FS, P>(filesystem: FS, mount_point: P) -> Result<Session>
where
    FS: Filesystem + Send + 'static,
    P: AsRef<Path>,
{
    Ok(Session::new(filesystem, mount_point).await?)
}

#[macro_export]
macro_rules! path {
    ($arg:expr) => {
        $arg.as_os_str().to_str().unwrap()
    };
}
