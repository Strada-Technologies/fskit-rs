use fskit_rs::{FileAttr, FileType, Filesystem, Result};
use std::path::Path;
use tokio::sync::oneshot;

struct FsHandler;

impl Filesystem for FsHandler {
    fn init(&mut self) -> Result<()> {
        todo!()
    }

    fn lookup(&self, parent: &str, name: &str) -> Result<FileAttr> {
        todo!()
    }

    fn getattr(&self, path: &str) -> Result<FileAttr> {
        todo!()
    }

    fn setattr(&mut self, path: &str, attr: FileAttr) -> Result<()> {
        todo!()
    }

    fn mkdir(&mut self, path: &str, mode: u32) -> Result<()> {
        todo!()
    }

    fn unlink(&mut self, path: &str) -> Result<()> {
        todo!()
    }

    fn rmdir(&mut self, path: &str) -> Result<()> {
        todo!()
    }

    fn rename(&mut self, old: &str, new: &str) -> Result<()> {
        todo!()
    }

    fn open(&self, path: &str, flags: i32) -> Result<()> {
        todo!()
    }

    fn read(&self, path: &str, offset: u64, size: u32) -> Result<Vec<u8>> {
        todo!()
    }

    fn write(&mut self, path: &str, offset: u64, data: &[u8]) -> Result<u32> {
        todo!()
    }

    fn flush(&mut self, path: &str) -> Result<()> {
        todo!()
    }

    fn readdir(&self, path: &str) -> Result<Vec<(String, FileType)>> {
        todo!()
    }

    fn destroy(&mut self) -> Result<()> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    tokio::task::spawn_blocking(move || {
        let handler = FsHandler;

        let mount_point = Path::new("/tmp/fskit-rs-mount_point");

        let session = match fskit_rs::mount(handler, mount_point) {
            Ok(session) => session,
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        };

        futures::executor::block_on(async {
            let _ = stop_rx.await;
        });

        drop(session);
    })
    .await
    .unwrap();
}
