use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::info;

use crate::simulation::ant::AntStorage;
use crate::simulation::colony::Colony;
use crate::simulation::food::FoodSource;
use crate::simulation::pheromone::PheromoneField;
use crate::simulation::terrain::Terrain;
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
    pub terrain: Terrain,
}

impl SimulationState {
    pub fn to_checkpoint(&self) -> CheckpointData {
        CheckpointData {
            tick_count: self.tick_count,
            total_food_collected: self.total_food_collected,
            ants: self.ants.clone(),
            colonies: self.colonies.clone(),
            food_sources: self.food_sources.clone(),
            pheromones: self.pheromones.clone(),
            terrain: self.terrain.clone(),
        }
    }

    pub fn restore_from_checkpoint(&mut self, cp: CheckpointData) {
        self.tick_count = cp.tick_count;
        self.total_food_collected = cp.total_food_collected;
        self.ants = cp.ants;
        self.colonies = cp.colonies;
        self.food_sources = cp.food_sources;
        self.pheromones = cp.pheromones;
        self.terrain = cp.terrain;
        info!(
            "Restored simulation from checkpoint at tick {}",
            self.tick_count
        );
    }
}

/// A row from the `simulations` table.
#[derive(Debug, sqlx::FromRow)]
pub struct SimulationRow {
    pub id: i32,
    pub world_width: i32,
    pub world_height: i32,
    pub config: serde_json::Value,
}

pub async fn load_simulation_row(
    pool: &PgPool,
    simulation_id: i32,
) -> anyhow::Result<Option<SimulationRow>> {
    let row = sqlx::query_as::<_, SimulationRow>(
        "SELECT id, world_width, world_height, config FROM simulations WHERE id = $1",
    )
    .bind(simulation_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn load_latest_active_simulation(
    pool: &PgPool,
) -> anyhow::Result<Option<SimulationRow>> {
    let row = sqlx::query_as::<_, SimulationRow>(
        "SELECT id, world_width, world_height, config FROM simulations WHERE is_active = true ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn save_checkpoint_blob(
    pool: &PgPool,
    simulation_id: i32,
    tick: u64,
    blob: &[u8],
    summary: &serde_json::Value,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO simulation_checkpoints (simulation_id, tick, state_blob, summary) VALUES ($1, $2, $3, $4)",
    )
    .bind(simulation_id)
    .bind(tick as i64)
    .bind(blob)
    .bind(summary)
    .execute(pool)
    .await?;

    // keep only the 3 most recent checkpoints per simulation
    sqlx::query(
        "DELETE FROM simulation_checkpoints
         WHERE simulation_id = $1
           AND id NOT IN (
               SELECT id FROM simulation_checkpoints
               WHERE simulation_id = $1
               ORDER BY tick DESC LIMIT 3
           )",
    )
    .bind(simulation_id)
    .execute(pool)
    .await?;

    info!(
        "Saved checkpoint: sim={} tick={} size={}KB",
        simulation_id,
        tick,
        blob.len() / 1024
    );
    Ok(())
}

pub async fn save_stats(
    pool: &PgPool,
    simulation_id: i32,
    tick: u64,
    total_ants: i32,
    food_collected: f32,
    colony_stats: &serde_json::Value,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO simulation_stats (simulation_id, tick, total_ants, food_collected, colony_stats) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(simulation_id)
    .bind(tick as i64)
    .bind(total_ants)
    .bind(food_collected)
    .bind(colony_stats)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn load_latest_checkpoint(
    pool: &PgPool,
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
            info!(
                "Loaded checkpoint for sim {} at tick {}",
                simulation_id, cp.tick_count
            );
            Ok(Some(cp))
        }
        None => Ok(None),
    }
}
