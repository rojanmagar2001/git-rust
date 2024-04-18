use clap::Parser;
use std::{fs, path::PathBuf};

pub(crate) mod commands;
pub(crate) mod objects;

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
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_hash: String,
    },
    WriteTree,
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
        } => commands::cat_file::invoke(preety_print, &object_hash)?,

        Command::HashObject { write, file } => commands::hash_object::invoke(write, &file)?,
        Command::LsTree {
            name_only,
            tree_hash,
        } => commands::ls_tree::invoke(name_only, &tree_hash)?,
        Command::WriteTree => commands::write_tree::invoke()?,
    }
    Ok(())
}
