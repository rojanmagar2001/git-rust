use anyhow::Context;
use clap::Parser;
use flate2::read::ZlibDecoder;
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
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
    // HashObject {
    //     #[clap(short = 'w')]
    //     write: bool,

    //     file: PathBuf,
    // },
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
                .parse::<usize>()
                .context(".git/objects file header has invalid size: {size}")?;

            buf.clear();
            buf.resize(size, 0);
            z.read_exact(&mut buf[..])
                .context("read true contents of .git/objects file")?;

            let n = z
                .read(&mut [0])
                .context("validate EOF in .git/objects file")?;

            anyhow::ensure!(n == 0, ".git/object file had {n} trailing bytes");
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
