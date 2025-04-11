use std::error::Error;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use clap::Parser;
use tokio::time;
use tracing::{info, error, warn, Level};
use tracing_subscriber::FmtSubscriber;

use crypto_index_collector::notification::{Notifier, ConsoleNotifier, ScriptNotifier};
use crypto_index_collector::notification::sender::Severity;

/// Supervisor for Crypto Index Collector - Monitors and automatically restarts the main application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Maximum number of restarts in the monitoring period before giving up
    #[arg(long, default_value_t = 5)]
    max_restarts: u32,

    /// Monitoring period in minutes
    #[arg(long, default_value_t = 10)]
    monitoring_period_minutes: u64,

    /// Initial delay before restarting after a failure (in seconds)
    #[arg(long, default_value_t = 5)]
    initial_restart_delay: u64,

    /// Maximum delay between restarts (in seconds)
    #[arg(long, default_value_t = 60)]
    max_restart_delay: u64,

    /// Path to the notification script (if any)
    #[arg(long)]
    notification_script: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse command line arguments
    let args = Args::parse();
    
    // Create notifier
    let notifier: Box<dyn Notifier> = match &args.notification_script {
        Some(script) => Box::new(ScriptNotifier::new(script.clone())),
        None => Box::new(ConsoleNotifier),
    };
    
    info!("[SUPERVISOR] Starting Crypto Index Collector supervisor");
    
    // Track restart attempts
    let mut restart_count = 0;
    let monitoring_start = Instant::now();
    let monitoring_period = Duration::from_secs(args.monitoring_period_minutes * 60);
    
    loop {
        // Reset restart count if monitoring period has elapsed
        if monitoring_start.elapsed() > monitoring_period {
            if restart_count > 0 {
                info!("[SUPERVISOR] Resetting restart counter after monitoring period");
            }
            restart_count = 0;
        }
        
        // Check if we've exceeded the maximum number of restarts
        if restart_count >= args.max_restarts {
            error!("[SUPERVISOR] Exceeded maximum number of restarts ({}) within monitoring period. Giving up.", args.max_restarts);
            notifier.notify(Severity::Critical, "Crypto Index Collector failed to start after multiple attempts")?;
            return Err("Too many restart attempts".into());
        }
        
        // Start the main application
        info!("[SUPERVISOR] Starting Crypto Index Collector");
        
        let status = Command::new("cargo")
            .args(["run", "--bin", "crypto-index-collector"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        
        match status {
            Ok(exit_status) => {
                if exit_status.success() {
                    info!("[SUPERVISOR] Crypto Index Collector exited normally");
                    // If the application exited normally, we're done
                    break;
                } else {
                    // Application crashed or exited with an error
                    restart_count += 1;
                    let exit_code = exit_status.code().unwrap_or(-1);
                    warn!("[SUPERVISOR] Crypto Index Collector failed with exit code: {}", exit_code);
                    
                    // Calculate backoff delay
                    let delay = calculate_backoff_delay(restart_count, args.initial_restart_delay, args.max_restart_delay);
                    
                    // Send notification about the restart
                    let message = format!(
                        "Crypto Index Collector crashed with exit code {}. Restarting in {} seconds (attempt {}/{})",
                        exit_code, delay, restart_count, args.max_restarts
                    );
                    notifier.notify(Severity::Warning, &message)?;
                    
                    info!("[SUPERVISOR] Restarting in {} seconds (attempt {}/{})", 
                          delay, restart_count, args.max_restarts);
                    time::sleep(Duration::from_secs(delay)).await;
                }
            },
            Err(e) => {
                // Failed to start the application
                restart_count += 1;
                error!("[SUPERVISOR] Failed to start Crypto Index Collector: {}", e);
                
                // Calculate backoff delay
                let delay = calculate_backoff_delay(restart_count, args.initial_restart_delay, args.max_restart_delay);
                
                // Send notification about the restart
                let message = format!(
                    "Failed to start Crypto Index Collector: {}. Retrying in {} seconds (attempt {}/{})",
                    e, delay, restart_count, args.max_restarts
                );
                notifier.notify(Severity::Error, &message)?;
                
                info!("[SUPERVISOR] Retrying in {} seconds (attempt {}/{})", 
                      delay, restart_count, args.max_restarts);
                time::sleep(Duration::from_secs(delay)).await;
            }
        }
    }
    
    Ok(())
}

fn calculate_backoff_delay(attempts: u32, base_delay: u64, max_delay: u64) -> u64 {
    // Exponential backoff with a maximum delay
    let delay = base_delay * (1 << attempts.saturating_sub(1));
    delay.min(max_delay)
}
