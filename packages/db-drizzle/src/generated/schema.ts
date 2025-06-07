import { pgTable, uuid, varchar, text, integer, timestamp, boolean, bigint, foreignKey, jsonb, index, unique, serial, pgView, numeric } from "drizzle-orm/pg-core"
import { sql } from "drizzle-orm"



export const simulations = pgTable("simulations", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	name: varchar({ length: 255 }).notNull(),
	description: text(),
	world_width: integer().default(800).notNull(),
	world_height: integer().default(600).notNull(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
	updated_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
	is_active: boolean().default(true),
	simulation_speed: integer().default(1),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	current_tick: bigint({ mode: "number" }).default(0),
	season: varchar({ length: 20 }).default('spring'),
	time_of_day: integer().default(720),
	weather_type: varchar({ length: 20 }).default('clear'),
	weather_intensity: integer().default(0),
});

export const colonies = pgTable("colonies", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	name: varchar({ length: 100 }).notNull(),
	center_x: integer().notNull(),
	center_y: integer().notNull(),
	radius: integer().default(30).notNull(),
	population: integer().default(0).notNull(),
	color_hue: integer().default(30).notNull(),
	resources: jsonb().default({}).notNull(),
	nest_level: integer().default(1).notNull(),
	territory_radius: integer().default(100).notNull(),
	aggression_level: integer().default(1).notNull(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
	is_active: boolean().default(true),
}, (table) => {
	return {
		colonies_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "colonies_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const ants = pgTable("ants", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	colony_id: uuid().notNull(),
	ant_type_id: integer().notNull(),
	position_x: integer().notNull(),
	position_y: integer().notNull(),
	angle: integer().default(0).notNull(),
	current_speed: integer().notNull(),
	health: integer().notNull(),
	age_ticks: integer().default(0).notNull(),
	state: varchar({ length: 30 }).default('wandering').notNull(),
	target_x: integer(),
	target_y: integer(),
	target_type: varchar({ length: 30 }),
	target_id: uuid(),
	carried_resources: jsonb().default({}),
	traits: jsonb(),
	energy: integer().default(100).notNull(),
	mood: varchar({ length: 20 }).default('neutral'),
	last_updated: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_ants_colony_state: index("idx_ants_colony_state").using("btree", table.colony_id.asc().nullsLast().op("text_ops"), table.state.asc().nullsLast().op("text_ops")),
		idx_ants_position: index("idx_ants_position").using("btree", table.position_x.asc().nullsLast().op("int4_ops"), table.position_y.asc().nullsLast().op("int4_ops")),
		ants_colony_id_fkey: foreignKey({
			columns: [table.colony_id],
			foreignColumns: [colonies.id],
			name: "ants_colony_id_fkey"
		}).onDelete("cascade"),
		ants_ant_type_id_fkey: foreignKey({
			columns: [table.ant_type_id],
			foreignColumns: [ant_types.id],
			name: "ants_ant_type_id_fkey"
		}),
	}
});

export const ant_types = pgTable("ant_types", {
	id: serial().primaryKey().notNull(),
	name: varchar({ length: 50 }).notNull(),
	base_speed: integer().default(1).notNull(),
	base_strength: integer().default(1).notNull(),
	base_health: integer().default(100).notNull(),
	base_size: integer().default(3).notNull(),
	lifespan_ticks: integer().default(50000).notNull(),
	carrying_capacity: integer().default(1).notNull(),
	role: varchar({ length: 30 }).notNull(),
	color_hue: integer().default(30).notNull(),
	special_abilities: jsonb(),
	food_preferences: jsonb(),
}, (table) => {
	return {
		ant_types_name_key: unique("ant_types_name_key").on(table.name),
	}
});

export const food_sources = pgTable("food_sources", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	food_type: varchar({ length: 30 }).notNull(),
	position_x: integer().notNull(),
	position_y: integer().notNull(),
	amount: integer().notNull(),
	max_amount: integer().notNull(),
	regeneration_rate: integer().default(0),
	discovery_difficulty: integer().default(1),
	nutritional_value: integer().default(1).notNull(),
	spoilage_rate: integer().default(0),
	is_renewable: boolean().default(false),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_food_sources_position: index("idx_food_sources_position").using("btree", table.position_x.asc().nullsLast().op("int4_ops"), table.position_y.asc().nullsLast().op("int4_ops")),
		idx_food_sources_simulation: index("idx_food_sources_simulation").using("btree", table.simulation_id.asc().nullsLast().op("uuid_ops")),
		food_sources_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "food_sources_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const pheromone_trails = pgTable("pheromone_trails", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	colony_id: uuid().notNull(),
	trail_type: varchar({ length: 30 }).notNull(),
	position_x: integer().notNull(),
	position_y: integer().notNull(),
	strength: integer().notNull(),
	decay_rate: integer().default(0).notNull(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
	expires_at: timestamp({ withTimezone: true, mode: 'string' }),
	source_ant_id: uuid(),
	target_food_id: uuid(),
}, (table) => {
	return {
		idx_pheromone_trails_colony: index("idx_pheromone_trails_colony").using("btree", table.colony_id.asc().nullsLast().op("text_ops"), table.trail_type.asc().nullsLast().op("uuid_ops")),
		idx_pheromone_trails_expires: index("idx_pheromone_trails_expires").using("btree", table.expires_at.asc().nullsLast().op("timestamptz_ops")),
		idx_pheromone_trails_position: index("idx_pheromone_trails_position").using("btree", table.position_x.asc().nullsLast().op("int4_ops"), table.position_y.asc().nullsLast().op("int4_ops")),
		pheromone_trails_colony_id_fkey: foreignKey({
			columns: [table.colony_id],
			foreignColumns: [colonies.id],
			name: "pheromone_trails_colony_id_fkey"
		}).onDelete("cascade"),
		pheromone_trails_source_ant_id_fkey: foreignKey({
			columns: [table.source_ant_id],
			foreignColumns: [ants.id],
			name: "pheromone_trails_source_ant_id_fkey"
		}),
		pheromone_trails_target_food_id_fkey: foreignKey({
			columns: [table.target_food_id],
			foreignColumns: [food_sources.id],
			name: "pheromone_trails_target_food_id_fkey"
		}),
	}
});

export const obstacles = pgTable("obstacles", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	obstacle_type: varchar({ length: 30 }).notNull(),
	shape: varchar({ length: 20 }).notNull(),
	position_x: integer().notNull(),
	position_y: integer().notNull(),
	width: integer(),
	height: integer(),
	radius: integer(),
	polygon_points: jsonb(),
	is_passable: boolean().default(false),
	movement_cost: integer().default(2),
	affects_pheromones: boolean().default(false),
	visual_properties: jsonb(),
}, (table) => {
	return {
		obstacles_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "obstacles_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const predators = pgTable("predators", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	predator_type: varchar({ length: 30 }).notNull(),
	position_x: integer().notNull(),
	position_y: integer().notNull(),
	angle: integer().default(0).notNull(),
	speed: integer().default(1).notNull(),
	detection_radius: integer().default(40).notNull(),
	attack_radius: integer().default(10).notNull(),
	health: integer().default(50).notNull(),
	hunger: integer().default(0).notNull(),
	state: varchar({ length: 30 }).default('patrolling'),
	target_ant_id: uuid(),
	last_hunt_tick: integer().default(0),
	territory_center_x: integer(),
	territory_center_y: integer(),
	territory_radius: integer().default(80),
}, (table) => {
	return {
		predators_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "predators_simulation_id_fkey"
		}).onDelete("cascade"),
		predators_target_ant_id_fkey: foreignKey({
			columns: [table.target_ant_id],
			foreignColumns: [ants.id],
			name: "predators_target_ant_id_fkey"
		}),
	}
});

export const simulation_events = pgTable("simulation_events", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	event_type: varchar({ length: 30 }).notNull(),
	severity: integer().default(1).notNull(),
	center_x: integer(),
	center_y: integer(),
	radius: integer(),
	start_tick: integer().notNull(),
	duration_ticks: integer(),
	effects: jsonb(),
	is_active: boolean().default(true),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_simulation_events_active: index("idx_simulation_events_active").using("btree", table.simulation_id.asc().nullsLast().op("bool_ops"), table.is_active.asc().nullsLast().op("bool_ops")),
		simulation_events_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "simulation_events_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const ant_interactions = pgTable("ant_interactions", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	ant1_id: uuid().notNull(),
	ant2_id: uuid().notNull(),
	interaction_type: varchar({ length: 30 }).notNull(),
	outcome: varchar({ length: 30 }),
	damage_dealt: integer().default(0),
	resources_exchanged: jsonb(),
	tick_occurred: integer().notNull(),
	position_x: integer(),
	position_y: integer(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		ant_interactions_ant1_id_fkey: foreignKey({
			columns: [table.ant1_id],
			foreignColumns: [ants.id],
			name: "ant_interactions_ant1_id_fkey"
		}),
		ant_interactions_ant2_id_fkey: foreignKey({
			columns: [table.ant2_id],
			foreignColumns: [ants.id],
			name: "ant_interactions_ant2_id_fkey"
		}),
	}
});

export const ant_genetics = pgTable("ant_genetics", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	ant_id: uuid().notNull(),
	parent1_id: uuid(),
	parent2_id: uuid(),
	generation: integer().default(1).notNull(),
	genes: jsonb().notNull(),
	fitness_score: integer(),
	mutations: jsonb(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		ant_genetics_ant_id_fkey: foreignKey({
			columns: [table.ant_id],
			foreignColumns: [ants.id],
			name: "ant_genetics_ant_id_fkey"
		}).onDelete("cascade"),
		ant_genetics_parent1_id_fkey: foreignKey({
			columns: [table.parent1_id],
			foreignColumns: [ants.id],
			name: "ant_genetics_parent1_id_fkey"
		}),
		ant_genetics_parent2_id_fkey: foreignKey({
			columns: [table.parent2_id],
			foreignColumns: [ants.id],
			name: "ant_genetics_parent2_id_fkey"
		}),
	}
});

export const colony_upgrades = pgTable("colony_upgrades", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	colony_id: uuid().notNull(),
	upgrade_type: varchar({ length: 50 }).notNull(),
	level: integer().default(1).notNull(),
	cost_paid: jsonb(),
	effects: jsonb(),
	unlocked_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		colony_upgrades_colony_id_fkey: foreignKey({
			columns: [table.colony_id],
			foreignColumns: [colonies.id],
			name: "colony_upgrades_colony_id_fkey"
		}).onDelete("cascade"),
	}
});

export const simulation_stats = pgTable("simulation_stats", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	tick_number: integer().notNull(),
	total_ants: integer().notNull(),
	total_food_collected: integer().default(0).notNull(),
	total_distance_traveled: integer().default(0).notNull(),
	pheromone_trail_count: integer().default(0).notNull(),
	active_combats: integer().default(0).notNull(),
	weather_effects_active: integer().default(0).notNull(),
	average_ant_health: integer(),
	dominant_colony_id: uuid(),
	recorded_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_simulation_stats_tick: index("idx_simulation_stats_tick").using("btree", table.simulation_id.asc().nullsLast().op("int4_ops"), table.tick_number.asc().nullsLast().op("int4_ops")),
		simulation_stats_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "simulation_stats_simulation_id_fkey"
		}).onDelete("cascade"),
		simulation_stats_dominant_colony_id_fkey: foreignKey({
			columns: [table.dominant_colony_id],
			foreignColumns: [colonies.id],
			name: "simulation_stats_dominant_colony_id_fkey"
		}),
	}
});

export const scenario_templates = pgTable("scenario_templates", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	name: varchar({ length: 255 }).notNull(),
	description: text(),
	creator_id: varchar({ length: 255 }),
	world_config: jsonb().notNull(),
	difficulty_rating: integer().default(1),
	tags: varchar({ length: 255 }).array(),
	is_public: boolean().default(false),
	play_count: integer().default(0),
	average_rating: integer(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
});

export const plants = pgTable("plants", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	plant_type: varchar({ length: 30 }).notNull(),
	species: varchar({ length: 50 }).notNull(),
	position_x: integer().notNull(),
	position_y: integer().notNull(),
	size: integer().default(5).notNull(),
	max_size: integer().default(20).notNull(),
	growth_rate: integer().default(0).notNull(),
	health: integer().default(100).notNull(),
	age_ticks: integer().default(0).notNull(),
	root_radius: integer().default(15).notNull(),
	canopy_radius: integer().default(10).notNull(),
	fruit_production_rate: integer().default(0),
	oxygen_production: integer().default(0),
	water_requirement: integer().default(1).notNull(),
	nutrient_requirements: jsonb(),
	symbiotic_species: varchar({ length: 50 }).array(),
	seasonal_behavior: jsonb(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_plants_position: index("idx_plants_position").using("btree", table.position_x.asc().nullsLast().op("int4_ops"), table.position_y.asc().nullsLast().op("int4_ops")),
		idx_plants_simulation: index("idx_plants_simulation").using("btree", table.simulation_id.asc().nullsLast().op("text_ops"), table.plant_type.asc().nullsLast().op("uuid_ops")),
		plants_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "plants_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const decomposers = pgTable("decomposers", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	decomposer_type: varchar({ length: 30 }).notNull(),
	position_x: integer().notNull(),
	position_y: integer().notNull(),
	radius: integer().default(3).notNull(),
	efficiency: integer().default(0).notNull(),
	nutrient_output: jsonb(),
	target_material: varchar({ length: 30 }),
	population: integer().default(100).notNull(),
	optimal_temperature: integer(),
	optimal_ph: integer(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_decomposers_position: index("idx_decomposers_position").using("btree", table.position_x.asc().nullsLast().op("int4_ops"), table.position_y.asc().nullsLast().op("int4_ops")),
		decomposers_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "decomposers_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const species = pgTable("species", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	species_type: varchar({ length: 30 }).notNull(),
	species_name: varchar({ length: 50 }).notNull(),
	position_x: integer().notNull(),
	position_y: integer().notNull(),
	population: integer().default(1).notNull(),
	mobility: varchar({ length: 20 }).notNull(),
	diet_type: varchar({ length: 20 }).notNull(),
	symbiotic_relationships: jsonb(),
	territory_radius: integer().default(20),
	reproduction_rate: integer().default(0),
	mortality_rate: integer().default(0),
	food_requirements: jsonb(),
	environmental_preferences: jsonb(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_species_position: index("idx_species_position").using("btree", table.position_x.asc().nullsLast().op("int4_ops"), table.position_y.asc().nullsLast().op("int4_ops")),
		species_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "species_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const diseases = pgTable("diseases", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	disease_name: varchar({ length: 50 }).notNull(),
	pathogen_type: varchar({ length: 30 }).notNull(),
	transmission_method: varchar({ length: 30 }).notNull(),
	transmission_rate: integer().default(0).notNull(),
	incubation_period: integer().default(100).notNull(),
	mortality_rate: integer().default(0).notNull(),
	recovery_rate: integer().default(0).notNull(),
	immunity_duration: integer(),
	affected_species: varchar({ length: 30 }).array(),
	symptoms: jsonb(),
	environmental_survival: integer().default(1000),
	mutation_rate: integer().default(0),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		diseases_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "diseases_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const infections = pgTable("infections", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	disease_id: uuid().notNull(),
	host_id: uuid().notNull(),
	host_type: varchar({ length: 20 }).notNull(),
	infection_stage: varchar({ length: 20 }).notNull(),
	infected_at_tick: integer().notNull(),
	symptoms_start_tick: integer(),
	recovery_tick: integer(),
	transmission_events: integer().default(0),
	strain_mutations: jsonb(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_infections_host: index("idx_infections_host").using("btree", table.host_id.asc().nullsLast().op("uuid_ops"), table.host_type.asc().nullsLast().op("text_ops"), table.infection_stage.asc().nullsLast().op("uuid_ops")),
		infections_disease_id_fkey: foreignKey({
			columns: [table.disease_id],
			foreignColumns: [diseases.id],
			name: "infections_disease_id_fkey"
		}).onDelete("cascade"),
	}
});

export const ant_castes = pgTable("ant_castes", {
	id: serial().primaryKey().notNull(),
	caste_name: varchar({ length: 50 }).notNull(),
	specialization: varchar({ length: 50 }).notNull(),
	base_attributes: jsonb().notNull(),
	special_abilities: jsonb(),
	training_requirements: jsonb(),
	population_cap_percentage: integer().default(0),
	unlock_conditions: jsonb(),
	maintenance_cost: jsonb(),
}, (table) => {
	return {
		ant_castes_caste_name_key: unique("ant_castes_caste_name_key").on(table.caste_name),
	}
});

export const colony_relations = pgTable("colony_relations", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	colony1_id: uuid().notNull(),
	colony2_id: uuid().notNull(),
	relationship_type: varchar({ length: 30 }).notNull(),
	trust_level: integer().default(0).notNull(),
	trade_agreements: jsonb(),
	military_pacts: jsonb(),
	territorial_agreements: jsonb(),
	last_interaction_tick: integer(),
	relationship_history: jsonb().array(),
	established_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
	expires_at: timestamp({ withTimezone: true, mode: 'string' }),
}, (table) => {
	return {
		idx_colony_relations_colonies: index("idx_colony_relations_colonies").using("btree", table.colony1_id.asc().nullsLast().op("uuid_ops"), table.colony2_id.asc().nullsLast().op("uuid_ops")),
		colony_relations_colony1_id_fkey: foreignKey({
			columns: [table.colony1_id],
			foreignColumns: [colonies.id],
			name: "colony_relations_colony1_id_fkey"
		}).onDelete("cascade"),
		colony_relations_colony2_id_fkey: foreignKey({
			columns: [table.colony2_id],
			foreignColumns: [colonies.id],
			name: "colony_relations_colony2_id_fkey"
		}).onDelete("cascade"),
		colony_relations_colony1_id_colony2_id_key: unique("colony_relations_colony1_id_colony2_id_key").on(table.colony1_id, table.colony2_id),
	}
});

export const colony_culture = pgTable("colony_culture", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	colony_id: uuid().notNull(),
	cultural_trait: varchar({ length: 50 }).notNull(),
	trait_strength: integer().default(1).notNull(),
	origin_story: jsonb(),
	behavioral_modifiers: jsonb(),
	ritual_behaviors: jsonb(),
	knowledge_traditions: jsonb(),
	innovation_rate: integer().default(0),
	influence_radius: integer().default(50),
	developed_at_tick: integer().notNull(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		colony_culture_colony_id_fkey: foreignKey({
			columns: [table.colony_id],
			foreignColumns: [colonies.id],
			name: "colony_culture_colony_id_fkey"
		}).onDelete("cascade"),
	}
});

export const espionage_missions = pgTable("espionage_missions", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	spy_ant_id: uuid().notNull(),
	origin_colony_id: uuid().notNull(),
	target_colony_id: uuid().notNull(),
	mission_type: varchar({ length: 30 }).notNull(),
	mission_status: varchar({ length: 20 }).default('planning').notNull(),
	objectives: jsonb().notNull(),
	cover_identity: varchar({ length: 50 }),
	discovery_risk: integer().default(0).notNull(),
	intelligence_gathered: jsonb(),
	resources_stolen: jsonb(),
	started_at_tick: integer(),
	completed_at_tick: integer(),
	success_rating: integer(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_espionage_missions_active: index("idx_espionage_missions_active").using("btree", table.mission_status.asc().nullsLast().op("text_ops"), table.target_colony_id.asc().nullsLast().op("text_ops")),
		espionage_missions_spy_ant_id_fkey: foreignKey({
			columns: [table.spy_ant_id],
			foreignColumns: [ants.id],
			name: "espionage_missions_spy_ant_id_fkey"
		}).onDelete("cascade"),
		espionage_missions_origin_colony_id_fkey: foreignKey({
			columns: [table.origin_colony_id],
			foreignColumns: [colonies.id],
			name: "espionage_missions_origin_colony_id_fkey"
		}).onDelete("cascade"),
		espionage_missions_target_colony_id_fkey: foreignKey({
			columns: [table.target_colony_id],
			foreignColumns: [colonies.id],
			name: "espionage_missions_target_colony_id_fkey"
		}).onDelete("cascade"),
	}
});

export const migration_patterns = pgTable("migration_patterns", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	colony_id: uuid().notNull(),
	pattern_type: varchar({ length: 30 }).notNull(),
	trigger_conditions: jsonb().notNull(),
	destination_preferences: jsonb(),
	migration_routes: jsonb().array(),
	preparation_time: integer().default(1000),
	migration_speed: integer().default(1),
	survival_rate: integer().default(1),
	last_migration_tick: integer(),
	seasonal_schedule: jsonb(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		migration_patterns_colony_id_fkey: foreignKey({
			columns: [table.colony_id],
			foreignColumns: [colonies.id],
			name: "migration_patterns_colony_id_fkey"
		}).onDelete("cascade"),
	}
});

export const water_bodies = pgTable("water_bodies", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	water_type: varchar({ length: 30 }).notNull(),
	shape: varchar({ length: 20 }).notNull(),
	center_x: integer().notNull(),
	center_y: integer().notNull(),
	width: integer(),
	length: integer(),
	radius: integer(),
	polygon_points: jsonb(),
	depth: integer().default(1).notNull(),
	flow_direction: integer(),
	flow_speed: integer().default(0),
	water_quality: integer().default(1),
	evaporation_rate: integer().default(0),
	is_seasonal: boolean().default(false),
	temperature: integer(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_water_bodies_position: index("idx_water_bodies_position").using("btree", table.center_x.asc().nullsLast().op("int4_ops"), table.center_y.asc().nullsLast().op("int4_ops")),
		water_bodies_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "water_bodies_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const fire_zones = pgTable("fire_zones", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	center_x: integer().notNull(),
	center_y: integer().notNull(),
	radius: integer().default(10).notNull(),
	intensity: integer().default(1).notNull(),
	fuel_remaining: integer().default(100).notNull(),
	spread_rate: integer().default(0).notNull(),
	wind_influence: integer().default(1),
	started_at_tick: integer().notNull(),
	extinguished_at_tick: integer(),
	ignition_source: varchar({ length: 30 }),
	suppression_efforts: jsonb(),
	casualties: jsonb(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_fire_zones_active: index("idx_fire_zones_active").using("btree", table.simulation_id.asc().nullsLast().op("uuid_ops")).where(sql`(extinguished_at_tick IS NULL)`),
		fire_zones_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "fire_zones_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const soil_zones = pgTable("soil_zones", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	zone_name: varchar({ length: 50 }),
	center_x: integer().notNull(),
	center_y: integer().notNull(),
	radius: integer().default(50).notNull(),
	soil_type: varchar({ length: 30 }).notNull(),
	ph_level: integer().default(7).notNull(),
	nutrients: jsonb().notNull(),
	moisture_content: integer().default(1).notNull(),
	compaction: integer().default(0).notNull(),
	temperature: integer(),
	microbial_activity: integer().default(1),
	drainage_rate: integer().default(0),
	contamination_level: integer().default(0),
	fertility_score: integer(),
	last_updated_tick: integer(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_soil_zones_position: index("idx_soil_zones_position").using("btree", table.center_x.asc().nullsLast().op("int4_ops"), table.center_y.asc().nullsLast().op("int4_ops")),
		soil_zones_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "soil_zones_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const climate_zones = pgTable("climate_zones", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	zone_name: varchar({ length: 50 }),
	center_x: integer().notNull(),
	center_y: integer().notNull(),
	radius: integer().default(75).notNull(),
	temperature: integer().notNull(),
	humidity: integer().notNull(),
	wind_speed: integer().default(0),
	wind_direction: integer().default(0),
	light_level: integer().default(1).notNull(),
	air_pressure: integer().default(1013),
	seasonal_variations: jsonb(),
	elevation: integer().default(0),
	vegetation_cover: integer().default(1),
	created_by: varchar({ length: 30 }),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		idx_climate_zones_position: index("idx_climate_zones_position").using("btree", table.center_x.asc().nullsLast().op("int4_ops"), table.center_y.asc().nullsLast().op("int4_ops")),
		climate_zones_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "climate_zones_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});

export const weather_systems = pgTable("weather_systems", {
	id: uuid().defaultRandom().primaryKey().notNull(),
	simulation_id: uuid().notNull(),
	weather_type: varchar({ length: 30 }).notNull(),
	center_x: integer(),
	center_y: integer(),
	radius: integer(),
	intensity: integer().default(1).notNull(),
	movement_vector_x: integer().default(0),
	movement_vector_y: integer().default(0),
	duration_remaining: integer().notNull(),
	effects: jsonb().notNull(),
	pressure_change: integer(),
	started_at_tick: integer().notNull(),
	forecast_accuracy: integer().default(1),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`CURRENT_TIMESTAMP`),
}, (table) => {
	return {
		weather_systems_simulation_id_fkey: foreignKey({
			columns: [table.simulation_id],
			foreignColumns: [simulations.id],
			name: "weather_systems_simulation_id_fkey"
		}).onDelete("cascade"),
	}
});
export const ecosystem_health = pgView("ecosystem_health", {	simulation_id: uuid(),
	name: varchar({ length: 255 }),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	active_colonies: bigint({ mode: "number" }),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	total_ants: bigint({ mode: "number" }),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	plant_count: bigint({ mode: "number" }),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	other_species_count: bigint({ mode: "number" }),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	active_diseases: bigint({ mode: "number" }),
	avg_soil_fertility: numeric(),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	water_sources: bigint({ mode: "number" }),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	active_fires: bigint({ mode: "number" }),
	avg_temperature: numeric(),
	avg_humidity: numeric(),
}).as(sql`SELECT s.id AS simulation_id, s.name, count(DISTINCT c.id) AS active_colonies, count(DISTINCT a.id) AS total_ants, count(DISTINCT p.id) AS plant_count, count(DISTINCT sp.id) AS other_species_count, count(DISTINCT d.id) AS active_diseases, avg(sz.fertility_score) AS avg_soil_fertility, count(DISTINCT wb.id) AS water_sources, count(DISTINCT fz.id) AS active_fires, avg(cz.temperature) AS avg_temperature, avg(cz.humidity) AS avg_humidity FROM simulations s LEFT JOIN colonies c ON s.id = c.simulation_id AND c.is_active = true LEFT JOIN ants a ON c.id = a.colony_id AND a.state::text <> 'dead'::text LEFT JOIN plants p ON s.id = p.simulation_id LEFT JOIN species sp ON s.id = sp.simulation_id LEFT JOIN diseases d ON s.id = d.simulation_id LEFT JOIN soil_zones sz ON s.id = sz.simulation_id LEFT JOIN water_bodies wb ON s.id = wb.simulation_id LEFT JOIN fire_zones fz ON s.id = fz.simulation_id AND fz.extinguished_at_tick IS NULL LEFT JOIN climate_zones cz ON s.id = cz.simulation_id GROUP BY s.id, s.name`);

export const colony_performance = pgView("colony_performance", {	id: uuid(),
	name: varchar({ length: 100 }),
	population: integer(),
	resources: jsonb(),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	nearby_food_sources: bigint({ mode: "number" }),
	avg_ant_health: numeric(),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	ants_with_food: bigint({ mode: "number" }),
	// You can use { mode: "bigint" } if numbers are exceeding js number limitations
	active_pheromone_trails: bigint({ mode: "number" }),
}).as(sql`SELECT c.id, c.name, c.population, c.resources, count(DISTINCT fs.id) AS nearby_food_sources, avg(a.health) AS avg_ant_health, count( CASE WHEN a.state::text = 'carrying_food'::text THEN 1 ELSE NULL::integer END) AS ants_with_food, count(DISTINCT pt.id) AS active_pheromone_trails FROM colonies c LEFT JOIN ants a ON c.id = a.colony_id AND a.state::text <> 'dead'::text LEFT JOIN food_sources fs ON (fs.simulation_id IN ( SELECT colonies.simulation_id FROM colonies WHERE colonies.id = c.id)) AND sqrt(power((fs.position_x - c.center_x)::double precision, 2::double precision) + power((fs.position_y - c.center_y)::double precision, 2::double precision)) < c.territory_radius::double precision LEFT JOIN pheromone_trails pt ON c.id = pt.colony_id AND pt.strength > 0 GROUP BY c.id, c.name, c.population, c.resources`);