use crate::objects::{Kind, Object};
use anyhow::Context;
use std::io;

pub(crate) fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        pretty_print,
        "mode must be given without -p, and we don't support mode"
    );
    let mut object = Object::read(&object_hash).context("parse out blob object file")?;

    match object.kind {
        Kind::Blob => {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();

            let len = io::copy(&mut object.reader, &mut stdout)
                .context("write .git/objects into stdout")?;

            anyhow::ensure!(
                len == object.expected_size,
                ".git/objects file was not the expected size (expected: {}, actual: {})",
                object.expected_size,
                len
            );
            Ok(())
        }
        _ => anyhow::bail!("don't yet know how to print '{}'", object.kind),
    }
}
