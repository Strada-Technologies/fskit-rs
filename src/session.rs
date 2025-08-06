use std::path::Path;

use crate::error::Result;
use crate::mounter::Mount;
use crate::socket::Socket;
use crate::Filesystem;

/// The session data structure
#[derive(Debug)]
pub struct Session {
    socket: Socket,
    mount: Option<Mount>,
}

impl Session {
    pub(super) async fn new<FS, P>(filesystem: FS, mount_point: P) -> Result<Self>
    where
        FS: Filesystem + Send + 'static,
        P: AsRef<Path>,
    {
        let socket = Socket::start().await?;

        let mount = match Mount::mount(mount_point.as_ref().to_path_buf()) {
            Ok(mount) => mount,
            Err(e) => {
                socket.stop().await;
                return Err(e);
            }
        };

        Ok(Self {
            socket,
            mount: Some(mount),
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(mount) = self.mount.take() {
            mount.unmount().unwrap();
        }

        let socket = self.socket.clone();
        tokio::spawn(async move {
            socket.stop().await;
        });
    }
}
