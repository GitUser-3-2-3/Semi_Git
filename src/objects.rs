use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Read};
use std::{fmt, fs};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) reader: R,
    pub(crate) expected_size: u64,
}

impl Object<()> {
    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let file_path = fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
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
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => anyhow::bail!("what even is a {kind} object?"),
        };

        let size = size.parse::<usize>().context(format!(
            ".git/objects file header has invalid size: '{size}'"
        ))?;

        let decoder = decoder.take(size as u64);
        Ok(Object {
            reader: decoder,
            kind,
            expected_size: size as u64,
        })
    }
}
