use crate::models::*;
use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Load initial simulation state
    pub async fn load_simulation(&self, simulation_id: i32) -> Result<Simulation> {
        let row = sqlx::query(
            "SELECT * FROM simulations WHERE id = $1 AND is_active = true"
        )
        .bind(simulation_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Simulation {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            world_width: row.get("world_width"),
            world_height: row.get("world_height"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
            simulation_speed: row.get("simulation_speed"),
            current_tick: row.get("current_tick"),
            season: row.get("season"),
            time_of_day: row.get("time_of_day"),
            weather_type: row.get("weather_type"),
            weather_intensity: row.get("weather_intensity"),
        })
    }

    pub async fn load_colonies(&self, simulation_id: i32) -> Result<Vec<Colony>> {
        let rows = sqlx::query(
            "SELECT * FROM colonies WHERE simulation_id = $1 AND is_active = true"
        )
        .bind(simulation_id)
        .fetch_all(&self.pool)
        .await?;

        let mut colonies = Vec::new();
        for row in rows {
            colonies.push(Colony {
                id: row.get("id"),
                simulation_id: row.get("simulation_id"),
                name: row.get("name"),
                center_x: row.get("center_x"),
                center_y: row.get("center_y"),
                radius: row.get("radius"),
                population: row.get("population"),
                color_hue: row.get("color_hue"),
                resources: row.get("resources"),
                nest_level: row.get("nest_level"),
                territory_radius: row.get("territory_radius"),
                aggression_level: row.get("aggression_level"),
                created_at: row.get("created_at"),
                is_active: row.get("is_active"),
            });
        }
        Ok(colonies)
    }

    pub async fn load_ants(&self, simulation_id: i32) -> Result<Vec<Ant>> {
        let rows = sqlx::query(
            r#"
            SELECT a.* FROM ants a
            JOIN colonies c ON a.colony_id = c.id
            WHERE c.simulation_id = $1 AND a.state != 'dead'
            "#
        )
        .bind(simulation_id)
        .fetch_all(&self.pool)
        .await?;

        let mut ants = Vec::new();
        for row in rows {
            ants.push(Ant {
                id: row.get("id"),
                colony_id: row.get("colony_id"),
                ant_type_id: row.get("ant_type_id"),
                position_x: row.get("position_x"),
                position_y: row.get("position_y"),
                angle: row.get("angle"),
                current_speed: row.get("current_speed"),
                health: row.get("health"),
                age_ticks: row.get("age_ticks"),
                state: row.get("state"),
                target_x: row.get("target_x"),
                target_y: row.get("target_y"),
                target_type: row.get("target_type"),
                target_id: row.get("target_id"),
                carried_resources: row.get("carried_resources"),
                traits: row.get("traits"),
                energy: row.get("energy"),
                mood: row.get("mood"),
                last_updated: row.get("last_updated"),
                created_at: row.get("created_at"),
            });
        }
        Ok(ants)
    }

    pub async fn load_ant_types(&self) -> Result<Vec<AntType>> {
        let rows = sqlx::query("SELECT * FROM ant_types")
            .fetch_all(&self.pool)
            .await?;

        let mut ant_types = Vec::new();
        for row in rows {
            ant_types.push(AntType {
                id: row.get("id"),
                name: row.get("name"),
                base_speed: row.get("base_speed"),
                base_strength: row.get("base_strength"),
                base_health: row.get("base_health"),
                base_size: row.get("base_size"),
                lifespan_ticks: row.get("lifespan_ticks"),
                carrying_capacity: row.get("carrying_capacity"),
                role: row.get("role"),
                color_hue: row.get("color_hue"),
                special_abilities: row.get("special_abilities"),
                food_preferences: row.get("food_preferences"),
            });
        }
        Ok(ant_types)
    }

    pub async fn load_food_sources(&self, simulation_id: i32) -> Result<Vec<FoodSource>> {
        let rows = sqlx::query(
            "SELECT * FROM food_sources WHERE simulation_id = $1 AND amount > 0"
        )
        .bind(simulation_id)
        .fetch_all(&self.pool)
        .await?;

        let mut food_sources = Vec::new();
        for row in rows {
            food_sources.push(FoodSource {
                id: row.get("id"),
                simulation_id: row.get("simulation_id"),
                food_type: row.get("food_type"),
                position_x: row.get("position_x"),
                position_y: row.get("position_y"),
                amount: row.get("amount"),
                max_amount: row.get("max_amount"),
                regeneration_rate: row.get("regeneration_rate"),
                discovery_difficulty: row.get("discovery_difficulty"),
                nutritional_value: row.get("nutritional_value"),
                spoilage_rate: row.get("spoilage_rate"),
                is_renewable: row.get("is_renewable"),
                created_at: row.get("created_at"),
            });
        }
        Ok(food_sources)
    }

    // Batch update operations - much more efficient than individual updates
    pub async fn batch_update_ants(&self, ants: &[FastAnt]) -> Result<()> {
        if ants.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            UPDATE ants SET 
                position_x = updates.position_x,
                position_y = updates.position_y,
                angle = updates.angle,
                current_speed = updates.current_speed,
                health = updates.health,
                energy = updates.energy,
                age_ticks = updates.age_ticks,
                state = updates.state,
                last_updated = NOW()
            FROM (VALUES
            "#
        );

        for (i, ant) in ants.iter().enumerate() {
            if i > 0 {
                query_builder.push(", ");
            }

            let state_str = match ant.state {
                AntState::Wandering => "wandering",
                AntState::SeekingFood => "seeking_food",
                AntState::CarryingFood => "carrying_food",
                AntState::Following => "following",
                AntState::Exploring => "exploring",
                AntState::Patrolling => "patrolling",
                AntState::Dead => "dead",
            };

            query_builder.push("(");
            query_builder.push_bind(ant.id);
            query_builder.push(", ");
            query_builder.push_bind(ant.position.0 as i32);
            query_builder.push(", ");
            query_builder.push_bind(ant.position.1 as i32);
            query_builder.push(", ");
            query_builder.push_bind(ant.angle as i32);
            query_builder.push(", ");
            query_builder.push_bind(ant.speed as i32);
            query_builder.push(", ");
            query_builder.push_bind(ant.health);
            query_builder.push(", ");
            query_builder.push_bind(ant.energy);
            query_builder.push(", ");
            query_builder.push_bind(ant.age_ticks);
            query_builder.push(", ");
            query_builder.push_bind(state_str);
            query_builder.push(")");
        }

        query_builder.push(
            r#"
            ) AS updates(id, position_x, position_y, angle, current_speed, health, energy, age_ticks, state)
            WHERE ants.id = updates.id::integer
            "#
        );

        let query = query_builder.build();
        query.execute(&self.pool).await?;

        tracing::info!("Batch updated {} ants", ants.len());
        Ok(())
    }

    pub async fn batch_update_colonies(&self, colonies: &[FastColony]) -> Result<()> {
        if colonies.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            UPDATE colonies SET
                population = updates.population
            FROM (VALUES
            "#
        );

        for (i, colony) in colonies.iter().enumerate() {
            if i > 0 {
                query_builder.push(", ");
            }

            query_builder.push("(");
            query_builder.push_bind(colony.id);
            query_builder.push(", ");
            query_builder.push_bind(colony.population);
            query_builder.push(")");
        }

        query_builder.push(
            r#"
            ) AS updates(id, population)
            WHERE colonies.id = updates.id::integer
            "#
        );

        let query = query_builder.build();
        query.execute(&self.pool).await?;

        tracing::info!("Batch updated {} colonies", colonies.len());
        Ok(())
    }

    pub async fn batch_update_food_sources(&self, food_sources: &[FastFoodSource]) -> Result<()> {
        if food_sources.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            UPDATE food_sources SET
                amount = updates.amount
            FROM (VALUES
            "#
        );

        for (i, food) in food_sources.iter().enumerate() {
            if i > 0 {
                query_builder.push(", ");
            }

            query_builder.push("(");
            query_builder.push_bind(food.id);
            query_builder.push(", ");
            query_builder.push_bind(food.amount);
            query_builder.push(")");
        }

        query_builder.push(
            r#"
            ) AS updates(id, amount)
            WHERE food_sources.id = updates.id::integer
            "#
        );

        let query = query_builder.build();
        query.execute(&self.pool).await?;

        tracing::info!("Batch updated {} food sources", food_sources.len());
        Ok(())
    }

    pub async fn update_simulation_tick(&self, simulation_id: i32, tick: i64) -> Result<()> {
        sqlx::query(
            "UPDATE simulations SET current_tick = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(tick)
        .bind(simulation_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Clean up operations
    pub async fn clean_expired_pheromone_trails(&self) -> Result<i64> {
        let result = sqlx::query(
            "DELETE FROM pheromone_trails WHERE expires_at < NOW() OR strength < 1"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    pub async fn create_initial_colonies(&self, simulation_id: i32, world_width: i32, world_height: i32) -> Result<Vec<Colony>> {
        // Check if colonies already exist
        let existing_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM colonies WHERE simulation_id = $1 AND is_active = true"
        )
        .bind(simulation_id)
        .fetch_one(&self.pool)
        .await?;

        if existing_count > 0 {
            return self.load_colonies(simulation_id).await;
        }

        // Create two initial colonies
        let spacing = (world_width.min(world_height) as f32 * 0.4) as i32;
        let center_x = world_width / 2;
        let center_y = world_height / 2;

        let colonies_data = vec![
            ("Red Colony", center_x - spacing / 2, center_y, 0),
            ("Blue Colony", center_x + spacing / 2, center_y, 240),
        ];

        let mut created_colonies = Vec::new();

        for (name, x, y, hue) in colonies_data {
            let resources = serde_json::json!({
                "seeds": 100,
                "sugar": 50,
                "protein": 25
            });

            let colony_id = sqlx::query_scalar::<_, i32>(
                r#"
                INSERT INTO colonies (simulation_id, name, center_x, center_y, radius, population, color_hue, resources, nest_level, territory_radius, aggression_level, is_active)
                VALUES ($1, $2, $3, $4, 30, 0, $5, $6, 1, 100, 1, true)
                RETURNING id
                "#
            )
            .bind(simulation_id)
            .bind(name)
            .bind(x)
            .bind(y)
            .bind(hue)
            .bind(&resources)
            .fetch_one(&self.pool)
            .await?;

            created_colonies.push(Colony {
                id: colony_id,
                simulation_id,
                name: name.to_string(),
                center_x: x,
                center_y: y,
                radius: 30,
                population: 0,
                color_hue: hue,
                resources,
                nest_level: 1,
                territory_radius: 100,
                aggression_level: 1,
                created_at: None,
                is_active: Some(true),
            });
        }

        tracing::info!("Created {} initial colonies", created_colonies.len());
        Ok(created_colonies)
    }

    pub async fn spawn_ant(&self, colony_id: i32, ant_type_id: i32, position: (f32, f32)) -> Result<i32> {
        let ant_id = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO ants (colony_id, ant_type_id, position_x, position_y, angle, current_speed, health, state, energy, mood)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'wandering', 100, 'neutral')
            RETURNING id
            "#
        )
        .bind(colony_id)
        .bind(ant_type_id)
        .bind(position.0 as i32)
        .bind(position.1 as i32)
        .bind(0) // angle
        .bind(2) // current_speed
        .bind(100) // health
        .fetch_one(&self.pool)
        .await?;

        Ok(ant_id)
    }
} 