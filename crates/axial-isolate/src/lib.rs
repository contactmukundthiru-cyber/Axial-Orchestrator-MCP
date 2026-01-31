use std::process::Command;
use anyhow::{Result, anyhow};
use bollard::Docker;

pub struct Isolate {
    pub use_docker: bool,
    pub docker_connection: Option<Docker>,
}

impl Isolate {
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults().ok();
        Ok(Self {
            use_docker: docker.is_some(),
            docker_connection: docker,
        })
    }

    /// Run a command inside a bubblewrap sandbox (Linux only)
    pub fn wrap_bwrap(&self, command: &mut Command, workspace: &str) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let mut bwrap = Command::new("bwrap");
            bwrap
                .arg("--ro-bind").arg("/usr").arg("/usr")
                .arg("--ro-bind").arg("/bin").arg("/bin")
                .arg("--ro-bind").arg("/lib").arg("/lib")
                .arg("--ro-bind").arg("/lib64").arg("/lib64")
                .arg("--dev").arg("/dev")
                .arg("--proc").arg("/proc")
                .arg("--tmpfs").arg("/tmp")
                .arg("--bind").arg(workspace).arg(workspace)
                .arg("--unshare-all")
                .arg("--share-net") // Allow net for now, Shield will block
                .arg("--die-with-parent")
                .arg("--")
                .arg(command.get_program());

            for arg in command.get_args() {
                bwrap.arg(arg);
            }

            *command = bwrap;
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(anyhow!("Bubblewrap isolation only supported on Linux. Use Docker profile on Windows/macOS."))
        }
    }

    pub async fn check_capabilities(&self) -> Result<serde_json::Value> {
        let mut caps = serde_json::json!({
            "docker": self.use_docker,
            "bwrap": false,
            "seccomp": false,
            "cgroups_v2": false,
        });

        #[cfg(target_os = "linux")]
        {
            caps["bwrap"] = Command::new("bwrap").arg("--version").output().is_ok().into();
            caps["seccomp"] = std::path::Path::new("/proc/sys/kernel/seccomp").exists().into();
            caps["cgroups_v2"] = std::path::Path::new("/sys/fs/cgroup/cgroup.controllers").exists().into();
        }

        Ok(caps)
    }
}
