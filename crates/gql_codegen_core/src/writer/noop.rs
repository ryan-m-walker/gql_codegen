use std::io;
use std::path::Path;

use crate::Writer;

pub struct NoopWriter;

impl Writer for NoopWriter {
    fn matches_existing(&self, _path: &std::path::Path, _content: &[u8]) -> bool {
        false
    }

    fn write(&self, _path: &Path, _content: &[u8]) -> io::Result<()> {
        Ok(())
    }
}
