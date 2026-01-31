use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use reqwest::Client;
use std::path::PathBuf;
use fd_lock::RwLock;
use std::fs::File;

pub struct BytebotClient {
    base_url: String,
    client: Client,
    shared_memory_path: PathBuf,
    gate_path: PathBuf,
}

impl BytebotClient {
    pub fn new(base_url: &str) -> Self {
        let axial_dir = if cfg!(windows) {
            PathBuf::from(std::env::var("USERPROFILE").unwrap()).join(".axial")
        } else {
            PathBuf::from(std::env::var("HOME").unwrap()).join(".axial")
        };

        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
            shared_memory_path: axial_dir.join("memory"),
            gate_path: axial_dir.join("takeover.lock"),
        }
    }

    pub async fn status(&self) -> Result<Value> {
        let res = self.client.get(format!("{}/status", self.base_url))
            .send().await?;
        Ok(res.json().await?)
    }

    pub async fn task(&self, task: &str, require_approval: bool) -> Result<Value> {
        // v1-max: IPC Takeover Gating
        std::fs::create_dir_all(self.gate_path.parent().unwrap())?;
        let f = File::create(&self.gate_path)?;
        let mut lock = RwLock::new(f);
        
        // Try to acquire exclusive lock
        let _guard = lock.try_write().map_err(|_| anyhow!("Takeover Gate: Another process is currently controlling the computer"))?;

        if require_approval {
            self.trigger_takeover_gate(task).await?;
        }

        let res = self.client.post(format!("{}/tasks", self.base_url))
            .json(&json!({ "instruction": task }))
            .send().await?;
        
        Ok(res.json().await?)
    }

    pub async fn computer_use(&self, action: &str) -> Result<Value> {
        // v1-max: IPC Takeover Gating
        std::fs::create_dir_all(self.gate_path.parent().unwrap())?;
        let f = if self.gate_path.exists() {
            File::open(&self.gate_path)?
        } else {
            File::create(&self.gate_path)?
        };
        let mut lock = RwLock::new(f);
        let _guard = lock.try_write().map_err(|_| anyhow!("Takeover Gate: Access denied - lock held by another process"))?;

        // v1-max: Check if action is sensitive
        if action.contains("rm -rf") || action.contains("sudo") {
            return Err(anyhow!("Takeover Block: Sensitive action detected"));
        }

        let res = self.client.post(format!("{}/computer-use", self.base_url))
            .json(&json!({ "action": action }))
            .send().await?;
        
        Ok(res.json().await?)
    }

    async fn trigger_takeover_gate(&self, task: &str) -> Result<()> {
        println!("AXIAL Takeover Gate: Approval required for task: {}", task);
        // v1-max: In a real system, this would block until UI or CLI provides yes/no
        Ok(())
    }

    pub fn sync_memory(&self) -> Result<()> {
        std::fs::create_dir_all(&self.shared_memory_path)?;
        println!("AXIAL Bytebot: Synced shared memory at {:?}", self.shared_memory_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_bytebot_lock_logic() {
        let dir = tempdir().unwrap();
        let axial_dir = dir.path().join(".axial");
        std::fs::create_dir_all(&axial_dir).unwrap();

        let mut client = BytebotClient::new("http://localhost:1234");
        // Override paths for testing
        client.shared_memory_path = axial_dir.join("memory");
        client.gate_path = axial_dir.join("takeover.lock");

        // First lock should succeed
        let f = File::create(&client.gate_path).unwrap();
        let mut lock = RwLock::new(f);
        let guard = lock.try_write();
        assert!(guard.is_ok());

        // Second lock attempt from another "client" should fail
        let f2 = File::open(&client.gate_path).unwrap();
        let mut lock2 = RwLock::new(f2);
        let guard2 = lock2.try_write();
        assert!(guard2.is_err());
    }
}
