use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::{BufRead, BufReader, Read};
use std::{fs, io};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
}

enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git")?;
            fs::create_dir(".git/objects")?;
            fs::create_dir(".git/refs")?;
            fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
            println!("Initialized git repository in the current directory.");
        }
        Command::CatFile {
            pretty_print: _pretty_print,
            object_hash,
        } => {
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

                    let len = io::copy(&mut decoder, &mut stdout)
                        .context("write .git/objects into stdout")?;

                    anyhow::ensure!(
                        len == size as u64,
                        ".git/objects file was not the expected size (expected: {size}, actual: {len})"
                    )
                }
            }
        }
    }
    Ok(())
}

// one way to limit the size of a reader

/*
struct LimitReader<R> {
    reader: R,
    limit: usize,
}

impl<R> Read for LimitReader<R>
where
    R: Read,
{
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() > self.limit {
            buf = &mut buf[..self.limit + 1];
        }
        let len = self.reader.read(buf)?;
        if len > self.limit {
            return Err(io::Error::new(Other, "too many bytes"));
        }
        self.limit -= len;
        Ok(len)
    }
}
*/
