pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Socket starting error")]
    SocketFailed,

    #[error("Mount failed: mount point does not exist")]
    MountPoint,

    #[error("Mount failed: unable to complete the request")]
    MountFailed,

    #[error("Unmount failed: unable to complete the request")]
    UnmountFailed,

    #[error("POSIX error: {0}")]
    Posix(std::ffi::c_int),
}
