mod config;
mod db;
mod server;
mod simulation;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sqlx::PgPool;
use tokio::sync::mpsc::UnboundedSender;

use config::SimConfig;
use server::binary;
use server::messages::ControlMsg;
use server::websocket::{BroadcastTx, WsOut};
use simulation::SimulationState;

/// Persistence jobs handed off from the simulation thread to an async writer,
/// so DB latency never stalls the tick loop.
enum DbJob {
    Checkpoint {
        simulation_id: i32,
        tick: u64,
        blob: Vec<u8>,
        summary: serde_json::Value,
    },
    Stats {
        simulation_id: i32,
        tick: u64,
        total_ants: i32,
        food_collected: f32,
        colony_stats: serde_json::Value,
    },
}

const CHECKPOINT_INTERVAL: Duration = Duration::from_secs(30);
const STATS_INTERVAL: Duration = Duration::from_secs(10);

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("simulator=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let rt = tokio::runtime::Runtime::new()?;
    let handle = rt.handle().clone();

    // ── optional Postgres connection ───────────────────────────────────
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@127.0.0.1:57322/postgres".into());
    let pool: Option<PgPool> = handle.block_on(async {
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_url)
            .await
        {
            Ok(pool) => {
                tracing::info!("Connected to Postgres");
                Some(pool)
            }
            Err(e) => {
                tracing::warn!("Postgres unavailable ({}); running without persistence", e);
                None
            }
        }
    });

    let broadcast_tx = server::create_broadcast();
    let (control_tx, control_rx) = std::sync::mpsc::channel::<ControlMsg>();
    let shutdown = Arc::new(AtomicBool::new(false));

    // ── async persistence writer ───────────────────────────────────────
    let (db_tx, mut db_rx) = tokio::sync::mpsc::unbounded_channel::<DbJob>();
    let writer_handle = pool.clone().map(|pool| {
        rt.spawn(async move {
            while let Some(job) = db_rx.recv().await {
                let result = match job {
                    DbJob::Checkpoint {
                        simulation_id,
                        tick,
                        blob,
                        summary,
                    } => db::save_checkpoint_blob(&pool, simulation_id, tick, &blob, &summary).await,
                    DbJob::Stats {
                        simulation_id,
                        tick,
                        total_ants,
                        food_collected,
                        colony_stats,
                    } => {
                        db::save_stats(
                            &pool,
                            simulation_id,
                            tick,
                            total_ants,
                            food_collected,
                            &colony_stats,
                        )
                        .await
                    }
                };
                if let Err(e) = result {
                    tracing::warn!("DB write failed: {}", e);
                }
            }
        })
    });

    // ── simulation thread ──────────────────────────────────────────────
    let sim_handle = {
        let handle = handle.clone();
        let pool = pool.clone();
        let tx = broadcast_tx.clone();
        let shutdown = Arc::clone(&shutdown);
        let db_tx = if pool.is_some() { Some(db_tx) } else { None };
        std::thread::spawn(move || run_simulation(handle, pool, tx, control_rx, db_tx, shutdown))
    };
    // make sure the writer channel closes once the sim thread drops its sender
    // (main's copy was moved into the thread above)

    // ── WebSocket server (until ctrl-c) ────────────────────────────────
    let server_result = rt.block_on(async {
        tokio::select! {
            res = server::start_server("0.0.0.0:8080", broadcast_tx, control_tx) => res,
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Shutting down, saving final checkpoint...");
                Ok(())
            }
        }
    });

    shutdown.store(true, Ordering::SeqCst);
    let _ = sim_handle.join();

    // wait for the writer to flush the final checkpoint
    if let Some(writer) = writer_handle {
        let _ = rt.block_on(writer);
    }

    server_result
}

/// Load a simulation's config (and checkpoint, if any) from the DB,
/// falling back to a default world when there is no DB or no row.
fn load_simulation(
    handle: &tokio::runtime::Handle,
    pool: Option<&PgPool>,
    simulation_id: i32,
) -> SimulationState {
    let row = pool.and_then(|pool| {
        handle
            .block_on(db::load_simulation_row(pool, simulation_id))
            .ok()
            .flatten()
    });

    let config = match row {
        Some(row) => SimConfig::from_row(row.id, row.world_width, row.world_height, &row.config),
        None => {
            tracing::warn!(
                "Simulation {} not found in DB; using default config",
                simulation_id
            );
            SimConfig {
                simulation_id,
                terrain_seed: simulation_id as u64,
                ..SimConfig::default()
            }
        }
    };

    let checkpoint = pool.and_then(|pool| {
        handle
            .block_on(db::load_latest_checkpoint(pool, simulation_id))
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load checkpoint: {}", e);
                None
            })
    });

    tracing::info!(
        "Starting simulation {}: world={}x{} max_ants={} seed={}{}",
        config.simulation_id,
        config.world_width,
        config.world_height,
        config.max_ants,
        config.terrain_seed,
        if checkpoint.is_some() {
            " (resuming from checkpoint)"
        } else {
            ""
        },
    );

    let mut sim = SimulationState::new(config);
    if let Some(cp) = checkpoint {
        sim.restore_from_checkpoint(cp);
    }
    sim
}

fn initial_simulation_id(handle: &tokio::runtime::Handle, pool: Option<&PgPool>) -> i32 {
    pool.and_then(|pool| {
        handle
            .block_on(db::load_latest_active_simulation(pool))
            .ok()
            .flatten()
            .map(|row| row.id)
    })
    .unwrap_or(1)
}

fn run_simulation(
    handle: tokio::runtime::Handle,
    pool: Option<PgPool>,
    tx: BroadcastTx,
    control_rx: Receiver<ControlMsg>,
    db_tx: Option<UnboundedSender<DbJob>>,
    shutdown: Arc<AtomicBool>,
) {
    let mut sim = load_simulation(&handle, pool.as_ref(), initial_simulation_id(&handle, pool.as_ref()));

    let tick_duration = Duration::from_secs_f64(1.0 / sim.config.tick_rate as f64);
    let mut last_log = Instant::now();
    let mut last_checkpoint = Instant::now();
    let mut last_stats = Instant::now();
    let mut ticks_since_log: u32 = 0;
    let mut tps: f32 = sim.config.tick_rate;

    // broadcast cadence (in ticks)
    const ANTS_EVERY: u64 = 4; // 15 Hz
    const PHEROMONE_EVERY: u64 = 30; // 2 Hz
    const FOOD_EVERY: u64 = 30; // 2 Hz
    const STATS_EVERY: u64 = 60; // 1 Hz

    // absolute schedule so sleep jitter doesn't accumulate into a lower tick rate
    let mut next_tick = Instant::now();

    loop {
        next_tick += tick_duration;

        if shutdown.load(Ordering::SeqCst) {
            send_checkpoint(&sim, &db_tx);
            break;
        }

        // ── control messages from WebSocket clients ────────────────────
        while let Ok(msg) = control_rx.try_recv() {
            match msg {
                ControlMsg::Subscribe { simulation_id } => {
                    if simulation_id != sim.config.simulation_id {
                        // checkpoint the old sim before switching
                        send_checkpoint(&sim, &db_tx);
                        sim = load_simulation(&handle, pool.as_ref(), simulation_id);
                    }
                    let _ = tx.send(WsOut::Binary(Arc::new(binary::encode_init(&sim))));
                }
            }
        }

        sim.tick();
        ticks_since_log += 1;

        // ── broadcasts ─────────────────────────────────────────────────
        if tx.receiver_count() > 0 {
            if sim.tick_count % ANTS_EVERY == 0 {
                let _ = tx.send(WsOut::Binary(Arc::new(binary::encode_ants(&sim))));
            }
            if sim.tick_count % PHEROMONE_EVERY == 0 {
                let _ = tx.send(WsOut::Binary(Arc::new(binary::encode_pheromones(&sim))));
            }
            if sim.tick_count % FOOD_EVERY == 0 {
                let _ = tx.send(WsOut::Binary(Arc::new(binary::encode_food(&sim))));
            }
            if sim.tick_count % STATS_EVERY == 0 {
                let _ = tx.send(WsOut::Text(Arc::new(binary::encode_stats_json(&sim, tps))));
            }
        }

        // ── persistence ────────────────────────────────────────────────
        if db_tx.is_some() && last_checkpoint.elapsed() >= CHECKPOINT_INTERVAL {
            send_checkpoint(&sim, &db_tx);
            last_checkpoint = Instant::now();
        }
        if let Some(db_tx) = &db_tx {
            if last_stats.elapsed() >= STATS_INTERVAL {
                let colony_stats = serde_json::json!(sim
                    .colonies
                    .iter()
                    .map(|c| serde_json::json!({ "id": c.id, "food_stored": c.food_stored }))
                    .collect::<Vec<_>>());
                let _ = db_tx.send(DbJob::Stats {
                    simulation_id: sim.config.simulation_id,
                    tick: sim.tick_count,
                    total_ants: sim.ants.count as i32,
                    food_collected: sim.total_food_collected,
                    colony_stats,
                });
                last_stats = Instant::now();
            }
        }

        // ── logging ────────────────────────────────────────────────────
        if last_log.elapsed() >= Duration::from_secs(1) {
            tps = ticks_since_log as f32 / last_log.elapsed().as_secs_f32();
            ticks_since_log = 0;
            let colony_food: f32 = sim.colonies.iter().map(|c| c.food_stored).sum();
            tracing::info!(
                "sim={} tick={:<8} tps={:.0} ants={} collected={:.0} colony_food={:.0}",
                sim.config.simulation_id,
                sim.tick_count,
                tps,
                sim.ants.count,
                sim.total_food_collected,
                colony_food,
            );
            last_log = Instant::now();
        }

        let now = Instant::now();
        if next_tick > now {
            std::thread::sleep(next_tick - now);
        } else if now - next_tick > Duration::from_millis(250) {
            // fell far behind (e.g. huge frame); don't death-spiral trying to catch up
            next_tick = now;
        }
    }
}

fn send_checkpoint(sim: &SimulationState, db_tx: &Option<UnboundedSender<DbJob>>) {
    let Some(db_tx) = db_tx else { return };
    let cp = sim.to_checkpoint();
    match bincode::serialize(&cp) {
        Ok(blob) => {
            let summary = serde_json::json!({
                "tick": sim.tick_count,
                "ants": sim.ants.count,
                "colonies": sim.colonies.len(),
                "food_collected": sim.total_food_collected,
            });
            let _ = db_tx.send(DbJob::Checkpoint {
                simulation_id: sim.config.simulation_id,
                tick: sim.tick_count,
                blob,
                summary,
            });
        }
        Err(e) => tracing::warn!("Failed to serialize checkpoint: {}", e),
    }
}
