use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use crate::error::Result;
use crate::pb::{request, response};
use crate::{Error, Filesystem};

#[derive(Debug)]
pub(super) struct Handler<FS>
where
    FS: Filesystem + Send + Sync + 'static,
{
    filesystem: FS,
}

impl<FS> Handler<FS>
where
    FS: Filesystem + Send + Sync + 'static,
{
    pub(super) fn new(filesystem: FS) -> Self {
        Self { filesystem }
    }

    pub(super) fn handle(&mut self, request: request::Content) -> Result<response::Content> {
        Ok(match request {
            request::Content::SetVolCaps(_) => {
                response::Content::VolumeCapabilities(self.filesystem.set_vol_caps()?)
            }
            request::Content::Lookup(msg) => {
                match self
                    .filesystem
                    .lookup(msg.parent, &OsString::from_vec(msg.name))
                {
                    Ok(attrs) => response::Content::ItemAttributes(attrs),
                    Err(Error::POSIX(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
        })
    }
}
