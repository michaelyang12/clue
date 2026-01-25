use std::env;
use std::process::Command;

pub struct ShellContext {
    pub os: String,
    pub shell: String,
    pub cwd: String,
}

impl ShellContext {
    pub fn detect() -> Self {
        Self {
            os: detect_os(),
            shell: detect_shell(),
            cwd: env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
        }
    }

    pub fn as_prompt_context(&self) -> String {
        format!(
            "<context>\n  <os>{}</os>\n  <shell>{}</shell>\n  <cwd>{}</cwd>\n</context>",
            self.os, self.shell, self.cwd
        )
    }
}

fn detect_os() -> String {
    if cfg!(target_os = "macos") {
        "macOS".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn detect_shell() -> String {
    // Try SHELL env var first (Unix)
    if let Ok(shell_path) = env::var("SHELL") {
        if let Some(shell_name) = shell_path.split('/').last() {
            return shell_name.to_string();
        }
    }

    // Try running shell detection command
    if cfg!(target_os = "windows") {
        // Check if PowerShell or cmd
        if env::var("PSModulePath").is_ok() {
            return "powershell".to_string();
        }
        return "cmd".to_string();
    }

    // Fallback: try to get parent process name
    if let Ok(output) = Command::new("ps")
        .args(["-p", &std::process::id().to_string(), "-o", "ppid="])
        .output()
    {
        if let Ok(ppid) = String::from_utf8_lossy(&output.stdout).trim().parse::<u32>() {
            if let Ok(output) = Command::new("ps")
                .args(["-p", &ppid.to_string(), "-o", "comm="])
                .output()
            {
                let comm = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !comm.is_empty() {
                    return comm.split('/').last().unwrap_or(&comm).to_string();
                }
            }
        }
    }

    "unknown".to_string()
}
