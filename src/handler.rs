use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use crate::error::Result;
use crate::pb::{request, response};
use crate::{Error, Filesystem, ItemType, OpenMode};

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

    pub(super) async fn handle(&mut self, request: request::Content) -> Result<response::Content> {
        Ok(match request {
            request::Content::GetVolumeCapabilities(_) => {
                response::Content::GetVolumeCapabilities(response::GetVolumeCapabilities {
                    capabilities: Some(self.filesystem.get_volume_capabilities().await?),
                })
            }
            request::Content::GetAttributes(msg) => {
                match self.filesystem.get_attributes(msg.file_id).await {
                    Ok(attrs) => response::Content::GetAttributes(response::GetAttributes {
                        attributes: Some(attrs),
                    }),
                    Err(Error::POSIX(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::LookupItem(msg) => {
                match self
                    .filesystem
                    .lookup_item(&OsString::from_vec(msg.name), msg.parent_id)
                    .await
                {
                    Ok((attrs, name)) => response::Content::LookupItem(response::LookupItem {
                        attributes: Some(attrs),
                        name: name.into_vec(),
                    }),
                    Err(Error::POSIX(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::CreateItem(msg) => {
                match self
                    .filesystem
                    .create_item(
                        &OsString::from_vec(msg.name),
                        ItemType::try_from(msg.r#type).unwrap(),
                        msg.parent_id,
                        msg.attributes.unwrap(),
                    )
                    .await
                {
                    Ok((attrs, name)) => response::Content::CreateItem(response::CreateItem {
                        attributes: Some(attrs),
                        name: name.into_vec(),
                    }),
                    Err(Error::POSIX(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::OpenItem(msg) => {
                match self
                    .filesystem
                    .open_item(msg.attributes.unwrap(), to_open_modes(msg.modes))
                    .await
                {
                    Ok(_) => response::Content::Success(response::Success {}),
                    Err(Error::POSIX(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::CloseItem(msg) => {
                match self
                    .filesystem
                    .close_item(msg.attributes.unwrap(), to_open_modes(msg.modes))
                    .await
                {
                    Ok(_) => response::Content::Success(response::Success {}),
                    Err(Error::POSIX(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
        })
    }
}

fn to_open_modes(modes: Vec<i32>) -> Vec<OpenMode> {
    modes
        .iter()
        .map(|&v| OpenMode::try_from(v).unwrap_or_default())
        .collect()
}
