use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use crate::error::Result;
use crate::pb::{request, response};
use crate::{Error, Filesystem, ItemType, OpenMode, SetXattrPolicy};

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
            request::Content::GetPathConfOperations(_) => response::Content::PathConfOperations(
                self.filesystem.get_path_conf_operations().await?,
            ),
            request::Content::GetVolumeCapabilities(_) => response::Content::VolumeCapabilities(
                self.filesystem.get_volume_capabilities().await?,
            ),
            request::Content::GetAttributes(msg) => {
                match self.filesystem.get_attributes(msg.item_id).await {
                    Ok(attrs) => response::Content::ItemAttributes(attrs),
                    Err(Error::Posix(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::SetAttributes(msg) => {
                match self
                    .filesystem
                    .set_attributes(msg.item_id, msg.attributes.unwrap())
                    .await
                {
                    Ok(attrs) => response::Content::ItemAttributes(attrs),
                    Err(Error::Posix(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::LookupItem(msg) => {
                match self
                    .filesystem
                    .lookup_item(&OsString::from_vec(msg.name), msg.directory_id)
                    .await
                {
                    Ok(item) => response::Content::Item(item),
                    Err(Error::Posix(code)) => {
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
                        msg.directory_id,
                        msg.attributes.unwrap(),
                    )
                    .await
                {
                    Ok(item) => response::Content::Item(item),
                    Err(Error::Posix(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::RemoveItem(msg) => match self
                .filesystem
                .remove_item(msg.item_id, &OsString::from_vec(msg.name), msg.directory_id)
                .await
            {
                Ok(_) => response::Content::Success(response::Success {}),
                Err(Error::Posix(code)) => {
                    response::Content::PosixError(response::PosixError { code })
                }
                Err(err) => return Err(err),
            },
            request::Content::RenameItem(msg) => match self
                .filesystem
                .rename_item(
                    msg.item_id,
                    msg.source_directory_id,
                    &OsString::from_vec(msg.source_name),
                    &OsString::from_vec(msg.destination_name),
                    msg.destination_directory_id,
                    msg.over_item_id,
                )
                .await
            {
                Ok(data) => response::Content::Data(data),
                Err(Error::Posix(code)) => {
                    response::Content::PosixError(response::PosixError { code })
                }
                Err(err) => return Err(err),
            },
            request::Content::EnumerateDirectory(msg) => {
                match self
                    .filesystem
                    .enumerate_directory(msg.directory_id, msg.cookie, msg.verifier)
                    .await
                {
                    Ok(entries) => response::Content::DirectoryEntries(entries),
                    Err(Error::Posix(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::GetXattrOperations(_) => {
                response::Content::XattrOperations(self.filesystem.get_xattr_operations().await?)
            }
            request::Content::GetXattr(msg) => match self
                .filesystem
                .get_xattr(&OsString::from_vec(msg.name), msg.item_id)
                .await
            {
                Ok(data) => response::Content::Data(data),
                Err(Error::Posix(code)) => {
                    response::Content::PosixError(response::PosixError { code })
                }
                Err(err) => return Err(err),
            },
            request::Content::SetXattr(msg) => match self
                .filesystem
                .set_xattr(
                    &OsString::from_vec(msg.name),
                    msg.value,
                    msg.item_id,
                    SetXattrPolicy::try_from(msg.policy).unwrap_or_default(),
                )
                .await
            {
                Ok(_) => response::Content::Success(response::Success {}),
                Err(Error::Posix(code)) => {
                    response::Content::PosixError(response::PosixError { code })
                }
                Err(err) => return Err(err),
            },
            request::Content::GetXattrs(msg) => {
                match self.filesystem.get_xattrs(msg.item_id).await {
                    Ok(xattrs) => response::Content::Xattrs(xattrs),
                    Err(Error::Posix(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::OpenItem(msg) => {
                match self
                    .filesystem
                    .open_item(msg.item_id, to_open_modes(msg.modes))
                    .await
                {
                    Ok(_) => response::Content::Success(response::Success {}),
                    Err(Error::Posix(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::CloseItem(msg) => {
                match self
                    .filesystem
                    .close_item(msg.item_id, to_open_modes(msg.modes))
                    .await
                {
                    Ok(_) => response::Content::Success(response::Success {}),
                    Err(Error::Posix(code)) => {
                        response::Content::PosixError(response::PosixError { code })
                    }
                    Err(err) => return Err(err),
                }
            }
            request::Content::Read(msg) => match self
                .filesystem
                .read(msg.item_id, msg.offset, msg.length)
                .await
            {
                Ok(data) => response::Content::Data(data),
                Err(Error::Posix(code)) => {
                    response::Content::PosixError(response::PosixError { code })
                }
                Err(err) => return Err(err),
            },
            request::Content::Write(msg) => match self
                .filesystem
                .write(msg.contents, msg.item_id, msg.offset)
                .await
            {
                Ok(count) => response::Content::ByteCount(count),
                Err(Error::Posix(code)) => {
                    response::Content::PosixError(response::PosixError { code })
                }
                Err(err) => return Err(err),
            },
        })
    }
}

fn to_open_modes(modes: Vec<i32>) -> Vec<OpenMode> {
    modes
        .iter()
        .map(|&v| OpenMode::try_from(v).unwrap_or_default())
        .collect()
}
