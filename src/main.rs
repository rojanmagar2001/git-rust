use anyhow::Context;
use clap::Parser;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

// Simple program to greet a person
#[derive(Parser, Debug)]
enum Command {
    // Doc Comment
    Init,
    CatFile {
        #[clap(short = 'p')]
        preety_print: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
}

#[derive(Debug)]
enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // You can print statements as follows for debugging, they'll be removed in release builds
    eprintln!("Logs from your program will appear here!");

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "refs: refs/heads/main\n").unwrap();
            println!("Initializing the repository")
        }
        Command::CatFile {
            preety_print,
            object_hash,
        } => {
            anyhow::ensure!(
                preety_print,
                "mode must be given without -p, and we don't support mode"
            );

            // dbg!(preety_print, object_hash);

            let object = fs::File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .context("open in .git/objects")?;

            let z = ZlibDecoder::new(object);
            let mut z = BufReader::new(z);
            let mut buf = Vec::new();

            z.read_until(0, &mut buf)
                .context("read header from .git/objects")?;

            let header = CStr::from_bytes_with_nul(&buf)
                .expect("Know there is exactly one nul, and it's at the end");

            let header = header
                .to_str()
                .context(".git/objects file header isn't valid UTF-8")?;

            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!(
                    "./git/objects file header did not start with a known type: '{header}'"
                )
            };

            let kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("We don't support reading objects of type '{kind}' yet"),
            };

            let size = size
                .parse::<u64>()
                .context(".git/objects file header has invalid size: {size}")?;

            let mut z = z.take(size);

            match kind {
                Kind::Blob => {
                    let stdout = std::io::stdout();
                    let mut stdout = stdout.lock();
                    let n = std::io::copy(&mut z, &mut stdout)
                        .context("write .git/objects file to stdout")?;
                    anyhow::ensure!(n == size, ".git/object file was not the expected size (expected: {size}, actual: {n})");
                }
            }
        }
        Command::HashObject { write, file } => {
            fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
            where
                W: Write,
            {
                let stat =
                    std::fs::metadata(&file).with_context(|| format!("stat {}", file.display()))?;

                let writer = ZlibEncoder::new(writer, Compression::default());
                let mut writer = HashWriter {
                    writer,
                    hasher: Sha1::new(),
                };
                write!(writer, "blob ")?;
                write!(writer, "{}\0", stat.len())?;

                let mut file = std::fs::File::open(&file)
                    .with_context(|| format!("open {}", file.display()))?;

                std::io::copy(&mut file, &mut writer).context("stream file into bob")?;
                let _ = writer.writer.finish()?;
                let hash = writer.hasher.finalize();
                Ok(hex::encode(hash))
            }

            let hash = if write {
                let tmp = "temporary";
                let hash = write_blob(
                    &file,
                    std::fs::File::create(tmp).context("construct remporary file for blob")?,
                )
                .context("write blob object")?;
                fs::create_dir_all(format!(".git/objects/{}", &hash[..2]))
                    .context("create subdir of .git/objects")?;
                std::fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
                    .context("move temporary file to final location")?;
                hash
            } else {
                write_blob(&file, std::io::sink()).context("write out blob object")?
            };

            println!("{}", hash);
        }
    }
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
