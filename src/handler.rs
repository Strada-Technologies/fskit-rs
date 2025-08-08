use crate::Filesystem;
use crate::error::Result;
use crate::pb::{request, response};

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

    pub(super) fn handle(&self, request: request::Content) -> Result<response::Content> {
        Ok(match request {
            request::Content::Capabilities(_) => {
                response::Content::Capabilities(self.filesystem.set_vol_caps()?)
            }
            request::Content::TypeTwo(_) => {
                response::Content::Capabilities(self.filesystem.set_vol_caps()?)
            }
        })
    }
}
