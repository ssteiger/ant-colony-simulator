use ant_colony_simulator::simulation::AntColonySimulator;
use anyhow::Result;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Simulation ID to run
    #[arg(short, long)]
    simulation_id: Option<i32>,
    
    /// Database URL
    #[arg(short, long)]
    database_url: Option<String>,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();

    info!("🚀 Starting Ant Colony Simulator (Rust Backend)");
    info!("📊 Log level: {}", log_level);

    // Get database URL from argument or environment variable
    let database_url = args.database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL must be provided either as argument or environment variable"))?;

    // Connect to database
    info!("🔌 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    info!("✅ Database connection established");

    // Get simulation ID
    let simulation_id = if let Some(id) = args.simulation_id {
        id
    } else {
        info!("🔍 No simulation ID provided, looking for active simulation...");
        
        let active_simulation: Option<(i32,)> = sqlx::query_as(
            "SELECT id FROM simulations WHERE is_active = true ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_optional(&pool)
        .await?;

        match active_simulation {
            Some((id,)) => {
                info!("🎯 Found active simulation: {}", id);
                id
            }
            None => {
                return Err(anyhow::anyhow!(
                    "No active simulation found. Please provide a simulation ID or create a simulation first."
                ));
            }
        }
    };

    // Create and start simulator
    info!("🎮 Initializing simulation: {}", simulation_id);
    let mut simulator = AntColonySimulator::new(pool, simulation_id).await?;

    // Handle graceful shutdown
    let mut simulator_handle = tokio::spawn(async move {
        if let Err(e) = simulator.start().await {
            tracing::error!("Simulation error: {}", e);
        }
    });

    // Wait for Ctrl+C
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Received Ctrl+C, shutting down gracefully...");
        }
        _ = &mut simulator_handle => {
            info!("🏁 Simulation completed");
        }
    }

    info!("👋 Goodbye!");
    Ok(())
} 