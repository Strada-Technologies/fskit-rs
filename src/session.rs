use std::fs::File;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, RwLock};

use tokio::io::Interest;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::error::Error::{MountFailed, UnmountFailed};
use crate::error::Result;
use crate::{Filesystem, path};

//type RequestChannel = (Request, Sender<response::Content>);

/// The session data structure
#[derive(Debug)]
pub struct Session {
    mount_point: Box<Path>,
    inner: Arc<RwLock<SessionInner>>,
}

#[derive(Debug)]
struct SessionInner {
    stop_tx: Sender<()>,
}

impl Session {
    pub(crate) fn new<FS, P>(filesystem: FS, mount_point: P) -> Result<Self>
    where
        FS: Filesystem + Send + 'static,
        P: AsRef<Path>,
    {
        execute_mount(mount_point.as_ref())?;

        let (stop_tx, stop_rx) = mpsc::channel::<()>(1);
        //tokio::spawn(Session::spawn_loop(filesystem, stop_rx));

        Ok(Self {
            mount_point: Box::from(mount_point.as_ref()),
            inner: Arc::new(RwLock::new(SessionInner { stop_tx })),
        })
    }

    async fn spawn_loop<FS>(filesystem: FS, stop_rx: Receiver<()>) -> Result<()>
    where
        FS: Filesystem + Send + 'static,
    {
        let dir = tempfile::tempdir()?;
        let bind_path = dir.path().join("bind_path");

        let listener = UnixListener::bind(bind_path.clone())?;
        println!("📡 Listening on {:?}", bind_path);

        let stream = UnixStream::connect(bind_path).await.unwrap(); // TODO: replace bind_path
        loop {
            let ready = stream
                .ready(Interest::READABLE | Interest::WRITABLE)
                .await?;

            if ready.is_readable() {
                let mut data = vec![0; 1024];
                // Try to read data, this may still fail with `WouldBlock`
                // if the readiness event is a false positive.
                match stream.try_read(&mut data) {
                    Ok(n) => {
                        println!("read {} bytes", n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        //return Err(e.into());
                    }
                }
            }

            if ready.is_writable() {
                // Try to write data, this may still fail with `WouldBlock`
                // if the readiness event is a false positive.
                match stream.try_write(b"hello world") {
                    Ok(n) => {
                        println!("write {} bytes", n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        // return Err(e.into());
                    }
                }
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        execute_unmount(self.mount_point.as_ref()).ok();
    }
}

fn execute_mount(mount_point: &Path) -> Result<()> {
    // Create a blank disk image
    let path = Path::new("/tmp/fskit-rs.dmg");
    if !path.exists() {
        File::create(&path)?;
    }

    // Attach the raw image as a virtual disk
    let params = format!(
        "attach -imagekey diskimage-class=CRawDiskImage -nomount {}",
        path!(path)
    );
    let output = Command::new("hdiutil")
        .args(params.split_whitespace())
        .output()?;
    let device = std::str::from_utf8(&output.stdout).unwrap().trim();

    // Check if already mounted
    let output = Command::new("mount").output()?;
    let out = std::str::from_utf8(&output.stdout).unwrap();
    if out.contains(path!(mount_point)) || out.contains(device) {
        execute_unmount(mount_point)?;
    }

    // Check if the mount point exists
    if !mount_point.exists() {
        std::fs::create_dir(mount_point)?;
    }

    // Mount a filesystem
    let params = format!("-F -t BridgeFS {} {}", device, path!(mount_point));
    if !Command::new("mount")
        .args(params.split_whitespace())
        .status()?
        .success()
    {
        Err(MountFailed)?
    }

    println!(
        "Filesystem mounted - type: BridgeFS, mount point: {} ({})",
        path!(mount_point),
        device
    );

    Ok(())
}

fn execute_unmount(mount_point: &Path) -> Result<()> {
    // Unmount a filesystem
    if !Command::new("umount")
        .args([mount_point])
        .status()?
        .success()
    {
        Err(UnmountFailed)?
    }

    println!(
        "Filesystem unmounted - type: BridgeFS, mount point: {}",
        path!(mount_point)
    );

    Ok(())
}
