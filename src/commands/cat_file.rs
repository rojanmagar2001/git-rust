use anyhow::Context;

use crate::objects::{Kind, Object};

pub(crate) fn invoke(preety_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        preety_print,
        "mode must be given without -p, and we don't support mode"
    );

    let mut object = Object::read(object_hash).context("parse out blob object file")?;

    match object.kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n = std::io::copy(&mut object.reader, &mut stdout)
                .context("write .git/objects file to stdout")?;
            anyhow::ensure!(
                n == object.expected_size,
                ".git/object file was not the expected size (expected: {}, actual: {n})",
                object.expected_size
            );
        }
        _ => anyhow::bail!(
            "We don't support reading objects of type '{}' yet",
            object.kind
        ),
    }

    Ok(())
}
