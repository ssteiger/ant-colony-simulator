use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::simulation::ant::AntStorage;
use crate::simulation::colony::Colony;
use crate::simulation::food::FoodSource;
use crate::simulation::pheromone::PheromoneField;
use crate::simulation::SimulationState;

/// Serializable snapshot of everything needed to resume a simulation.
#[derive(Serialize, Deserialize)]
pub struct CheckpointData {
    pub tick_count: u64,
    pub total_food_collected: f32,
    pub ants: AntStorage,
    pub colonies: Vec<Colony>,
    pub food_sources: Vec<FoodSource>,
    pub pheromones: PheromoneField,
}

impl SimulationState {
    pub fn to_checkpoint(&self) -> CheckpointData {
        CheckpointData {
            tick_count: self.tick_count,
            total_food_collected: self.total_food_collected,
            ants: bincode_clone_ants(&self.ants),
            colonies: self.colonies.clone(),
            food_sources: self.food_sources.clone(),
            pheromones: bincode_clone_pheromones(&self.pheromones),
        }
    }

    pub fn restore_from_checkpoint(&mut self, cp: CheckpointData) {
        self.tick_count = cp.tick_count;
        self.total_food_collected = cp.total_food_collected;
        self.ants = cp.ants;
        self.colonies = cp.colonies;
        self.food_sources = cp.food_sources;
        self.pheromones = cp.pheromones;
        info!("Restored simulation from checkpoint at tick {}", self.tick_count);
    }
}

fn bincode_clone_ants(a: &AntStorage) -> AntStorage {
    let bytes = bincode::serialize(a).expect("serialize ants");
    bincode::deserialize(&bytes).expect("deserialize ants")
}

fn bincode_clone_pheromones(p: &PheromoneField) -> PheromoneField {
    let bytes = bincode::serialize(p).expect("serialize pheromones");
    bincode::deserialize(&bytes).expect("deserialize pheromones")
}

pub async fn save_checkpoint(
    pool: &sqlx::PgPool,
    simulation_id: i32,
    state: &SimulationState,
) -> anyhow::Result<()> {
    let cp = state.to_checkpoint();
    let blob = bincode::serialize(&cp)?;

    let summary = serde_json::json!({
        "tick": state.tick_count,
        "ants": state.ants.count,
        "colonies": state.colonies.len(),
        "food_collected": state.total_food_collected,
        "colony_food": state.colonies.iter().map(|c| c.food_stored).sum::<f32>(),
    });

    sqlx::query(
        "INSERT INTO simulation_checkpoints (simulation_id, tick, state_blob, summary) VALUES ($1, $2, $3, $4)",
    )
    .bind(simulation_id)
    .bind(state.tick_count as i64)
    .bind(&blob)
    .bind(&summary)
    .execute(pool)
    .await?;

    info!(
        "Saved checkpoint: tick={} size={}KB",
        state.tick_count,
        blob.len() / 1024
    );
    Ok(())
}

pub async fn save_stats(
    pool: &sqlx::PgPool,
    simulation_id: i32,
    state: &SimulationState,
) -> anyhow::Result<()> {
    let colony_stats = serde_json::json!(
        state.colonies.iter().map(|c| {
            serde_json::json!({
                "id": c.id,
                "food_stored": c.food_stored,
            })
        }).collect::<Vec<_>>()
    );

    sqlx::query(
        "INSERT INTO simulation_stats (simulation_id, tick, total_ants, food_collected, colony_stats) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(simulation_id)
    .bind(state.tick_count as i64)
    .bind(state.ants.count as i32)
    .bind(state.total_food_collected)
    .bind(&colony_stats)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_latest_checkpoint(
    pool: &sqlx::PgPool,
    simulation_id: i32,
) -> anyhow::Result<Option<CheckpointData>> {
    let row: Option<(Vec<u8>,)> = sqlx::query_as(
        "SELECT state_blob FROM simulation_checkpoints WHERE simulation_id = $1 ORDER BY tick DESC LIMIT 1",
    )
    .bind(simulation_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((blob,)) => {
            let cp: CheckpointData = bincode::deserialize(&blob)?;
            info!("Loaded checkpoint at tick {}", cp.tick_count);
            Ok(Some(cp))
        }
        None => {
            warn!("No checkpoint found for simulation {}", simulation_id);
            Ok(None)
        }
    }
}
