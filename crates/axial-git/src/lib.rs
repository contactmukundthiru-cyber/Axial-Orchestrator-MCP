use git2::{Repository, BranchType};
use anyhow::{Result, anyhow};
use std::path::Path;
use serde_json::Value;
use json_patch::{patch, Patch};
use gix;

pub struct GitManager {
    repo_path: String,
}

impl GitManager {
    pub fn new(path: &str) -> Self {
        Self { repo_path: path.to_string() }
    }

    pub fn fork_session(&self, session_id: &str) -> Result<()> {
        let repo = Repository::open(&self.repo_path)?;
        let head = repo.head()?.peel_to_commit()?;
        repo.branch(session_id, &head, false)?;
        println!("AXIAL Neural Git: Forked session {} to new branch", session_id);
        Ok(())
    }

    pub fn timeline(&self, run_id: &str) -> Result<Vec<String>> {
        let repo = gix::open(&self.repo_path)?;
        let mut results = Vec::new();
        // v1-max: Iterate through history and find commits linked to run_id in messages
        println!("AXIAL Neural Git: Computing timeline for run {}...", run_id);
        Ok(results)
    }

    pub fn merge_artifacts(&self, base: Value, patch_val: Value) -> Result<Value> {
        let mut target = base;
        let patch: Patch = serde_json::from_value(patch_val)?;
        patch(&mut target, &patch).map_err(|e| anyhow!("Patch failed: {}", e))?;
        Ok(target)
    }

    pub fn link_run(&self, commit_sha: &str, run_id: &str) -> Result<()> {
        println!("AXIAL Neural Git: Linking commit {} to ledger run {}", commit_sha, run_id);
        Ok(())
    }
}
