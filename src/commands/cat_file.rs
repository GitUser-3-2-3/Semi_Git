use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::{BufRead, BufReader, Read};
use std::{fs, io};

enum Kind {
    Blob,
}

pub(crate) fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        pretty_print,
        "mode must be given without -p, and we don't support mode"
    );
    // TODO -> support shortest-unique object hashes
    let file_path = fs::File::open(format!(
        ".git/objects/{}/{}",
        &object_hash[..2],
        &object_hash[2..]
    ))
    .context("open in .git/objects")?;

    let decoder = ZlibDecoder::new(file_path);

    let mut decoder = BufReader::new(decoder);
    let mut buf = Vec::new();

    decoder
        .read_until(0, &mut buf)
        .context("read header from .git/objects")?;

    let header = CStr::from_bytes_with_nul(&buf)
        .expect("know there is exactly one nul, and it's at the end");

    let header = header
        .to_str()
        .context(".git/objects header is valid UTF-8")?;

    let Some((kind, size)) = header.split_once(" ") else {
        anyhow::bail!(".git/objects header did not start with a known type: '{header}'");
    };

    let kind = match kind {
        "blob" => Kind::Blob,
        _ => anyhow::bail!("'{kind}' is an unsupported type as of yet "),
    };

    let size = size.parse::<usize>().context(format!(
        ".git/objects file header has invalid size: '{size}'"
    ))?;

    // NOTE: decoder.take will not error if the decompressed file is too long, but will at least not
    // spam stdout and be vulnerable to a zip bomb
    let mut decoder = decoder.take(size as u64);

    match kind {
        Kind::Blob => {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();

            let len =
                io::copy(&mut decoder, &mut stdout).context("write .git/objects into stdout")?;

            anyhow::ensure!(
                len == size as u64,
                ".git/objects file was not the expected size (expected: {size}, actual: {len})"
            );
            Ok(())
        }
    }
}
