mod args;
mod client;
mod config;

use crate::args::Args;
use crate::client::Client;
use clap::Parser;
use colored::*;

fn main() {
    let client = Client::new();
    let args = Args::parse();
    let prompt = Client::gen_prompt(args);
    let res = client.send_prompt(&prompt).expect("Error getting response");
    println!("{}", res.green());
}
