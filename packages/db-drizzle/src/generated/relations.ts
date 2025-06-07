import { relations } from "drizzle-orm/relations";
import { simulations, colonies, ants, ant_types, food_sources, pheromone_trails, obstacles, predators, simulation_events, ant_interactions, ant_genetics, colony_upgrades, simulation_stats, plants, decomposers, species, diseases, infections, colony_relations, colony_culture, espionage_missions, migration_patterns, water_bodies, fire_zones, soil_zones, climate_zones, weather_systems } from "./schema";

export const coloniesRelations = relations(colonies, ({one, many}) => ({
	simulation: one(simulations, {
		fields: [colonies.simulation_id],
		references: [simulations.id]
	}),
	ants: many(ants),
	pheromone_trails: many(pheromone_trails),
	colony_upgrades: many(colony_upgrades),
	simulation_stats: many(simulation_stats),
	colony_relations_colony1_id: many(colony_relations, {
		relationName: "colony_relations_colony1_id_colonies_id"
	}),
	colony_relations_colony2_id: many(colony_relations, {
		relationName: "colony_relations_colony2_id_colonies_id"
	}),
	colony_cultures: many(colony_culture),
	espionage_missions_origin_colony_id: many(espionage_missions, {
		relationName: "espionage_missions_origin_colony_id_colonies_id"
	}),
	espionage_missions_target_colony_id: many(espionage_missions, {
		relationName: "espionage_missions_target_colony_id_colonies_id"
	}),
	migration_patterns: many(migration_patterns),
}));

export const simulationsRelations = relations(simulations, ({many}) => ({
	colonies: many(colonies),
	food_sources: many(food_sources),
	obstacles: many(obstacles),
	predators: many(predators),
	simulation_events: many(simulation_events),
	simulation_stats: many(simulation_stats),
	plants: many(plants),
	decomposers: many(decomposers),
	species: many(species),
	diseases: many(diseases),
	water_bodies: many(water_bodies),
	fire_zones: many(fire_zones),
	soil_zones: many(soil_zones),
	climate_zones: many(climate_zones),
	weather_systems: many(weather_systems),
}));

export const antsRelations = relations(ants, ({one, many}) => ({
	colony: one(colonies, {
		fields: [ants.colony_id],
		references: [colonies.id]
	}),
	ant_type: one(ant_types, {
		fields: [ants.ant_type_id],
		references: [ant_types.id]
	}),
	pheromone_trails: many(pheromone_trails),
	predators: many(predators),
	ant_interactions_ant1_id: many(ant_interactions, {
		relationName: "ant_interactions_ant1_id_ants_id"
	}),
	ant_interactions_ant2_id: many(ant_interactions, {
		relationName: "ant_interactions_ant2_id_ants_id"
	}),
	ant_genetics_ant_id: many(ant_genetics, {
		relationName: "ant_genetics_ant_id_ants_id"
	}),
	ant_genetics_parent1_id: many(ant_genetics, {
		relationName: "ant_genetics_parent1_id_ants_id"
	}),
	ant_genetics_parent2_id: many(ant_genetics, {
		relationName: "ant_genetics_parent2_id_ants_id"
	}),
	espionage_missions: many(espionage_missions),
}));

export const ant_typesRelations = relations(ant_types, ({many}) => ({
	ants: many(ants),
}));

export const food_sourcesRelations = relations(food_sources, ({one, many}) => ({
	simulation: one(simulations, {
		fields: [food_sources.simulation_id],
		references: [simulations.id]
	}),
	pheromone_trails: many(pheromone_trails),
}));

export const pheromone_trailsRelations = relations(pheromone_trails, ({one}) => ({
	colony: one(colonies, {
		fields: [pheromone_trails.colony_id],
		references: [colonies.id]
	}),
	ant: one(ants, {
		fields: [pheromone_trails.source_ant_id],
		references: [ants.id]
	}),
	food_source: one(food_sources, {
		fields: [pheromone_trails.target_food_id],
		references: [food_sources.id]
	}),
}));

export const obstaclesRelations = relations(obstacles, ({one}) => ({
	simulation: one(simulations, {
		fields: [obstacles.simulation_id],
		references: [simulations.id]
	}),
}));

export const predatorsRelations = relations(predators, ({one}) => ({
	simulation: one(simulations, {
		fields: [predators.simulation_id],
		references: [simulations.id]
	}),
	ant: one(ants, {
		fields: [predators.target_ant_id],
		references: [ants.id]
	}),
}));

export const simulation_eventsRelations = relations(simulation_events, ({one}) => ({
	simulation: one(simulations, {
		fields: [simulation_events.simulation_id],
		references: [simulations.id]
	}),
}));

export const ant_interactionsRelations = relations(ant_interactions, ({one}) => ({
	ant_ant1_id: one(ants, {
		fields: [ant_interactions.ant1_id],
		references: [ants.id],
		relationName: "ant_interactions_ant1_id_ants_id"
	}),
	ant_ant2_id: one(ants, {
		fields: [ant_interactions.ant2_id],
		references: [ants.id],
		relationName: "ant_interactions_ant2_id_ants_id"
	}),
}));

export const ant_geneticsRelations = relations(ant_genetics, ({one}) => ({
	ant_ant_id: one(ants, {
		fields: [ant_genetics.ant_id],
		references: [ants.id],
		relationName: "ant_genetics_ant_id_ants_id"
	}),
	ant_parent1_id: one(ants, {
		fields: [ant_genetics.parent1_id],
		references: [ants.id],
		relationName: "ant_genetics_parent1_id_ants_id"
	}),
	ant_parent2_id: one(ants, {
		fields: [ant_genetics.parent2_id],
		references: [ants.id],
		relationName: "ant_genetics_parent2_id_ants_id"
	}),
}));

export const colony_upgradesRelations = relations(colony_upgrades, ({one}) => ({
	colony: one(colonies, {
		fields: [colony_upgrades.colony_id],
		references: [colonies.id]
	}),
}));

export const simulation_statsRelations = relations(simulation_stats, ({one}) => ({
	simulation: one(simulations, {
		fields: [simulation_stats.simulation_id],
		references: [simulations.id]
	}),
	colony: one(colonies, {
		fields: [simulation_stats.dominant_colony_id],
		references: [colonies.id]
	}),
}));

export const plantsRelations = relations(plants, ({one}) => ({
	simulation: one(simulations, {
		fields: [plants.simulation_id],
		references: [simulations.id]
	}),
}));

export const decomposersRelations = relations(decomposers, ({one}) => ({
	simulation: one(simulations, {
		fields: [decomposers.simulation_id],
		references: [simulations.id]
	}),
}));

export const speciesRelations = relations(species, ({one}) => ({
	simulation: one(simulations, {
		fields: [species.simulation_id],
		references: [simulations.id]
	}),
}));

export const diseasesRelations = relations(diseases, ({one, many}) => ({
	simulation: one(simulations, {
		fields: [diseases.simulation_id],
		references: [simulations.id]
	}),
	infections: many(infections),
}));

export const infectionsRelations = relations(infections, ({one}) => ({
	disease: one(diseases, {
		fields: [infections.disease_id],
		references: [diseases.id]
	}),
}));

export const colony_relationsRelations = relations(colony_relations, ({one}) => ({
	colony_colony1_id: one(colonies, {
		fields: [colony_relations.colony1_id],
		references: [colonies.id],
		relationName: "colony_relations_colony1_id_colonies_id"
	}),
	colony_colony2_id: one(colonies, {
		fields: [colony_relations.colony2_id],
		references: [colonies.id],
		relationName: "colony_relations_colony2_id_colonies_id"
	}),
}));

export const colony_cultureRelations = relations(colony_culture, ({one}) => ({
	colony: one(colonies, {
		fields: [colony_culture.colony_id],
		references: [colonies.id]
	}),
}));

export const espionage_missionsRelations = relations(espionage_missions, ({one}) => ({
	ant: one(ants, {
		fields: [espionage_missions.spy_ant_id],
		references: [ants.id]
	}),
	colony_origin_colony_id: one(colonies, {
		fields: [espionage_missions.origin_colony_id],
		references: [colonies.id],
		relationName: "espionage_missions_origin_colony_id_colonies_id"
	}),
	colony_target_colony_id: one(colonies, {
		fields: [espionage_missions.target_colony_id],
		references: [colonies.id],
		relationName: "espionage_missions_target_colony_id_colonies_id"
	}),
}));

export const migration_patternsRelations = relations(migration_patterns, ({one}) => ({
	colony: one(colonies, {
		fields: [migration_patterns.colony_id],
		references: [colonies.id]
	}),
}));

export const water_bodiesRelations = relations(water_bodies, ({one}) => ({
	simulation: one(simulations, {
		fields: [water_bodies.simulation_id],
		references: [simulations.id]
	}),
}));

export const fire_zonesRelations = relations(fire_zones, ({one}) => ({
	simulation: one(simulations, {
		fields: [fire_zones.simulation_id],
		references: [simulations.id]
	}),
}));

export const soil_zonesRelations = relations(soil_zones, ({one}) => ({
	simulation: one(simulations, {
		fields: [soil_zones.simulation_id],
		references: [simulations.id]
	}),
}));

export const climate_zonesRelations = relations(climate_zones, ({one}) => ({
	simulation: one(simulations, {
		fields: [climate_zones.simulation_id],
		references: [simulations.id]
	}),
}));

export const weather_systemsRelations = relations(weather_systems, ({one}) => ({
	simulation: one(simulations, {
		fields: [weather_systems.simulation_id],
		references: [simulations.id]
	}),
}));