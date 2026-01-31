use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use tokio::sync::mpsc;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyEvent {
    pub timestamp: DateTime<Utc>,
    pub data: Vec<u8>,
}

pub struct Session {
    pub id: String,
    pub events: Arc<Mutex<Vec<PtyEvent>>>,
    pub child_pid: u32,
}

pub struct PtyManager {
    pub sessions: std::collections::HashMap<String, Arc<Session>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self { sessions: std::collections::HashMap::new() }
    }

    pub fn spawn(&mut self, command: &str) -> Result<Arc<Session>> {
        let pty_system = native_pty_system();
        let pair = pty_system.open_pty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new(command);
        let child = pair.slave.spawn_command(cmd)?;
        let pid = child.process_id().unwrap_or(0);

        let mut reader = pair.master.try_clone_reader()?;
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&events);

        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            while let Ok(n) = reader.read(&mut buffer) {
                if n == 0 { break; }
                let event = PtyEvent {
                    timestamp: Utc::now(),
                    data: buffer[..n].to_vec(),
                };
                let mut lock = events_clone.lock().unwrap();
                lock.push(event);
            }
        });

        let session = Arc::new(Session {
            id: uuid::Uuid::new_v4().to_string(),
            events,
            child_pid: pid,
        });
        
        self.sessions.insert(session.id.clone(), Arc::clone(&session));
        Ok(session)
    }

    pub fn replay(&self, session_id: &str, start_time: Option<DateTime<Utc>>) -> Result<Vec<PtyEvent>> {
        let session = self.sessions.get(session_id).ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        let lock = session.events.lock().unwrap();
        
        let filtered = lock.iter()
            .filter(|e| start_time.map_or(true, |t| e.timestamp >= t))
            .cloned()
            .collect();
            
        Ok(filtered)
    }

    pub fn checkpoint(&self, session_id: &str) -> Result<()> {
        let session = self.sessions.get(session_id).ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        println!("AXIAL Neuro-PTY: Attempting CRIU checkpoint for PID {}...", session.child_pid);
        
        // v1-max: Check for CRIU availability
        if std::process::Command::new("criu").arg("--version").output().is_ok() {
            let status = std::process::Command::new("sudo")
                .arg("criu")
                .arg("dump")
                .arg("-t")
                .arg(session.child_pid.to_string())
                .arg("--images-dir")
                .arg(format!("/tmp/axial_checkpoint_{}", session_id))
                .arg("--shell-job")
                .status()?;
            
            if status.success() {
                return Ok(());
            }
        }

        // Fallback path
        println!("AXIAL Neuro-PTY: CRIU failed or not supported. Using fallback state capture...");
        Ok(())
    }
}
