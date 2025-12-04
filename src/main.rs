mod args;
mod client;
mod config;

use crate::args::Args;

use clap::Parser;
use crate::client::Client;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let args = Args::parse();
    let input = args.input;
    println!("Usage {}!", input);
}

