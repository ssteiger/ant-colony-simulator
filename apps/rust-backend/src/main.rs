use ant_colony_simulator::{AntColonySimulator};
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
    
    /// Test mode - run without database
    #[arg(short, long)]
    test: bool,
}

fn main() -> Result<()> {
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

    info!("🚀 Starting Ant Colony Simulator (Bevy + Big Brain Backend)");
    info!("📊 Log level: {}", log_level);

    if args.test {
        info!("🧪 Running in test mode - no database required");
        
        // Create a simple test simulation without database
        let mut simulator = AntColonySimulator::new_test()?;
        
        info!("🎮 Starting test simulation...");
        if let Err(e) = simulator.start() {
            tracing::error!("Bevy simulation error: {}", e);
        }
        
        info!("✅ Test completed successfully!");
        return Ok(());
    }

    // Get database URL from argument or environment variable
    let database_url = args.database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL must be provided either as argument or environment variable"))?;

    // Create a simple runtime for database operations
    let rt = tokio::runtime::Runtime::new()?;
    
    // Connect to database
    info!("🔌 Connecting to database...");
    let pool = rt.block_on(PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url))?;

    info!("✅ Database connection established");

    // Get simulation ID
    let simulation_id = if let Some(id) = args.simulation_id {
        id
    } else {
        info!("🔍 No simulation ID provided, looking for active simulation...");
        
        let active_simulation: Option<(i32,)> = rt.block_on(sqlx::query_as(
            "SELECT id FROM simulations WHERE is_active = true ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_optional(&pool))?;

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

    // Create and start Bevy simulator
    info!("🎮 Initializing Bevy simulation: {}", simulation_id);
    let mut simulator = AntColonySimulator::new(pool, simulation_id)?;

    // Handle graceful shutdown with Ctrl+C
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();
    
    // Spawn a thread to handle Ctrl+C
    std::thread::spawn(move || {
        ctrlc::set_handler(move || {
            info!("🛑 Received Ctrl+C, shutting down gracefully...");
            let _ = shutdown_tx.send(());
        }).expect("Error setting Ctrl-C handler");
    });

    // Start the simulation
    info!("🎮 Starting Bevy simulation...");
    if let Err(e) = simulator.start() {
        tracing::error!("Bevy simulation error: {}", e);
    }

    // Wait for shutdown signal
    let _ = shutdown_rx.recv();
    
    info!("👋 Goodbye!");
    Ok(())
} 