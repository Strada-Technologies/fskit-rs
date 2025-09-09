use std::path::Path;

use crate::handler::Handler;
use crate::mounter::Mounter;
use crate::socket::Socket;
use crate::{Filesystem, mounter, socket};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Session {
    socket: Socket,
    mounter: Mounter,
}

impl Session {
    pub(super) async fn new<FS, P>(filesystem: FS, fs_type: &str, mount_point: P) -> Result<Self>
    where
        FS: Filesystem + Send + Sync + Clone + 'static,
        P: AsRef<Path>,
    {
        let handler = Handler::new(filesystem);

        let socket = Socket::start(handler).await?;

        let mounter = match Mounter::mount(fs_type, mount_point.as_ref().to_path_buf()) {
            Ok(mount) => mount,
            Err(err) => {
                socket.stop().await;
                return Err(Error::Mounter(err));
            }
        };

        Ok(Self { socket, mounter })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.mounter.unmount().inspect_err(|err| eprintln!("{err}"));

        futures::executor::block_on(async {
            self.socket.stop().await;
        });
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Socket(#[from] socket::Error),

    #[error(transparent)]
    Mounter(#[from] mounter::Error),
}
