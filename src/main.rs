use std::{env, fs};

fn main() {
    println!("Logs from your program will appear here.");

    let args: Vec<String> = env::args().collect();

    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master").unwrap();
        println!("Initialized git repository.");
    } else {
        println!("unknown command: {}", args[1]);
    }
}
