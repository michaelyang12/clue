mod args;
mod cache;
mod client;
mod config;
mod context;
mod history;
mod setup;

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

    if args.upgrade {
        upgrade();
        return;
    }

    if args.config {
        setup::run_setup();
        return;
    }

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
    let history = History::load();
    history.add(args.input.clone(), res.clone());

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

    for entry in entries.iter() {
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

const REPO_URL: &str = "https://github.com/michaelyang12/knock.git";
const CARGO_TOML_URL: &str = "https://raw.githubusercontent.com/michaelyang12/knock/master/Cargo.toml";
const LOCAL_VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_remote_version() -> Option<String> {
    let output = Command::new("curl")
        .args(["-sL", CARGO_TOML_URL])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let content = String::from_utf8(output.stdout).ok()?;
    for line in content.lines() {
        if line.starts_with("version") {
            let version = line
                .split('=')
                .nth(1)?
                .trim()
                .trim_matches('"');
            return Some(version.to_string());
        }
    }
    None
}

fn upgrade() {
    println!("{}", "Checking for updates...".cyan());

    match get_remote_version() {
        Some(remote_version) if remote_version == LOCAL_VERSION => {
            println!(
                "{}",
                format!("Already up to date (v{}).", LOCAL_VERSION).green()
            );
            return;
        }
        Some(remote_version) => {
            println!(
                "{}",
                format!("Upgrading from v{} to v{}...", LOCAL_VERSION, remote_version).cyan()
            );
        }
        None => {
            println!("{}", "Could not check remote version, upgrading anyway...".yellow());
        }
    }

    let status = Command::new("cargo")
        .args(["install", "--git", REPO_URL, "--locked", "--force"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("{}", "Upgrade complete!".green());
        }
        Ok(_) => {
            eprintln!("{}", "Upgrade failed".red());
        }
        Err(e) => {
            eprintln!("{}", format!("Failed to run cargo: {}", e).red());
        }
    }
}
