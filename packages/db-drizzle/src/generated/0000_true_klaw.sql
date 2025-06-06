-- Current sql file was generated after introspecting the database
-- If you want to run this migration please uncomment this code before executing migrations
/*
CREATE TABLE IF NOT EXISTS "food_sources" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"food_type" varchar(30) NOT NULL,
	"position_x" numeric(8, 2) NOT NULL,
	"position_y" numeric(8, 2) NOT NULL,
	"amount" numeric(8, 2) NOT NULL,
	"max_amount" numeric(8, 2) NOT NULL,
	"regeneration_rate" numeric(4, 2) DEFAULT '0',
	"discovery_difficulty" numeric(3, 2) DEFAULT '0.5',
	"nutritional_value" numeric(4, 2) DEFAULT '1.0' NOT NULL,
	"spoilage_rate" numeric(6, 4) DEFAULT '0',
	"is_renewable" boolean DEFAULT false,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "colonies" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"name" varchar(100) NOT NULL,
	"center_x" numeric(8, 2) NOT NULL,
	"center_y" numeric(8, 2) NOT NULL,
	"radius" numeric(6, 2) DEFAULT '30.0' NOT NULL,
	"population" integer DEFAULT 0 NOT NULL,
	"color_hue" integer DEFAULT 30 NOT NULL,
	"resources" jsonb DEFAULT '{}'::jsonb NOT NULL,
	"nest_level" integer DEFAULT 1 NOT NULL,
	"territory_radius" numeric(6, 2) DEFAULT '100.0' NOT NULL,
	"aggression_level" numeric(3, 2) DEFAULT '0.5' NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
	"is_active" boolean DEFAULT true
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "simulations" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"name" varchar(255) NOT NULL,
	"description" text,
	"world_width" integer DEFAULT 800 NOT NULL,
	"world_height" integer DEFAULT 600 NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
	"updated_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
	"is_active" boolean DEFAULT true,
	"simulation_speed" numeric(3, 2) DEFAULT '1.0',
	"current_tick" bigint DEFAULT 0,
	"season" varchar(20) DEFAULT 'spring',
	"time_of_day" integer DEFAULT 720,
	"weather_type" varchar(20) DEFAULT 'clear',
	"weather_intensity" numeric(3, 2) DEFAULT '0.0'
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "ants" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"colony_id" uuid NOT NULL,
	"ant_type_id" integer NOT NULL,
	"position_x" numeric(8, 2) NOT NULL,
	"position_y" numeric(8, 2) NOT NULL,
	"angle" numeric(5, 2) DEFAULT '0' NOT NULL,
	"current_speed" numeric(4, 2) NOT NULL,
	"health" integer NOT NULL,
	"age_ticks" integer DEFAULT 0 NOT NULL,
	"state" varchar(30) DEFAULT 'wandering' NOT NULL,
	"target_x" numeric(8, 2),
	"target_y" numeric(8, 2),
	"target_type" varchar(30),
	"target_id" uuid,
	"carried_resources" jsonb DEFAULT '{}'::jsonb,
	"traits" jsonb,
	"energy" integer DEFAULT 100 NOT NULL,
	"mood" varchar(20) DEFAULT 'neutral',
	"last_updated" timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "ant_types" (
	"id" serial PRIMARY KEY NOT NULL,
	"name" varchar(50) NOT NULL,
	"base_speed" numeric(4, 2) DEFAULT '1.0' NOT NULL,
	"base_strength" numeric(4, 2) DEFAULT '1.0' NOT NULL,
	"base_health" integer DEFAULT 100 NOT NULL,
	"base_size" numeric(3, 2) DEFAULT '3.0' NOT NULL,
	"lifespan_ticks" integer DEFAULT 50000 NOT NULL,
	"carrying_capacity" numeric(4, 2) DEFAULT '1.0' NOT NULL,
	"role" varchar(30) NOT NULL,
	"color_hue" integer DEFAULT 30 NOT NULL,
	"special_abilities" jsonb,
	"food_preferences" jsonb,
	CONSTRAINT "ant_types_name_key" UNIQUE("name")
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "pheromone_trails" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"colony_id" uuid NOT NULL,
	"trail_type" varchar(30) NOT NULL,
	"position_x" numeric(8, 2) NOT NULL,
	"position_y" numeric(8, 2) NOT NULL,
	"strength" numeric(4, 2) NOT NULL,
	"decay_rate" numeric(6, 4) DEFAULT '0.005' NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
	"expires_at" timestamp with time zone,
	"source_ant_id" uuid,
	"target_food_id" uuid
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "obstacles" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"obstacle_type" varchar(30) NOT NULL,
	"shape" varchar(20) NOT NULL,
	"position_x" numeric(8, 2) NOT NULL,
	"position_y" numeric(8, 2) NOT NULL,
	"width" numeric(6, 2),
	"height" numeric(6, 2),
	"radius" numeric(6, 2),
	"polygon_points" jsonb,
	"is_passable" boolean DEFAULT false,
	"movement_cost" numeric(3, 2) DEFAULT '2.0',
	"affects_pheromones" boolean DEFAULT false,
	"visual_properties" jsonb
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "predators" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"predator_type" varchar(30) NOT NULL,
	"position_x" numeric(8, 2) NOT NULL,
	"position_y" numeric(8, 2) NOT NULL,
	"angle" numeric(5, 2) DEFAULT '0' NOT NULL,
	"speed" numeric(4, 2) DEFAULT '0.5' NOT NULL,
	"detection_radius" numeric(6, 2) DEFAULT '40.0' NOT NULL,
	"attack_radius" numeric(6, 2) DEFAULT '10.0' NOT NULL,
	"health" integer DEFAULT 50 NOT NULL,
	"hunger" integer DEFAULT 0 NOT NULL,
	"state" varchar(30) DEFAULT 'patrolling',
	"target_ant_id" uuid,
	"last_hunt_tick" integer DEFAULT 0,
	"territory_center_x" numeric(8, 2),
	"territory_center_y" numeric(8, 2),
	"territory_radius" numeric(6, 2) DEFAULT '80.0'
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "simulation_events" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"event_type" varchar(30) NOT NULL,
	"severity" numeric(3, 2) DEFAULT '1.0' NOT NULL,
	"center_x" numeric(8, 2),
	"center_y" numeric(8, 2),
	"radius" numeric(6, 2),
	"start_tick" integer NOT NULL,
	"duration_ticks" integer,
	"effects" jsonb,
	"is_active" boolean DEFAULT true,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "ant_interactions" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"ant1_id" uuid NOT NULL,
	"ant2_id" uuid NOT NULL,
	"interaction_type" varchar(30) NOT NULL,
	"outcome" varchar(30),
	"damage_dealt" integer DEFAULT 0,
	"resources_exchanged" jsonb,
	"tick_occurred" integer NOT NULL,
	"position_x" numeric(8, 2),
	"position_y" numeric(8, 2),
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "ant_genetics" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"ant_id" uuid NOT NULL,
	"parent1_id" uuid,
	"parent2_id" uuid,
	"generation" integer DEFAULT 1 NOT NULL,
	"genes" jsonb NOT NULL,
	"fitness_score" numeric(8, 2),
	"mutations" jsonb,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "colony_upgrades" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"colony_id" uuid NOT NULL,
	"upgrade_type" varchar(50) NOT NULL,
	"level" integer DEFAULT 1 NOT NULL,
	"cost_paid" jsonb,
	"effects" jsonb,
	"unlocked_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "simulation_stats" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"tick_number" integer NOT NULL,
	"total_ants" integer NOT NULL,
	"total_food_collected" numeric(12, 2) DEFAULT '0' NOT NULL,
	"total_distance_traveled" numeric(15, 2) DEFAULT '0' NOT NULL,
	"pheromone_trail_count" integer DEFAULT 0 NOT NULL,
	"active_combats" integer DEFAULT 0 NOT NULL,
	"weather_effects_active" integer DEFAULT 0 NOT NULL,
	"average_ant_health" numeric(5, 2),
	"dominant_colony_id" uuid,
	"recorded_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "scenario_templates" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"name" varchar(255) NOT NULL,
	"description" text,
	"creator_id" varchar(255),
	"world_config" jsonb NOT NULL,
	"difficulty_rating" integer DEFAULT 1,
	"tags" varchar(255)[],
	"is_public" boolean DEFAULT false,
	"play_count" integer DEFAULT 0,
	"average_rating" numeric(3, 2),
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "plants" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"plant_type" varchar(30) NOT NULL,
	"species" varchar(50) NOT NULL,
	"position_x" numeric(8, 2) NOT NULL,
	"position_y" numeric(8, 2) NOT NULL,
	"size" numeric(6, 2) DEFAULT '5.0' NOT NULL,
	"max_size" numeric(6, 2) DEFAULT '20.0' NOT NULL,
	"growth_rate" numeric(4, 2) DEFAULT '0.01' NOT NULL,
	"health" numeric(5, 2) DEFAULT '100.0' NOT NULL,
	"age_ticks" integer DEFAULT 0 NOT NULL,
	"root_radius" numeric(6, 2) DEFAULT '15.0' NOT NULL,
	"canopy_radius" numeric(6, 2) DEFAULT '10.0' NOT NULL,
	"fruit_production_rate" numeric(4, 2) DEFAULT '0',
	"oxygen_production" numeric(4, 2) DEFAULT '0.1',
	"water_requirement" numeric(4, 2) DEFAULT '1.0' NOT NULL,
	"nutrient_requirements" jsonb,
	"symbiotic_species" varchar(50)[],
	"seasonal_behavior" jsonb,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "decomposers" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"decomposer_type" varchar(30) NOT NULL,
	"position_x" numeric(8, 2) NOT NULL,
	"position_y" numeric(8, 2) NOT NULL,
	"radius" numeric(6, 2) DEFAULT '3.0' NOT NULL,
	"efficiency" numeric(3, 2) DEFAULT '0.1' NOT NULL,
	"nutrient_output" jsonb,
	"target_material" varchar(30),
	"population" integer DEFAULT 100 NOT NULL,
	"optimal_temperature" numeric(4, 1),
	"optimal_ph" numeric(3, 1),
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "species" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"species_type" varchar(30) NOT NULL,
	"species_name" varchar(50) NOT NULL,
	"position_x" numeric(8, 2) NOT NULL,
	"position_y" numeric(8, 2) NOT NULL,
	"population" integer DEFAULT 1 NOT NULL,
	"mobility" varchar(20) NOT NULL,
	"diet_type" varchar(20) NOT NULL,
	"symbiotic_relationships" jsonb,
	"territory_radius" numeric(6, 2) DEFAULT '20.0',
	"reproduction_rate" numeric(4, 2) DEFAULT '0.001',
	"mortality_rate" numeric(4, 2) DEFAULT '0.001',
	"food_requirements" jsonb,
	"environmental_preferences" jsonb,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "diseases" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"disease_name" varchar(50) NOT NULL,
	"pathogen_type" varchar(30) NOT NULL,
	"transmission_method" varchar(30) NOT NULL,
	"transmission_rate" numeric(4, 2) DEFAULT '0.1' NOT NULL,
	"incubation_period" integer DEFAULT 100 NOT NULL,
	"mortality_rate" numeric(3, 2) DEFAULT '0.05' NOT NULL,
	"recovery_rate" numeric(3, 2) DEFAULT '0.1' NOT NULL,
	"immunity_duration" integer,
	"affected_species" varchar(30)[],
	"symptoms" jsonb,
	"environmental_survival" integer DEFAULT 1000,
	"mutation_rate" numeric(6, 4) DEFAULT '0.0001',
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "infections" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"disease_id" uuid NOT NULL,
	"host_id" uuid NOT NULL,
	"host_type" varchar(20) NOT NULL,
	"infection_stage" varchar(20) NOT NULL,
	"infected_at_tick" integer NOT NULL,
	"symptoms_start_tick" integer,
	"recovery_tick" integer,
	"transmission_events" integer DEFAULT 0,
	"strain_mutations" jsonb,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "ant_castes" (
	"id" serial PRIMARY KEY NOT NULL,
	"caste_name" varchar(50) NOT NULL,
	"specialization" varchar(50) NOT NULL,
	"base_attributes" jsonb NOT NULL,
	"special_abilities" jsonb,
	"training_requirements" jsonb,
	"population_cap_percentage" numeric(4, 2) DEFAULT '0.1',
	"unlock_conditions" jsonb,
	"maintenance_cost" jsonb,
	CONSTRAINT "ant_castes_caste_name_key" UNIQUE("caste_name")
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "colony_relations" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"colony1_id" uuid NOT NULL,
	"colony2_id" uuid NOT NULL,
	"relationship_type" varchar(30) NOT NULL,
	"trust_level" numeric(4, 2) DEFAULT '0.0' NOT NULL,
	"trade_agreements" jsonb,
	"military_pacts" jsonb,
	"territorial_agreements" jsonb,
	"last_interaction_tick" integer,
	"relationship_history" jsonb[],
	"established_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
	"expires_at" timestamp with time zone,
	CONSTRAINT "colony_relations_colony1_id_colony2_id_key" UNIQUE("colony1_id","colony2_id")
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "colony_culture" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"colony_id" uuid NOT NULL,
	"cultural_trait" varchar(50) NOT NULL,
	"trait_strength" numeric(3, 2) DEFAULT '1.0' NOT NULL,
	"origin_story" jsonb,
	"behavioral_modifiers" jsonb,
	"ritual_behaviors" jsonb,
	"knowledge_traditions" jsonb,
	"innovation_rate" numeric(4, 2) DEFAULT '0.1',
	"influence_radius" numeric(6, 2) DEFAULT '50.0',
	"developed_at_tick" integer NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "espionage_missions" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"spy_ant_id" uuid NOT NULL,
	"origin_colony_id" uuid NOT NULL,
	"target_colony_id" uuid NOT NULL,
	"mission_type" varchar(30) NOT NULL,
	"mission_status" varchar(20) DEFAULT 'planning' NOT NULL,
	"objectives" jsonb NOT NULL,
	"cover_identity" varchar(50),
	"discovery_risk" numeric(3, 2) DEFAULT '0.1' NOT NULL,
	"intelligence_gathered" jsonb,
	"resources_stolen" jsonb,
	"started_at_tick" integer,
	"completed_at_tick" integer,
	"success_rating" numeric(3, 2),
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "migration_patterns" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"colony_id" uuid NOT NULL,
	"pattern_type" varchar(30) NOT NULL,
	"trigger_conditions" jsonb NOT NULL,
	"destination_preferences" jsonb,
	"migration_routes" jsonb[],
	"preparation_time" integer DEFAULT 1000,
	"migration_speed" numeric(4, 2) DEFAULT '0.5',
	"survival_rate" numeric(3, 2) DEFAULT '0.9',
	"last_migration_tick" integer,
	"seasonal_schedule" jsonb,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "water_bodies" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"water_type" varchar(30) NOT NULL,
	"shape" varchar(20) NOT NULL,
	"center_x" numeric(8, 2) NOT NULL,
	"center_y" numeric(8, 2) NOT NULL,
	"width" numeric(6, 2),
	"length" numeric(6, 2),
	"radius" numeric(6, 2),
	"polygon_points" jsonb,
	"depth" numeric(4, 2) DEFAULT '1.0' NOT NULL,
	"flow_direction" numeric(5, 2),
	"flow_speed" numeric(4, 2) DEFAULT '0',
	"water_quality" numeric(3, 2) DEFAULT '1.0',
	"evaporation_rate" numeric(6, 4) DEFAULT '0.001',
	"is_seasonal" boolean DEFAULT false,
	"temperature" numeric(4, 1),
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "fire_zones" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"center_x" numeric(8, 2) NOT NULL,
	"center_y" numeric(8, 2) NOT NULL,
	"radius" numeric(6, 2) DEFAULT '10.0' NOT NULL,
	"intensity" numeric(3, 2) DEFAULT '1.0' NOT NULL,
	"fuel_remaining" numeric(6, 2) DEFAULT '100.0' NOT NULL,
	"spread_rate" numeric(4, 2) DEFAULT '0.1' NOT NULL,
	"wind_influence" numeric(3, 2) DEFAULT '0.5',
	"started_at_tick" integer NOT NULL,
	"extinguished_at_tick" integer,
	"ignition_source" varchar(30),
	"suppression_efforts" jsonb,
	"casualties" jsonb,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "soil_zones" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"zone_name" varchar(50),
	"center_x" numeric(8, 2) NOT NULL,
	"center_y" numeric(8, 2) NOT NULL,
	"radius" numeric(6, 2) DEFAULT '50.0' NOT NULL,
	"soil_type" varchar(30) NOT NULL,
	"ph_level" numeric(3, 1) DEFAULT '7.0' NOT NULL,
	"nutrients" jsonb NOT NULL,
	"moisture_content" numeric(3, 2) DEFAULT '0.5' NOT NULL,
	"compaction" numeric(3, 2) DEFAULT '0.3' NOT NULL,
	"temperature" numeric(4, 1),
	"microbial_activity" numeric(3, 2) DEFAULT '0.5',
	"drainage_rate" numeric(4, 2) DEFAULT '0.1',
	"contamination_level" numeric(3, 2) DEFAULT '0.0',
	"fertility_score" numeric(3, 2),
	"last_updated_tick" integer,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "climate_zones" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"zone_name" varchar(50),
	"center_x" numeric(8, 2) NOT NULL,
	"center_y" numeric(8, 2) NOT NULL,
	"radius" numeric(6, 2) DEFAULT '75.0' NOT NULL,
	"temperature" numeric(4, 1) NOT NULL,
	"humidity" numeric(3, 2) NOT NULL,
	"wind_speed" numeric(4, 2) DEFAULT '0',
	"wind_direction" numeric(5, 2) DEFAULT '0',
	"light_level" numeric(3, 2) DEFAULT '1.0' NOT NULL,
	"air_pressure" numeric(6, 2) DEFAULT '1013.25',
	"seasonal_variations" jsonb,
	"elevation" numeric(6, 2) DEFAULT '0',
	"vegetation_cover" numeric(3, 2) DEFAULT '0.5',
	"created_by" varchar(30),
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "weather_systems" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"simulation_id" uuid NOT NULL,
	"weather_type" varchar(30) NOT NULL,
	"center_x" numeric(8, 2),
	"center_y" numeric(8, 2),
	"radius" numeric(8, 2),
	"intensity" numeric(3, 2) DEFAULT '1.0' NOT NULL,
	"movement_vector_x" numeric(4, 2) DEFAULT '0',
	"movement_vector_y" numeric(4, 2) DEFAULT '0',
	"duration_remaining" integer NOT NULL,
	"effects" jsonb NOT NULL,
	"pressure_change" numeric(5, 2),
	"started_at_tick" integer NOT NULL,
	"forecast_accuracy" numeric(3, 2) DEFAULT '1.0',
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "food_sources" ADD CONSTRAINT "food_sources_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "colonies" ADD CONSTRAINT "colonies_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "ants" ADD CONSTRAINT "ants_colony_id_fkey" FOREIGN KEY ("colony_id") REFERENCES "public"."colonies"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "ants" ADD CONSTRAINT "ants_ant_type_id_fkey" FOREIGN KEY ("ant_type_id") REFERENCES "public"."ant_types"("id") ON DELETE no action ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "pheromone_trails" ADD CONSTRAINT "pheromone_trails_colony_id_fkey" FOREIGN KEY ("colony_id") REFERENCES "public"."colonies"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "pheromone_trails" ADD CONSTRAINT "pheromone_trails_source_ant_id_fkey" FOREIGN KEY ("source_ant_id") REFERENCES "public"."ants"("id") ON DELETE no action ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "pheromone_trails" ADD CONSTRAINT "pheromone_trails_target_food_id_fkey" FOREIGN KEY ("target_food_id") REFERENCES "public"."food_sources"("id") ON DELETE no action ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "obstacles" ADD CONSTRAINT "obstacles_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "predators" ADD CONSTRAINT "predators_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "predators" ADD CONSTRAINT "predators_target_ant_id_fkey" FOREIGN KEY ("target_ant_id") REFERENCES "public"."ants"("id") ON DELETE no action ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "simulation_events" ADD CONSTRAINT "simulation_events_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "ant_interactions" ADD CONSTRAINT "ant_interactions_ant1_id_fkey" FOREIGN KEY ("ant1_id") REFERENCES "public"."ants"("id") ON DELETE no action ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "ant_interactions" ADD CONSTRAINT "ant_interactions_ant2_id_fkey" FOREIGN KEY ("ant2_id") REFERENCES "public"."ants"("id") ON DELETE no action ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "ant_genetics" ADD CONSTRAINT "ant_genetics_ant_id_fkey" FOREIGN KEY ("ant_id") REFERENCES "public"."ants"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "ant_genetics" ADD CONSTRAINT "ant_genetics_parent1_id_fkey" FOREIGN KEY ("parent1_id") REFERENCES "public"."ants"("id") ON DELETE no action ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "ant_genetics" ADD CONSTRAINT "ant_genetics_parent2_id_fkey" FOREIGN KEY ("parent2_id") REFERENCES "public"."ants"("id") ON DELETE no action ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "colony_upgrades" ADD CONSTRAINT "colony_upgrades_colony_id_fkey" FOREIGN KEY ("colony_id") REFERENCES "public"."colonies"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "simulation_stats" ADD CONSTRAINT "simulation_stats_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "simulation_stats" ADD CONSTRAINT "simulation_stats_dominant_colony_id_fkey" FOREIGN KEY ("dominant_colony_id") REFERENCES "public"."colonies"("id") ON DELETE no action ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "plants" ADD CONSTRAINT "plants_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "decomposers" ADD CONSTRAINT "decomposers_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "species" ADD CONSTRAINT "species_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "diseases" ADD CONSTRAINT "diseases_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "infections" ADD CONSTRAINT "infections_disease_id_fkey" FOREIGN KEY ("disease_id") REFERENCES "public"."diseases"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "colony_relations" ADD CONSTRAINT "colony_relations_colony1_id_fkey" FOREIGN KEY ("colony1_id") REFERENCES "public"."colonies"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "colony_relations" ADD CONSTRAINT "colony_relations_colony2_id_fkey" FOREIGN KEY ("colony2_id") REFERENCES "public"."colonies"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "colony_culture" ADD CONSTRAINT "colony_culture_colony_id_fkey" FOREIGN KEY ("colony_id") REFERENCES "public"."colonies"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "espionage_missions" ADD CONSTRAINT "espionage_missions_spy_ant_id_fkey" FOREIGN KEY ("spy_ant_id") REFERENCES "public"."ants"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "espionage_missions" ADD CONSTRAINT "espionage_missions_origin_colony_id_fkey" FOREIGN KEY ("origin_colony_id") REFERENCES "public"."colonies"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "espionage_missions" ADD CONSTRAINT "espionage_missions_target_colony_id_fkey" FOREIGN KEY ("target_colony_id") REFERENCES "public"."colonies"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "migration_patterns" ADD CONSTRAINT "migration_patterns_colony_id_fkey" FOREIGN KEY ("colony_id") REFERENCES "public"."colonies"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "water_bodies" ADD CONSTRAINT "water_bodies_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "fire_zones" ADD CONSTRAINT "fire_zones_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "soil_zones" ADD CONSTRAINT "soil_zones_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "climate_zones" ADD CONSTRAINT "climate_zones_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
DO $$ BEGIN
 ALTER TABLE "weather_systems" ADD CONSTRAINT "weather_systems_simulation_id_fkey" FOREIGN KEY ("simulation_id") REFERENCES "public"."simulations"("id") ON DELETE cascade ON UPDATE no action;
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_food_sources_position" ON "food_sources" USING btree ("position_x" numeric_ops,"position_y" numeric_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_food_sources_simulation" ON "food_sources" USING btree ("simulation_id" uuid_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_ants_colony_state" ON "ants" USING btree ("colony_id" text_ops,"state" text_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_ants_position" ON "ants" USING btree ("position_x" numeric_ops,"position_y" numeric_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_pheromone_trails_colony" ON "pheromone_trails" USING btree ("colony_id" text_ops,"trail_type" uuid_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_pheromone_trails_expires" ON "pheromone_trails" USING btree ("expires_at" timestamptz_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_pheromone_trails_position" ON "pheromone_trails" USING btree ("position_x" numeric_ops,"position_y" numeric_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_simulation_events_active" ON "simulation_events" USING btree ("simulation_id" bool_ops,"is_active" bool_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_simulation_stats_tick" ON "simulation_stats" USING btree ("simulation_id" int4_ops,"tick_number" int4_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_plants_position" ON "plants" USING btree ("position_x" numeric_ops,"position_y" numeric_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_plants_simulation" ON "plants" USING btree ("simulation_id" text_ops,"plant_type" uuid_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_decomposers_position" ON "decomposers" USING btree ("position_x" numeric_ops,"position_y" numeric_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_species_position" ON "species" USING btree ("position_x" numeric_ops,"position_y" numeric_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_infections_host" ON "infections" USING btree ("host_id" uuid_ops,"host_type" text_ops,"infection_stage" uuid_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_colony_relations_colonies" ON "colony_relations" USING btree ("colony1_id" uuid_ops,"colony2_id" uuid_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_espionage_missions_active" ON "espionage_missions" USING btree ("mission_status" text_ops,"target_colony_id" text_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_water_bodies_position" ON "water_bodies" USING btree ("center_x" numeric_ops,"center_y" numeric_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_fire_zones_active" ON "fire_zones" USING btree ("simulation_id" uuid_ops) WHERE (extinguished_at_tick IS NULL);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_soil_zones_position" ON "soil_zones" USING btree ("center_x" numeric_ops,"center_y" numeric_ops);--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_climate_zones_position" ON "climate_zones" USING btree ("center_x" numeric_ops,"center_y" numeric_ops);--> statement-breakpoint
CREATE VIEW "public"."ecosystem_health" AS (SELECT s.id AS simulation_id, s.name, count(DISTINCT c.id) AS active_colonies, count(DISTINCT a.id) AS total_ants, count(DISTINCT p.id) AS plant_count, count(DISTINCT sp.id) AS other_species_count, count(DISTINCT d.id) AS active_diseases, avg(sz.fertility_score) AS avg_soil_fertility, count(DISTINCT wb.id) AS water_sources, count(DISTINCT fz.id) AS active_fires, avg(cz.temperature) AS avg_temperature, avg(cz.humidity) AS avg_humidity FROM simulations s LEFT JOIN colonies c ON s.id = c.simulation_id AND c.is_active = true LEFT JOIN ants a ON c.id = a.colony_id AND a.state::text <> 'dead'::text LEFT JOIN plants p ON s.id = p.simulation_id LEFT JOIN species sp ON s.id = sp.simulation_id LEFT JOIN diseases d ON s.id = d.simulation_id LEFT JOIN soil_zones sz ON s.id = sz.simulation_id LEFT JOIN water_bodies wb ON s.id = wb.simulation_id LEFT JOIN fire_zones fz ON s.id = fz.simulation_id AND fz.extinguished_at_tick IS NULL LEFT JOIN climate_zones cz ON s.id = cz.simulation_id GROUP BY s.id, s.name);--> statement-breakpoint
CREATE VIEW "public"."colony_performance" AS (SELECT c.id, c.name, c.population, c.resources, count(DISTINCT fs.id) AS nearby_food_sources, avg(a.health) AS avg_ant_health, count( CASE WHEN a.state::text = 'carrying_food'::text THEN 1 ELSE NULL::integer END) AS ants_with_food, count(DISTINCT pt.id) AS active_pheromone_trails FROM colonies c LEFT JOIN ants a ON c.id = a.colony_id AND a.state::text <> 'dead'::text LEFT JOIN food_sources fs ON (fs.simulation_id IN ( SELECT colonies.simulation_id FROM colonies WHERE colonies.id = c.id)) AND sqrt(power(fs.position_x - c.center_x, 2::numeric) + power(fs.position_y - c.center_y, 2::numeric)) < c.territory_radius LEFT JOIN pheromone_trails pt ON c.id = pt.colony_id AND pt.strength > 0.1 GROUP BY c.id, c.name, c.population, c.resources);
*/