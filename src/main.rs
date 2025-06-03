use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};

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

            buf.clear();
            buf.resize(size, 0);
            decoder
                .read_exact(&mut buf[..])
                .context("read true contents of .git/objects file")?;

            let len = decoder
                .read(&mut [0])
                .context("validate EOF in .git/objects file")?;

            anyhow::ensure!(len == 0, ".git/objects file has {len} trailing bytes");

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            match kind {
                Kind::Blob => stdout
                    .write_all(&buf)
                    .context("write object contents to stdout")?,
            }
        }
    }
    Ok(())
}
