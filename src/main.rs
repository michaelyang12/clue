mod args;
mod cache;
mod client;
mod config;
mod context;
mod history;

use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

use crate::args::Args;
use crate::cache::Cache;
use crate::client::RequestClient;
use crate::config::Config;
use crate::context::ShellContext;
use crate::history::History;
use clap::Parser;
use colored::*;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.history {
        show_history(&args.input);
        return;
    }

    if args.input.is_empty() {
        eprintln!("{}", "Error: Please provide a query".red());
        std::process::exit(1);
    }

    let context = ShellContext::detect();
    let config = Config::load();
    let cache_key = Cache::generate_key(&args.input, &context.os, &context.shell, args.verbose);
    let cache = Cache::load();

    let res = if let Some(cached) = cache.get(&cache_key) {
        cached
    } else {
        let response = RequestClient::new(args.clone(), context, config)
            .make_request()
            .await
            .expect("Error getting response");

        cache.insert(cache_key, response.clone());
        response
    };

    // Save to history
    let mut history = History::load();
    history.add(args.input.clone(), res.clone());
    let _ = history.save();

    println!("{}", &res.bright_green());

    if args.execute {
        print!("{}", "Execute? [y/N] ".yellow());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input).unwrap();

        if input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "---".dimmed());
            execute_command(&res);
        }
    } else if !args.verbose {
        copy_to_clipboard(&res).expect("Error copying to clipboard");
    }
}

fn show_history(filter: &str) {
    let history = History::load();
    let entries = if filter.is_empty() {
        history.recent(20)
    } else {
        history.search(filter)
    };

    if entries.is_empty() {
        println!("{}", "No history found.".dimmed());
        return;
    }

    for entry in entries.iter().rev() {
        println!("{}", entry.query.dimmed());
        println!("  {}", entry.command.bright_green());
    }
}

fn execute_command(cmd: &str) {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let status = Command::new(&shell)
        .arg("-c")
        .arg(cmd)
        .status();

    match status {
        Ok(s) if !s.success() => {
            if let Some(code) = s.code() {
                eprintln!("{}", format!("Command exited with code {}", code).red());
            }
        }
        Err(e) => eprintln!("{}", format!("Failed to execute: {}", e).red()),
        _ => {}
    }
}

fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    let cmd = "pbcopy";

    #[cfg(target_os = "linux")]
    let cmd = "wl-copy";

    let mut child = Command::new(cmd).stdin(Stdio::piped()).spawn()?;

    child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
    child.wait()?;
    Ok(())
}
