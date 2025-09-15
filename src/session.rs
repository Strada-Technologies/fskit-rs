use crate::handler::Handler;
use crate::mounter::Mounter;
use crate::socket::Socket;
use crate::{Filesystem, MountOptions, mounter, socket};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Session {
    socket: Socket,
    mounter: Mounter,
}

impl Session {
    pub(super) async fn new<FS>(fs: FS, opts: MountOptions) -> Result<Self>
    where
        FS: Filesystem + Send + Sync + Clone + 'static,
    {
        let handler = Handler::new(fs);

        let socket = Socket::start(handler, opts.clone()).await?;

        let mounter = match Mounter::mount(opts) {
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
