pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Mount failed: Unable to complete the request")]
    MountFailed,

    #[error("Unmount failed: Unable to complete the request")]
    UnmountFailed,
}
