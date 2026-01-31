use std::process::Command;
use anyhow::{Result, anyhow};
use colored::*;

#[derive(Debug, Clone)]
pub struct DoctorCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub install_cmd: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Pass,
    Fail,
    Warn,
}

pub async fn run_checks(all: bool, gpu: bool) -> Vec<DoctorCheck> {
    let mut results = Vec::new();

    // Core Build Tools
    results.push(check_command("rustc", "--version", None));
    results.push(check_command("node", "--version", None));
    results.push(check_command("npm", "--version", None));
    results.push(check_command("cargo", "--version", None));
    results.push(check_command("clang", "--version", Some("sudo apt-get install -y clang")));

    // Containers & Systems
    results.push(check_command("docker", "--version", None));
    results.push(check_command("criu", "--version", Some("sudo apt-get install -y criu")));

    // GUI / Tauri deps
    results.push(check_tauri_deps());

    // AI Providers & Engines
    results.push(check_ollama().await);
    results.push(check_command("goose", "--version", Some("curl -fsSL https://github.com/block/goose/releases/download/stable/install.sh | bash")));
    
    // Truth Engines (Layer 4)
    results.push(check_command("semgrep", "--version", Some("python3 -m pip install semgrep")));
    results.push(check_command("gitleaks", "version", Some("curl -sSfL https://github.com/gitleaks/gitleaks/releases/latest/download/gitleaks-linux-amd64 -o gitleaks && chmod +x gitleaks && sudo mv gitleaks /usr/local/bin/")));
    results.push(check_command("osv-scanner", "--version", Some("go install github.com/google/osv-scanner/cmd/osv-scanner@latest")));
    results.push(check_command("trufflehog", "--version", Some("curl -sSfL https://github.com/trufflesecurity/trufflehog/releases/latest/download/trufflehog_linux_amd64.tar.gz | tar xz && sudo mv trufflehog /usr/local/bin/")));

    // Build & Test (Layer 5)
    results.push(check_command("cargo-nextest", "nextest --version", Some("curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin")));
    results.push(check_command("pytest", "--version", Some("pip install pytest")));
    results.push(check_command("jest", "--version", Some("npm install -g jest")));

    // Tools / Agent CLIs
    results.push(check_command("cursor", "--version", None)); // IDE CLI usually installed by user or manual
    results.push(check_command("aider", "--version", Some("pip install aider-chat")));

    // Bytebot
    results.push(check_bytebot().await);

    if gpu {
        results.push(check_gpu());
    }

    results
}

fn check_command(name: &str, arg: &str, install: Option<&str>) -> DoctorCheck {
    match Command::new(name).arg(arg).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            DoctorCheck {
                name: name.to_string(),
                status: CheckStatus::Pass,
                message: version,
                install_cmd: None,
            }
        }
        _ => DoctorCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: format!("{} not found or failed", name),
            install_cmd: install.map(|s| s.to_string()),
        }
    }
}

fn check_tauri_deps() -> DoctorCheck {
    if cfg!(windows) {
        // Simple check for C++ build tools on Windows
        let output = Command::new("where").arg("cl.exe").output();
        if let Ok(output) = output {
            if output.status.success() {
                return DoctorCheck {
                    name: "Tauri Dependencies (MSVC)".to_string(),
                    status: CheckStatus::Pass,
                    message: "Visual Studio Build Tools (cl.exe) found".to_string(),
                    install_cmd: None,
                };
            }
        }
        return DoctorCheck {
            name: "Tauri Dependencies (MSVC)".to_string(),
            status: CheckStatus::Fail,
            message: "cl.exe not found. Visual Studio C++ Build Tools required.".to_string(),
            install_cmd: Some("winget install Microsoft.VisualStudio.2022.BuildTools --override \"--add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22000 --quiet\"".to_string()),
        };
    }

    let deps = vec!["libwebkit2gtk-4.1-dev", "libgtk-3-dev", "libayatana-appindicator3-dev"];
    let mut missing = Vec::new();
    
    for dep in deps {
        let output = Command::new("dpkg").arg("-s").arg(dep).output();
        if let Ok(output) = output {
            if !output.status.success() {
                missing.push(dep);
            }
        } else {
            return DoctorCheck {
                name: "Tauri Dependencies".to_string(),
                status: CheckStatus::Warn,
                message: "Could not check for dependencies (dpkg missing)".to_string(),
                install_cmd: None,
            };
        }
    }

    if missing.is_empty() {
        DoctorCheck {
            name: "Tauri Dependencies".to_string(),
            status: CheckStatus::Pass,
            message: "All required Tauri dependencies found".to_string(),
            install_cmd: None,
        }
    } else {
        DoctorCheck {
            name: "Tauri Dependencies".to_string(),
            status: CheckStatus::Fail,
            message: format!("Missing: {}", missing.join(", ")),
            install_cmd: Some("sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev".to_string()),
        }
    }
}

async fn check_ollama() -> DoctorCheck {
    let client = reqwest::Client::new();
    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(resp) if resp.status().is_success() => {
            DoctorCheck {
                name: "Ollama".to_string(),
                status: CheckStatus::Pass,
                message: "Ollama service is running".to_string(),
                install_cmd: None,
            }
        }
        _ => DoctorCheck {
            name: "Ollama".to_string(),
            status: CheckStatus::Warn,
            message: "Ollama service not detected at localhost:11434".to_string(),
            install_cmd: Some("curl -fsSL https://ollama.com/install.sh | sh".to_string()),
        }
    }
}

async fn check_bytebot() -> DoctorCheck {
    let client = reqwest::Client::new();
    match client.get("http://localhost:8080/status").send().await {
        Ok(resp) if resp.status().is_success() => {
            DoctorCheck {
                name: "Bytebot".to_string(),
                status: CheckStatus::Pass,
                message: "Bytebot service is running".to_string(),
                install_cmd: None,
            }
        }
        _ => DoctorCheck {
            name: "Bytebot".to_string(),
            status: CheckStatus::Warn,
            message: "Bytebot service not detected at localhost:8080".to_string(),
            install_cmd: None, // Bytebot usually runs in a separate container
        }
    }
}

fn check_gpu() -> DoctorCheck {
    match Command::new("nvidia-smi").output() {
        Ok(output) if output.status.success() => {
            DoctorCheck {
                name: "GPU".to_string(),
                status: CheckStatus::Pass,
                message: "NVIDIA GPU detected".to_string(),
                install_cmd: None,
            }
        }
        _ => DoctorCheck {
            name: "GPU".to_string(),
            status: CheckStatus::Fail,
            message: "No NVIDIA GPU detected or nvidia-smi failed".to_string(),
            install_cmd: None,
        }
    }
}

pub async fn perform_install(results: &[DoctorCheck], noninteractive: bool) -> Result<()> {
    for check in results {
        if check.status != CheckStatus::Pass {
            if let Some(cmd) = &check.install_cmd {
                if !noninteractive {
                    println!("{} Fix {}? [y/N]", "ACTION:".yellow(), check.name);
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if !input.trim().eq_ignore_ascii_case("y") {
                        continue;
                    }
                }
                
                println!("{} Installing {}...", "INFO:".blue(), check.name);
                
                let status = if cfg!(windows) {
                    Command::new("powershell").arg("-Command").arg(cmd).status()?
                } else {
                    Command::new("sh").arg("-c").arg(cmd).status()?
                };

                if !status.success() {
                    return Err(anyhow!("Failed to install {}", check.name));
                }
            }
        }
    }
    Ok(())
}
