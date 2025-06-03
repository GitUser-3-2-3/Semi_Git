use crate::HashWriter;
use anyhow::Context;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub(crate) fn invoke(write: bool, file: &PathBuf) -> anyhow::Result<()> {
    fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
    where
        W: Write,
    {
        let stat = fs::metadata(&file).with_context(|| format!("stat {}", file.display()))?;
        let writer = ZlibEncoder::new(writer, Compression::default());

        let mut writer = HashWriter {
            writer,
            hasher: Sha1::new(),
        };

        write!(writer, "blob ")?;
        write!(writer, "{}\0", stat.len())?;

        let mut file = fs::File::open(&file).with_context(|| format!("open {}", file.display()))?;
        io::copy(&mut file, &mut writer).context("stream file into blob")?;

        let _ = writer.writer.finish()?;
        let hash = writer.hasher.finalize();
        Ok(hex::encode(hash))
    }

    let hash = if write {
        let tmp = "temporary";
        let hash = write_blob(
            &file,
            fs::File::create(tmp).context("construct temporary file for blob")?,
        )
        .context("write out blob object")?;

        fs::create_dir_all(format!(".git/objects/{}/", &hash[..2]))
            .context("create subdir of .git/objects")?;

        fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("move blob files into .git/objects")?;
        hash
    } else {
        write_blob(&file, io::sink()).context("write out blob object")?
    };
    println!("{hash}");
    Ok(())
}
