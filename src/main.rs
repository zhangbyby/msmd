use clap::{Parser, Subcommand};

mod init;
mod download;

#[derive(Parser, Debug)]
#[command(author = "baoyun", version = "0.0.1", about = "Download monster-siren musics", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// Init albums and songs meta data
    Init,
    /// Download songs
    Download {
        /// Thread count for download, min(default) 1, max 16
        #[arg(short, long)]
        thread_count: Option<u8>,
    },
}

fn main() {
    let args = Args::parse();
    init::init(&args);
    download::download(&args);
}