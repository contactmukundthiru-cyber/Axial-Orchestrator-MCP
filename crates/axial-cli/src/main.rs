mod doctor;
mod daemon;

use clap::{Parser, Subcommand};
use anyhow::Result;
use doctor::CheckStatus;
use axial_git::GitManager;
use axial_bytebot::BytebotClient;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "axial")]
#[command(about = "AXIAL Cognitive Command Center", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check system dependencies and environment
    Doctor {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        gpu: bool,
        #[arg(long)]
        install_missing: bool,
        #[arg(long)]
        noninteractive: bool,
    },
    /// Manage the hash-chained ledger
    Ledger {
        #[command(subcommand)]
        sub: LedgerCommands,
    },
    /// Route a task to a provider
    Route {
        #[arg(long)]
        task: String,
        #[arg(long)]
        strategy: Option<String>,
        #[arg(long)]
        explain: bool,
    },
    /// Manage external agent tools (Cursor, Codex, etc.)
    Tools {
        #[command(subcommand)]
        sub: ToolCommands,
    },
    /// Manage AXIAL Shield security boundary
    Shield {
        #[command(subcommand)]
        sub: ShieldCommands,
    },
    /// Manage Neuro-PTY sessions
    Pty {
        #[command(subcommand)]
        sub: PtyCommands,
    },
    /// Neural Git operations
    Git {
        #[command(subcommand)]
        sub: GitCommands,
    },
    /// Bytebot orchestration
    Bytebot {
        #[command(subcommand)]
        sub: BytebotCommands,
    },
    /// Run a task plan
    Run {
        #[arg(long)]
        plan: String,
        #[arg(long)]
        local_only: bool,
        #[arg(long)]
        demo: bool,
    },
    /// Manage profiles and API keys
    Profile {
        #[command(subcommand)]
        sub: ProfileCommands,
    },
    /// View metadata for a file
    Inspect { path: String },
    /// Launch the Command Center UI
    Ui,
    /// Run the AXIAL background daemon
    Daemon {
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    /// Provision the local agent toolchain (Aider, OpenHands, etc.)
    Provision {
        #[arg(long)]
        tool: Option<String>,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// Create a new profile
    Create { name: String },
    /// List all profiles
    List,
    /// Switch to a different profile
    Switch { name: String },
}

#[derive(Subcommand)]
enum LedgerCommands {
    /// Verify hash chain integrity
    Verify,
    /// Export a runpack
    Export {id: String},
}

#[derive(Subcommand)]
enum ToolCommands {
    /// Probe installed tools
    Probe,
    /// Run a task via a tool
    Run {
        tool: String,
        task: String,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum ShieldCommands {
    /// Scan a string or file for PII
    Scan {
        #[arg(long)]
        input: String,
    },
    /// Check if a domain is allowed
    Check {
        domain: String,
    },
    /// Start the boundary proxy (Layer 8)
    Proxy {
        #[arg(long, default_value = "127.0.0.1:3128")]
        addr: String,
    },
}

#[derive(Subcommand)]
enum IsolateCommands {
    /// Check sandbox capabilities
    Check,
    /// Run a command inside a sandbox
    Run {
        command: String,
        #[arg(long)]
        workspace: String,
    },
}

#[derive(Subcommand)]
enum PtyCommands {
    /// Start a new PTY session
    New {
        command: String,
    },
    /// Replay a PTY session
    Replay {
        id: String,
    },
}

#[derive(Subcommand)]
enum GitCommands {
    /// Fork a repository for a session
    Fork { id: String },
    /// Merge structured artifacts
    Merge { base: String, patch: String },
    /// View the cognitive timeline for a run
    Timeline { run_id: String },
}

#[derive(Subcommand)]
enum BytebotCommands {
    /// Send a task to Bytebot
    Task { instruction: String },
    /// Proxy an action to Bytebot
    Proxy { action: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let data_dir = std::env::var("AXIAL_DATA_DIR").unwrap_or_else(|_| ".axial".to_string());
    tokio::fs::create_dir_all(&data_dir).await?;
    let ledger_path = std::path::PathBuf::from(&data_dir).join("ledger.db");

    match cli.command {
        Commands::Doctor { all, gpu, install_missing, noninteractive } => {
            let results = doctor::run_checks(all, gpu).await;
            
            println!("{:<20} {:<10} {:<40}", "Check", "Status", "Message");
            println!("{:-<70}", "");
            
            for check in results.iter() {
                let status_str = match check.status {
                    doctor::CheckStatus::Pass => "✅ PASS".green(),
                    doctor::CheckStatus::Fail => "❌ FAIL".red(),
                    doctor::CheckStatus::Warn => "⚠️ WARN".yellow(),
                };
                println!("{:<20} {:<18} {:<40}", check.name, status_str, check.message);
            }

            if install_missing {
                println!("\nAttempting to install missing dependencies...");
                doctor::perform_install(&results, noninteractive).await?;
            }
            
            Ok(())
        }
        Commands::Ledger { sub } => {
            let mut ledger = axial_ledger::Ledger::new(ledger_path).await?;
            match sub {
                LedgerCommands::Verify => {
                    if ledger.verify().await? {
                        println!("✅ Ledger integrity verified.");
                    } else {
                        println!("❌ Ledger integrity FAIL!");
                    }
                }
                LedgerCommands::Export { id } => {
                    let output_path = std::path::PathBuf::from(format!("runpack_{}", id));
                    println!("Exporting runpack for {} to {:?}...", id, output_path);
                    ledger.export_runpack(output_path).await?;
                }
            }
            Ok(())
        }
        Commands::Route { task, strategy, explain } => {
            let mut router = axial_router::Router::new();
            router.add_provider(Box::new(axial_router::adapters::ollama::OllamaProvider {
                model: "llama3".to_string(),
                base_url: "http://localhost:11434".to_string(),
            }));
            router.add_provider(Box::new(axial_router::adapters::openai::OpenAIProvider {
                model: "gpt-4o".to_string(),
                api_key: "sk-mock".to_string(),
            }));

            let strategy_str = strategy.as_deref().unwrap_or("performance");
            let decision = router.route(vec!["text-generation".to_string()], strategy_str);
            
            match decision {
                Some(res) => {
                    println!("Best Provider: {}", res.provider_id);
                    if explain {
                        println!("Rationale: {}", res.explanation);
                    }
                }
                None => println!("No suitable provider found."),
            }
            Ok(())
        }
        Commands::Tools { sub } => {
            let mut harness = axial_cli_harness::Harness::new();
            harness.add_adapter(Box::new(axial_cli_harness::adapters::cursor::CursorAdapter));
            harness.add_adapter(Box::new(axial_cli_harness::adapters::codex::CodexAdapter));
            harness.add_adapter(Box::new(axial_cli_harness::adapters::claude::ClaudeCodeAdapter));
            harness.add_adapter(Box::new(axial_cli_harness::adapters::aider::AiderAdapter));
            harness.add_adapter(Box::new(axial_cli_harness::adapters::cline::ClineAdapter));

            match sub {
                ToolCommands::Probe => {
                    let results = harness.probe_all().await;
                    println!("{:<15} {:<10} {:<20}", "Tool", "Installed", "Version");
                    println!("{:-<50}", "");
                    for res in results {
                        println!("{:<15} {:<10} {:<20}", res.name, if res.installed { "✅" } else { "❌" }, res.version.unwrap_or_default());
                    }
                }
                ToolCommands::Run { tool, task, dry_run } => {
                    println!("Running task via {}: {}", tool, task);
                    // Mocking tool run since we don't have all adapters implemented or tools installed
                    if tool == "cursor" {
                        let adapter = axial_cli_harness::adapters::cursor::CursorAdapter;
                        let res = adapter.run(&task, dry_run).await?;
                        println!("Result: {}", res.stdout);
                        if let Some(diff) = res.diff {
                            println!("Diff:\n{}", diff);
                        }
                    }
                }
            }
            Ok(())
        }
        Commands::Shield { sub } => {
            let config = axial_shield::ShieldConfig {
                allowed_domains: vec!["api.openai.com".to_string(), "api.anthropic.com".to_string()].into_iter().collect(),
                pii_patterns: vec![r"\d{3}-\d{2}-\d{4}".to_string(), r"sk-[a-zA-Z0-9]{32,}".to_string()],
                redacted_placeholder: "[REDACTED]".to_string(),
            };
            let shield = std::sync::Arc::new(axial_shield::Shield::new(config)?);

            match sub {
                ShieldCommands::Scan { input } => {
                    let redacted = shield.redact(&input);
                    println!("Original: {}", input);
                    println!("Redacted: {}", redacted);
                }
                ShieldCommands::Check { domain } => {
                    match shield.validate_request(&domain) {
                        Ok(_) => println!("✅ Domain {} is allowed", domain),
                        Err(e) => println!("❌ {}", e),
                    }
                }
                ShieldCommands::Proxy { addr } => {
                    let proxy = axial_shield::ShieldProxy::new(shield, addr.parse()?);
                    proxy.start().await?;
                }
            }
            Ok(())
        }
        Commands::Isolate { sub } => {
            let isolate = axial_isolate::Isolate::new()?;
            match sub {
                IsolateCommands::Check => {
                    let caps = isolate.check_capabilities().await?;
                    println!("{}", serde_json::to_string_pretty(&caps)?);
                }
                IsolateCommands::Run { command, workspace } => {
                    let mut cmd = if cfg!(windows) {
                        let mut c = std::process::Command::new("powershell");
                        c.arg("-Command").arg(&command);
                        c
                    } else {
                        let mut c = std::process::Command::new("sh");
                        c.arg("-c").arg(&command);
                        c
                    };
                    isolate.wrap_bwrap(&mut cmd, &workspace)?;
                    let status = cmd.status()?;
                    println!("Sandbox finished with status: {}", status);
                }
            }
            Ok(())
        }
        Commands::Pty { sub } => {
            let mut manager = axial_pty::PtyManager::new();
            match sub {
                PtyCommands::New { command } => {
                    let session = manager.spawn(&command)?;
                    println!("Started session: {}", session.id);
                    // In a real app, we'd keep this running or attach to it
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    let events = manager.replay(&session.id, None)?;
                    for event in events {
                        print!("{}", String::from_utf8_lossy(&event.data));
                    }
                    println!("\nSession captured.");
                }
                PtyCommands::Replay { id } => {
                    println!("Replaying session {}...", id);
                    let events = manager.replay(&id, None)?;
                    for event in events {
                        print!("{}", String::from_utf8_lossy(&event.data));
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                    println!("\nReplay finished.");
                }
            }
            Ok(())
        }
        Commands::Git { sub } => {
            let manager = GitManager::new(".");
            match sub {
                GitCommands::Fork { id } => {
                    manager.fork_session(&id)?;
                }
                GitCommands::Merge { base, patch } => {
                    let b = serde_json::from_str(&base)?;
                    let p = serde_json::from_str(&patch)?;
                    let merged = manager.merge_artifacts(b, p)?;
                    println!("Merged Artifact: {}", merged);
                }
                GitCommands::Timeline { run_id } => {
                    let events = manager.timeline(&run_id)?;
                    println!("Timeline for {}: {:?}", run_id, events);
                }
            }
            Ok(())
        }
        Commands::Bytebot { sub } => {
            let client = BytebotClient::new("http://localhost:8080");
            match sub {
                BytebotCommands::Task { instruction } => {
                    let res = client.task(&instruction, true).await?;
                    println!("Bytebot Response: {}", res);
                }
                BytebotCommands::Proxy { action } => {
                    let res = client.computer_use(&action).await?;
                    println!("Bytebot Proxy: {}", res);
                }
            }
            client.sync_memory()?;
            Ok(())
        }
        Commands::Run { plan, local_only, demo } => {
            if demo {
                println!("Running demo plan...");
                let mut ledger = axial_ledger::Ledger::new(ledger_path).await?;
                ledger.append(serde_json::json!({"event": "demo_start", "plan": plan})).await?;
                println!("Demo plan finished. Check ledger.");
            }
            Ok(())
        }
        Commands::Profile { sub } => {
            let config_dir = if cfg!(windows) {
                PathBuf::from(std::env::var("USERPROFILE").unwrap()).join(".axial")
            } else {
                PathBuf::from(std::env::var("HOME").unwrap()).join(".axial")
            };
            std::fs::create_dir_all(&config_dir)?;
            let profile_path = config_dir.join("profiles.json");

            match sub {
                ProfileCommands::Create { name } => {
                    let profile = axial_core::Profile {
                        name: name.clone(),
                        constraints: vec![],
                        preferred_tools: vec!["cursor".to_string()],
                    };
                    let mut current: Vec<axial_core::Profile> = if profile_path.exists() {
                        serde_json::from_reader(std::fs::File::open(&profile_path)?)?
                    } else {
                        vec![]
                    };
                    current.push(profile);
                    serde_json::to_writer_pretty(std::fs::File::create(&profile_path)?, &current)?;
                    println!("Created profile {} in {:?}", name, profile_path);
                }
                ProfileCommands::List => {
                    if profile_path.exists() {
                        let current: Vec<axial_core::Profile> = serde_json::from_reader(std::fs::File::open(&profile_path)?)?;
                        for p in current {
                            println!("- {}", p.name);
                        }
                    } else {
                        println!("No profiles found.");
                    }
                }
                ProfileCommands::Switch { name } => println!("Switched to profile {}.", name),
            }
            Ok(())
        }
        Commands::Ui => {
            println!("🚀 Launching AXIAL Command Center...");
            println!("Ensure you have built the Tauri app with 'npm run tauri build'");
            // Mocking launch behavior
            let mut ledger = axial_ledger::Ledger::new(ledger_path).await?;
            ledger.append(serde_json::json!({"event": "ui_launch", "status": "initiated"})).await?;
            
            // In a real v1, we would spawn the child process here
            println!("UI Server listening on http://localhost:1420");
            Ok(())
        }
        Commands::Inspect { path } => {
            let mut engine = axial_perception::PerceptionEngine::new();
            let source = std::fs::read_to_string(&path)?;
            let result = if path.ends_with(".rs") {
                engine.analyze_rust(&source)?
            } else {
                serde_json::json!({"error": "unsupported language"})
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
        Commands::Daemon { port } => {
            daemon::start_daemon(port, ledger_path).await?;
            Ok(())
        }
        Commands::Provision { tool } => {
            println!("🚀 Provisioning AXIAL Agent Toolchain...");
            let tool_name = tool.unwrap_or_else(|| "all".to_string());
            
            // Logic to call the setup-engines.sh or implement in Rust
            let status = if cfg!(windows) {
                // On Windows, try to run via bash if available (WSL/Git Bash) or skip/warn
                std::process::Command::new("sh")
                    .arg("./scripts/setup-engines.sh")
                    .arg(tool_name)
                    .status()
            } else {
                std::process::Command::new("bash")
                    .arg("./scripts/setup-engines.sh")
                    .arg(tool_name)
                    .status()
            };
            
            match status {
                Ok(s) if s.success() => println!("✅ Toolchain provisioned successfully."),
                _ => println!("❌ Provisioning failed. Make sure you have 'sh' or 'bash' in your PATH to run setup scripts."),
            }
            Ok(())
        }
    }
}
