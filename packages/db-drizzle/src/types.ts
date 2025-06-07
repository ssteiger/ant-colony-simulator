import type { InferInsertModel, InferModel, InferSelectModel } from "drizzle-orm";
import * as schema from "./generated/schema";

// Export the schema itself
export { schema };

// Re-export useful Drizzle utilities
export { eq, and, or, like, not, desc, asc } from "drizzle-orm";

// Export select types (for fetching data)
export type Simulation = InferSelectModel<typeof schema.simulations>;
export type Colony = InferSelectModel<typeof schema.colonies>;
export type Ant = InferSelectModel<typeof schema.ants>;
export type AntType = InferSelectModel<typeof schema.ant_types>;
export type FoodSource = InferSelectModel<typeof schema.food_sources>;
export type PheromoneTrail = InferSelectModel<typeof schema.pheromone_trails>;
export type Obstacle = InferSelectModel<typeof schema.obstacles>;
export type Predator = InferSelectModel<typeof schema.predators>;
export type SimulationEvent = InferSelectModel<typeof schema.simulation_events>;
export type AntInteraction = InferSelectModel<typeof schema.ant_interactions>;
export type AntGenetics = InferSelectModel<typeof schema.ant_genetics>;
export type ColonyUpgrade = InferSelectModel<typeof schema.colony_upgrades>;
export type SimulationStats = InferSelectModel<typeof schema.simulation_stats>;
export type ScenarioTemplate = InferSelectModel<typeof schema.scenario_templates>;
export type Plant = InferSelectModel<typeof schema.plants>;
export type Decomposer = InferSelectModel<typeof schema.decomposers>;
export type Species = InferSelectModel<typeof schema.species>;
export type Disease = InferSelectModel<typeof schema.diseases>;
export type Infection = InferSelectModel<typeof schema.infections>;
export type AntCaste = InferSelectModel<typeof schema.ant_castes>;
export type ColonyRelation = InferSelectModel<typeof schema.colony_relations>;
export type ColonyCulture = InferSelectModel<typeof schema.colony_culture>;
export type EspionageMission = InferSelectModel<typeof schema.espionage_missions>;
export type MigrationPattern = InferSelectModel<typeof schema.migration_patterns>;
export type WaterBody = InferSelectModel<typeof schema.water_bodies>;
export type FireZone = InferSelectModel<typeof schema.fire_zones>;
export type SoilZone = InferSelectModel<typeof schema.soil_zones>;
export type ClimateZone = InferSelectModel<typeof schema.climate_zones>;
export type WeatherSystem = InferSelectModel<typeof schema.weather_systems>;

// Export view types (views are read-only, so no insert types)
export type EcosystemHealth = typeof schema.ecosystem_health.$inferSelect;
export type ColonyPerformance = typeof schema.colony_performance.$inferSelect;

// Export insert types (for creating new records)
export type NewSimulation = InferInsertModel<typeof schema.simulations>;
export type NewColony = InferInsertModel<typeof schema.colonies>;
export type NewAnt = InferInsertModel<typeof schema.ants>;
export type NewAntType = InferInsertModel<typeof schema.ant_types>;
export type NewFoodSource = InferInsertModel<typeof schema.food_sources>;
export type NewPheromoneTrail = InferInsertModel<typeof schema.pheromone_trails>;
export type NewObstacle = InferInsertModel<typeof schema.obstacles>;
export type NewPredator = InferInsertModel<typeof schema.predators>;
export type NewSimulationEvent = InferInsertModel<typeof schema.simulation_events>;
export type NewAntInteraction = InferInsertModel<typeof schema.ant_interactions>;
export type NewAntGenetics = InferInsertModel<typeof schema.ant_genetics>;
export type NewColonyUpgrade = InferInsertModel<typeof schema.colony_upgrades>;
export type NewSimulationStats = InferInsertModel<typeof schema.simulation_stats>;
export type NewScenarioTemplate = InferInsertModel<typeof schema.scenario_templates>;
export type NewPlant = InferInsertModel<typeof schema.plants>;
export type NewDecomposer = InferInsertModel<typeof schema.decomposers>;
export type NewSpecies = InferInsertModel<typeof schema.species>;
export type NewDisease = InferInsertModel<typeof schema.diseases>;
export type NewInfection = InferInsertModel<typeof schema.infections>;
export type NewAntCaste = InferInsertModel<typeof schema.ant_castes>;
export type NewColonyRelation = InferInsertModel<typeof schema.colony_relations>;
export type NewColonyCulture = InferInsertModel<typeof schema.colony_culture>;
export type NewEspionageMission = InferInsertModel<typeof schema.espionage_missions>;
export type NewMigrationPattern = InferInsertModel<typeof schema.migration_patterns>;
export type NewWaterBody = InferInsertModel<typeof schema.water_bodies>;
export type NewFireZone = InferInsertModel<typeof schema.fire_zones>;
export type NewSoilZone = InferInsertModel<typeof schema.soil_zones>;
export type NewClimateZone = InferInsertModel<typeof schema.climate_zones>;
export type NewWeatherSystem = InferInsertModel<typeof schema.weather_systems>;
