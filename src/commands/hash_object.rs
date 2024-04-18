use anyhow::Context;
use sha1::{Digest, Sha1};
use std::{io::Write, path::Path};

use crate::objects::Object;

pub(crate) fn invoke(write: bool, file: &Path) -> anyhow::Result<()> {
    let object = Object::blob_from_file(file).context("open blob input file")?;

    let hash = if write {
        object
            .write_to_objects()
            .context("stream file into blob object file")?
    } else {
        object
            .write(std::io::sink())
            .context("stream file into blob object")?
    };

    println!("{}", hex::encode(hash));

    Ok(())
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
