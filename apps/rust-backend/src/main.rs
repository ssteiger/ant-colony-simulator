mod config;
mod db;
mod server;
mod simulation;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use config::SimConfig;
use simulation::ant::AntState;
use simulation::SimulationState;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("simulator=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let config = SimConfig::default();
    tracing::info!(
        "world={}x{} ants={} food_sources={} tick_rate={}",
        config.world_width,
        config.world_height,
        config.initial_ant_count,
        config.food_source_count,
        config.tick_rate,
    );

    let sim = Arc::new(Mutex::new(SimulationState::new(config.clone())));
    let broadcast_tx = server::create_broadcast();

    // simulation thread: tick at 60Hz, broadcast snapshot at ~20Hz
    let sim_handle = {
        let sim = Arc::clone(&sim);
        let tx = broadcast_tx.clone();
        let tick_rate = config.tick_rate;
        let broadcast_interval = 3u64; // broadcast every N ticks (60/3 = 20Hz)

        std::thread::spawn(move || {
            let tick_duration = Duration::from_secs_f64(1.0 / tick_rate as f64);
            let mut last_log = Instant::now();

            loop {
                let frame_start = Instant::now();

                let mut sim = sim.lock().unwrap();
                sim.tick();

                // broadcast snapshot to WebSocket clients
                if sim.tick_count % broadcast_interval == 0 {
                    let has_subscribers = tx.receiver_count() > 0;
                    if has_subscribers {
                        let msg = if sim.tick_count % (broadcast_interval * 20) == 0 {
                            sim.full_snapshot(1)
                        } else {
                            sim.delta_snapshot(1)
                        };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = tx.send(json);
                        }
                    }
                }

                // log stats every second
                if last_log.elapsed() >= Duration::from_secs(1) {
                    let foraging = sim
                        .ants
                        .state
                        .iter()
                        .filter(|&&s| s == AntState::Foraging)
                        .count();
                    let returning = sim.ants.count - foraging;
                    let colony_food: f32 = sim.colonies.iter().map(|c| c.food_stored).sum();
                    let remaining_food: f32 = sim.food_sources.iter().map(|f| f.amount).sum();

                    tracing::info!(
                        "tick={:<6} ants={} foraging={} returning={} collected={:.0} colony_food={:.0} world_food={:.0}",
                        sim.tick_count,
                        sim.ants.count,
                        foraging,
                        returning,
                        sim.total_food_collected,
                        colony_food,
                        remaining_food,
                    );
                    last_log = Instant::now();
                }

                drop(sim);

                let elapsed = frame_start.elapsed();
                if elapsed < tick_duration {
                    std::thread::sleep(tick_duration - elapsed);
                }
            }
        })
    };

    // tokio runtime for the WebSocket server
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        server::start_server("0.0.0.0:8080", broadcast_tx).await
    })?;

    sim_handle.join().unwrap();
    Ok(())
}
