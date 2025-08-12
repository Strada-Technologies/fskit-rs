use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use crate::error::Result;
use crate::pb::{PosixError, request, response};
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
            request::Content::GetVolumeCapabilities(_) => {
                response::Content::VolumeCapabilities(self.filesystem.get_volume_capabilities()?)
            }
            request::Content::GetAttributes(msg) => {
                match self.filesystem.get_attributes(msg.file_id) {
                    Ok(attrs) => response::Content::ItemAttributes(attrs),
                    Err(Error::POSIX(code)) => response::Content::PosixError(PosixError { code }),
                    Err(err) => return Err(err),
                }
            }
            request::Content::LookupItem(msg) => {
                match self
                    .filesystem
                    .lookup_item(msg.parent_id, &OsString::from_vec(msg.name))
                {
                    Ok(attrs) => response::Content::ItemAttributes(attrs),
                    Err(Error::POSIX(code)) => response::Content::PosixError(PosixError { code }),
                    Err(err) => return Err(err),
                }
            }
        })
    }
}
