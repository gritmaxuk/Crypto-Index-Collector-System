use std::process::Command;
use tracing::{info, error};
use crate::error::AppResult;

/// Severity level for notifications
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
    /// Critical error message
    Critical,
}

/// Trait for notification senders
pub trait Notifier {
    /// Send a notification
    fn notify(&self, severity: Severity, message: &str) -> AppResult<()>;
}

/// Console notifier that logs messages to the console
pub struct ConsoleNotifier;

impl Notifier for ConsoleNotifier {
    fn notify(&self, severity: Severity, message: &str) -> AppResult<()> {
        match severity {
            Severity::Info => info!("[NOTIFICATION] {}", message),
            Severity::Warning => info!("[WARNING] {}", message),
            Severity::Error => error!("[ERROR] {}", message),
            Severity::Critical => error!("[CRITICAL] {}", message),
        }
        Ok(())
    }
}

/// Script notifier that executes an external script
pub struct ScriptNotifier {
    script_path: String,
}

impl ScriptNotifier {
    /// Create a new script notifier
    pub fn new(script_path: String) -> Self {
        Self { script_path }
    }
}

impl Notifier for ScriptNotifier {
    fn notify(&self, severity: Severity, message: &str) -> AppResult<()> {
        // Log the notification message
        match severity {
            Severity::Info => info!("[NOTIFICATION] {}", message),
            Severity::Warning => info!("[WARNING] {}", message),
            Severity::Error => error!("[ERROR] {}", message),
            Severity::Critical => error!("[CRITICAL] {}", message),
        }
        
        // Format the message with severity prefix
        let prefixed_message = match severity {
            Severity::Info => format!("INFO: {}", message),
            Severity::Warning => format!("WARNING: {}", message),
            Severity::Error => format!("ERROR: {}", message),
            Severity::Critical => format!("CRITICAL: {}", message),
        };
        
        // Execute the script
        match Command::new(&self.script_path)
            .arg(&prefixed_message)
            .status() {
                Ok(_) => info!("[NOTIFICATION] Script executed successfully"),
                Err(e) => error!("[NOTIFICATION] Failed to execute script: {}", e),
            }
        
        Ok(())
    }
}
