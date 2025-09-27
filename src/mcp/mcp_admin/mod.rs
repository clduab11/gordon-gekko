use tokio::process::Command;

// Define action structs in a separate file later
// use crate::mcp::mcp_admin::actions::{FileOperation, WebTask};

/// Tenno-MCP provides unified, administrator-level access to the local machine,
/// combining OS, web, and filesystem operations.
#[derive(Debug, Default)]
pub struct TennoMcp {
    // Future fields for managing playwright instances, etc.
}

impl TennoMcp {
    /// Creates a new instance of Tenno-MCP.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Asynchronously executes a shell command and returns its output.
    ///
    /// # Arguments
    /// * `command_str` - A string representing the shell command to execute.
    ///
    /// # Returns
    /// A `Result` containing the combined stdout and stderr as a `String`,
    /// or an error string if the command fails.
    pub async fn execute_shell(&self, command_str: String) -> Result<String, String> {
        let trimmed = command_str.trim();
        if trimmed.is_empty() {
            return Err("Shell command must not be empty.".to_string());
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(trimmed)
            .output()
            .await
            .map_err(|error| format!("Failed to spawn shell command `{}`: {}", trimmed, error))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut combined = String::new();
        if !stdout.trim().is_empty() {
            combined.push_str(stdout.trim_end_matches('\n'));
        }
        if !stderr.trim().is_empty() {
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(stderr.trim_end_matches('\n'));
        }

        if output.status.success() {
            return Ok(combined);
        }

        let status_message = output
            .status
            .code()
            .map(|code| format!("Command `{}` exited with status code {}", trimmed, code))
            .unwrap_or_else(|| format!("Command `{}` terminated by signal", trimmed));

        if combined.is_empty() {
            Err(status_message)
        } else {
            Err(format!("{}\n{}", status_message, combined))
        }
    }

    // /// Manages a file operation (read, write, delete).
    // pub async fn manage_file(&self, operation: FileOperation) -> Result<(), String> {
    //     // To be implemented in a future step.
    //     unimplemented!();
    // }

    // /// Performs a web task using Playwright.
    // pub async fn perform_web_task(&self, task: WebTask) -> Result<String, String> {
    //     // To be implemented in a future step.
    //     unimplemented!();
    // }
}

#[cfg(test)]
mod tests {
    use super::TennoMcp;

    #[tokio::test]
    async fn execute_shell_returns_stdout_on_success() {
        let admin = TennoMcp::new();
        let result = admin
            .execute_shell("echo success".to_string())
            .await
            .expect("command should succeed");

        assert_eq!(result, "success");
    }

    #[tokio::test]
    async fn execute_shell_returns_error_on_failure() {
        let admin = TennoMcp::new();
        let error = admin
            .execute_shell("exit 5".to_string())
            .await
            .expect_err("command should fail");

        assert!(error.contains("status code 5"));
    }

    #[tokio::test]
    async fn execute_shell_rejects_empty_commands() {
        let admin = TennoMcp::new();
        let error = admin
            .execute_shell("   ".to_string())
            .await
            .expect_err("empty command should be rejected");

        assert!(error.contains("must not be empty"));
    }
}
