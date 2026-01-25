use crate::config::{Config, Provider};
use colored::*;
use std::io::{self, BufRead, Write};
use std::process::Command;

const OPENAI_MODELS: &[&str] = &[
    "gpt-4o-mini",
    "gpt-4o",
    "gpt-5.1",
    "o1-mini",
    "o1-preview",
];

const ANTHROPIC_MODELS: &[&str] = &[
    "claude-sonnet-4-20250514",
    "claude-opus-4-20250514",
    "claude-3-5-haiku-20241022",
];

pub fn run_setup() {
    println!("{}", "knock configuration\n".bold());

    // Show current config
    let current = Config::load();
    let current_provider = match current.provider {
        Provider::OpenAI => "openai",
        Provider::Anthropic => "anthropic",
        Provider::Ollama => "ollama",
    };
    let current_model = match current.provider {
        Provider::OpenAI => current.openai_model(),
        Provider::Anthropic => current.anthropic_model(),
        Provider::Ollama => current.ollama_model(),
    };
    println!("Current: {} / {}\n", current_provider.cyan(), current_model.cyan());

    // Select provider
    println!("Select provider:");
    let providers = [
        ("OpenAI", Provider::OpenAI),
        ("Anthropic", Provider::Anthropic),
        ("Ollama (local)", Provider::Ollama),
    ];
    for (i, (name, p)) in providers.iter().enumerate() {
        let marker = if std::mem::discriminant(p) == std::mem::discriminant(&current.provider) {
            " ←".cyan().to_string()
        } else {
            String::new()
        };
        println!("  {}. {}{}", i + 1, name, marker);
    }
    print!("\n> ");
    io::stdout().flush().unwrap();

    let provider_choice = read_line();
    let provider = match provider_choice.trim() {
        "1" => Provider::OpenAI,
        "2" => Provider::Anthropic,
        "3" => Provider::Ollama,
        _ => {
            eprintln!("{}", "Invalid choice".red());
            return;
        }
    };

    // Check API key for cloud providers
    match &provider {
        Provider::OpenAI => {
            if std::env::var("OPENAI_API_KEY").is_err() {
                eprintln!("\n{}", "Warning: OPENAI_API_KEY not set".yellow());
                eprintln!("Add to your shell profile:");
                eprintln!("  export OPENAI_API_KEY=\"your_key_here\"\n");
            } else {
                println!("\n{}", "OPENAI_API_KEY found".green());
            }
        }
        Provider::Anthropic => {
            if std::env::var("ANTHROPIC_API_KEY").is_err() {
                eprintln!("\n{}", "Warning: ANTHROPIC_API_KEY not set".yellow());
                eprintln!("Add to your shell profile:");
                eprintln!("  export ANTHROPIC_API_KEY=\"your_key_here\"\n");
            } else {
                println!("\n{}", "ANTHROPIC_API_KEY found".green());
            }
        }
        Provider::Ollama => {
            // Check if Ollama is running
            if !check_ollama_running() {
                eprintln!("\n{}", "Warning: Ollama doesn't appear to be running".yellow());
                eprintln!("Start it with: ollama serve\n");
            } else {
                println!("\n{}", "Ollama is running".green());
            }
        }
    }

    // Select model
    let model = select_model(&provider, &current);

    // Build and save config
    let mut config = Config::default();
    config.provider = provider.clone();

    match provider {
        Provider::OpenAI => config.openai_model = model,
        Provider::Anthropic => config.anthropic_model = model,
        Provider::Ollama => config.ollama_model = model,
    }

    if let Err(e) = config.save() {
        eprintln!("{}", format!("Failed to save config: {}", e).red());
        return;
    }

    println!("\n{}", "Configuration saved to ~/.knock/config.json".green());
}

fn select_model(provider: &Provider, current_config: &Config) -> Option<String> {
    let models: Vec<String> = match provider {
        Provider::OpenAI => OPENAI_MODELS.iter().map(|s| s.to_string()).collect(),
        Provider::Anthropic => ANTHROPIC_MODELS.iter().map(|s| s.to_string()).collect(),
        Provider::Ollama => {
            print!("Fetching models...");
            io::stdout().flush().unwrap();
            let m = get_ollama_models();
            print!("\r                   \r");
            m
        }
    };

    if models.is_empty() {
        eprintln!("{}", "No models available".yellow());
        return None;
    }

    let current_model = match provider {
        Provider::OpenAI => current_config.openai_model(),
        Provider::Anthropic => current_config.anthropic_model(),
        Provider::Ollama => current_config.ollama_model(),
    };

    println!("\nSelect model:");
    for (i, model) in models.iter().enumerate() {
        let marker = if model == current_model {
            " ←".cyan().to_string()
        } else if i == 0 {
            " (default)".dimmed().to_string()
        } else {
            String::new()
        };
        println!("  {}. {}{}", i + 1, model, marker);
    }
    print!("\n> ");
    io::stdout().flush().unwrap();

    let choice = read_line();
    let choice = choice.trim();

    if choice.is_empty() {
        return None; // Use default
    }

    match choice.parse::<usize>() {
        Ok(n) if n >= 1 && n <= models.len() => Some(models[n - 1].clone()),
        _ => {
            eprintln!("{}", "Invalid choice, using default".yellow());
            None
        }
    }
}

fn get_ollama_models() -> Vec<String> {
    let output = Command::new("ollama")
        .arg("list")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .skip(1) // Skip header
                .filter_map(|line| {
                    line.split_whitespace().next().map(|s| s.to_string())
                })
                .collect()
        }
        _ => vec![]
    }
}

fn check_ollama_running() -> bool {
    reqwest::blocking::get("http://localhost:11434/api/tags").is_ok()
}

fn read_line() -> String {
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).unwrap();
    input
}
