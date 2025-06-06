-- Ant Simulation Database Schema
-- Comprehensive structure for advanced ant colony simulation

-- Core simulation management
CREATE TABLE simulations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    world_width INTEGER NOT NULL DEFAULT 800,
    world_height INTEGER NOT NULL DEFAULT 600,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT true,
    simulation_speed DECIMAL(3,2) DEFAULT 1.0,
    current_tick BIGINT DEFAULT 0,
    season VARCHAR(20) DEFAULT 'spring', -- spring, summer, fall, winter
    time_of_day INTEGER DEFAULT 720, -- minutes since midnight (720 = noon)
    weather_type VARCHAR(20) DEFAULT 'clear', -- clear, rain, wind, storm
    weather_intensity DECIMAL(3,2) DEFAULT 0.0
);

-- Different ant species/types
CREATE TABLE ant_types (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    base_speed DECIMAL(4,2) NOT NULL DEFAULT 1.0,
    base_strength DECIMAL(4,2) NOT NULL DEFAULT 1.0,
    base_health INTEGER NOT NULL DEFAULT 100,
    base_size DECIMAL(3,2) NOT NULL DEFAULT 3.0,
    lifespan_ticks INTEGER NOT NULL DEFAULT 50000,
    carrying_capacity DECIMAL(4,2) NOT NULL DEFAULT 1.0,
    role VARCHAR(30) NOT NULL, -- worker, soldier, scout, queen, nurse
    color_hue INTEGER NOT NULL DEFAULT 30, -- HSL hue value
    special_abilities JSONB, -- {vision_range: 50, can_fight: true, etc}
    food_preferences JSONB -- {seeds: 1.0, sugar: 1.5, protein: 0.8}
);

-- Ant colonies/nests
CREATE TABLE colonies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    center_x DECIMAL(8,2) NOT NULL,
    center_y DECIMAL(8,2) NOT NULL,
    radius DECIMAL(6,2) NOT NULL DEFAULT 30.0,
    population INTEGER NOT NULL DEFAULT 0,
    color_hue INTEGER NOT NULL DEFAULT 30,
    resources JSONB NOT NULL DEFAULT '{}', -- {seeds: 100, sugar: 50, protein: 25}
    nest_level INTEGER NOT NULL DEFAULT 1,
    territory_radius DECIMAL(6,2) NOT NULL DEFAULT 100.0,
    aggression_level DECIMAL(3,2) NOT NULL DEFAULT 0.5,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT true
);

-- Individual ants
CREATE TABLE ants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    colony_id UUID NOT NULL REFERENCES colonies(id) ON DELETE CASCADE,
    ant_type_id INTEGER NOT NULL REFERENCES ant_types(id),
    position_x DECIMAL(8,2) NOT NULL,
    position_y DECIMAL(8,2) NOT NULL,
    angle DECIMAL(5,2) NOT NULL DEFAULT 0, -- radians
    current_speed DECIMAL(4,2) NOT NULL,
    health INTEGER NOT NULL,
    age_ticks INTEGER NOT NULL DEFAULT 0,
    state VARCHAR(30) NOT NULL DEFAULT 'wandering', -- wandering, seeking_food, carrying_food, fighting, fleeing, dead
    target_x DECIMAL(8,2),
    target_y DECIMAL(8,2),
    target_type VARCHAR(30), -- food_source, nest, enemy, obstacle
    target_id UUID,
    carried_resources JSONB DEFAULT '{}', -- {food_type: amount}
    traits JSONB, -- genetic traits: {speed_bonus: 0.1, strength_bonus: -0.05}
    energy INTEGER NOT NULL DEFAULT 100,
    mood VARCHAR(20) DEFAULT 'neutral', -- happy, scared, angry, excited
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Food sources in the world
CREATE TABLE food_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    food_type VARCHAR(30) NOT NULL, -- seeds, sugar, protein, fruit
    position_x DECIMAL(8,2) NOT NULL,
    position_y DECIMAL(8,2) NOT NULL,
    amount DECIMAL(8,2) NOT NULL,
    max_amount DECIMAL(8,2) NOT NULL,
    regeneration_rate DECIMAL(4,2) DEFAULT 0, -- amount per tick
    discovery_difficulty DECIMAL(3,2) DEFAULT 0.5, -- 0-1, how hard to find
    nutritional_value DECIMAL(4,2) NOT NULL DEFAULT 1.0,
    spoilage_rate DECIMAL(6,4) DEFAULT 0, -- decay per tick
    is_renewable BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Pheromone trails
CREATE TABLE pheromone_trails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    colony_id UUID NOT NULL REFERENCES colonies(id) ON DELETE CASCADE,
    trail_type VARCHAR(30) NOT NULL, -- food, danger, territory, recruitment
    position_x DECIMAL(8,2) NOT NULL,
    position_y DECIMAL(8,2) NOT NULL,
    strength DECIMAL(4,2) NOT NULL,
    decay_rate DECIMAL(6,4) NOT NULL DEFAULT 0.005,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    source_ant_id UUID REFERENCES ants(id),
    target_food_id UUID REFERENCES food_sources(id)
);

-- Environmental obstacles
CREATE TABLE obstacles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    obstacle_type VARCHAR(30) NOT NULL, -- rock, water, wall, nest_entrance
    shape VARCHAR(20) NOT NULL, -- circle, rectangle, polygon
    position_x DECIMAL(8,2) NOT NULL,
    position_y DECIMAL(8,2) NOT NULL,
    width DECIMAL(6,2),
    height DECIMAL(6,2),
    radius DECIMAL(6,2),
    polygon_points JSONB, -- for complex shapes: [{x: 10, y: 20}, ...]
    is_passable BOOLEAN DEFAULT false,
    movement_cost DECIMAL(3,2) DEFAULT 2.0, -- multiplier for crossing
    affects_pheromones BOOLEAN DEFAULT false,
    visual_properties JSONB -- {color: "#8B4513", opacity: 0.8}
);

-- Predators and other creatures
CREATE TABLE predators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    predator_type VARCHAR(30) NOT NULL, -- spider, bird, beetle
    position_x DECIMAL(8,2) NOT NULL,
    position_y DECIMAL(8,2) NOT NULL,
    angle DECIMAL(5,2) NOT NULL DEFAULT 0,
    speed DECIMAL(4,2) NOT NULL DEFAULT 0.5,
    detection_radius DECIMAL(6,2) NOT NULL DEFAULT 40.0,
    attack_radius DECIMAL(6,2) NOT NULL DEFAULT 10.0,
    health INTEGER NOT NULL DEFAULT 50,
    hunger INTEGER NOT NULL DEFAULT 0,
    state VARCHAR(30) DEFAULT 'patrolling', -- patrolling, hunting, eating, resting
    target_ant_id UUID REFERENCES ants(id),
    last_hunt_tick INTEGER DEFAULT 0,
    territory_center_x DECIMAL(8,2),
    territory_center_y DECIMAL(8,2),
    territory_radius DECIMAL(6,2) DEFAULT 80.0
);

-- Events and disasters
CREATE TABLE simulation_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    event_type VARCHAR(30) NOT NULL, -- flood, fire, food_abundance, predator_invasion
    severity DECIMAL(3,2) NOT NULL DEFAULT 1.0,
    center_x DECIMAL(8,2),
    center_y DECIMAL(8,2),
    radius DECIMAL(6,2),
    start_tick INTEGER NOT NULL,
    duration_ticks INTEGER,
    effects JSONB, -- {speed_modifier: 0.5, pheromone_decay: 2.0}
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Combat and interactions between ants
CREATE TABLE ant_interactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ant1_id UUID NOT NULL REFERENCES ants(id),
    ant2_id UUID NOT NULL REFERENCES ants(id),
    interaction_type VARCHAR(30) NOT NULL, -- fight, help, trade, recruit
    outcome VARCHAR(30), -- win, lose, draw, success, failure
    damage_dealt INTEGER DEFAULT 0,
    resources_exchanged JSONB,
    tick_occurred INTEGER NOT NULL,
    position_x DECIMAL(8,2),
    position_y DECIMAL(8,2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Genetic algorithm tracking
CREATE TABLE ant_genetics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ant_id UUID NOT NULL REFERENCES ants(id) ON DELETE CASCADE,
    parent1_id UUID REFERENCES ants(id),
    parent2_id UUID REFERENCES ants(id),
    generation INTEGER NOT NULL DEFAULT 1,
    genes JSONB NOT NULL, -- {speed: 0.12, strength: -0.05, intelligence: 0.08}
    fitness_score DECIMAL(8,2), -- calculated based on performance
    mutations JSONB, -- record of mutations that occurred
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Research/upgrades for colonies
CREATE TABLE colony_upgrades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    colony_id UUID NOT NULL REFERENCES colonies(id) ON DELETE CASCADE,
    upgrade_type VARCHAR(50) NOT NULL, -- faster_ants, better_carrying, stronger_pheromones
    level INTEGER NOT NULL DEFAULT 1,
    cost_paid JSONB, -- resources spent: {seeds: 100, protein: 50}
    effects JSONB, -- {speed_bonus: 0.1, carrying_bonus: 0.2}
    unlocked_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Performance tracking and analytics
CREATE TABLE simulation_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    tick_number INTEGER NOT NULL,
    total_ants INTEGER NOT NULL,
    total_food_collected DECIMAL(12,2) NOT NULL DEFAULT 0,
    total_distance_traveled DECIMAL(15,2) NOT NULL DEFAULT 0,
    pheromone_trail_count INTEGER NOT NULL DEFAULT 0,
    active_combats INTEGER NOT NULL DEFAULT 0,
    weather_effects_active INTEGER NOT NULL DEFAULT 0,
    average_ant_health DECIMAL(5,2),
    dominant_colony_id UUID REFERENCES colonies(id),
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- User-created custom scenarios/maps
CREATE TABLE scenario_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    creator_id VARCHAR(255), -- user ID if you have user system
    world_config JSONB NOT NULL, -- initial setup: obstacles, food, colonies
    difficulty_rating INTEGER DEFAULT 1, -- 1-5
    tags VARCHAR(255)[], -- {"survival", "competition", "puzzle"}
    is_public BOOLEAN DEFAULT false,
    play_count INTEGER DEFAULT 0,
    average_rating DECIMAL(3,2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_ants_colony_state ON ants(colony_id, state);
CREATE INDEX idx_ants_position ON ants(position_x, position_y);
CREATE INDEX idx_pheromone_trails_colony ON pheromone_trails(colony_id, trail_type);
CREATE INDEX idx_pheromone_trails_position ON pheromone_trails(position_x, position_y);
CREATE INDEX idx_pheromone_trails_expires ON pheromone_trails(expires_at);
CREATE INDEX idx_food_sources_simulation ON food_sources(simulation_id);
CREATE INDEX idx_food_sources_position ON food_sources(position_x, position_y);
CREATE INDEX idx_simulation_events_active ON simulation_events(simulation_id, is_active);
CREATE INDEX idx_simulation_stats_tick ON simulation_stats(simulation_id, tick_number);

-- Example trigger to update colony population count
CREATE OR REPLACE FUNCTION update_colony_population()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE colonies 
        SET population = population + 1 
        WHERE id = NEW.colony_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE colonies 
        SET population = population - 1 
        WHERE id = OLD.colony_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER colony_population_trigger
    AFTER INSERT OR DELETE ON ants
    FOR EACH ROW EXECUTE FUNCTION update_colony_population();

-- ===== ADVANCED ECOSYSTEM DYNAMICS =====

-- Plant and vegetation system
CREATE TABLE plants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    plant_type VARCHAR(30) NOT NULL, -- tree, bush, flower, grass, fungus
    species VARCHAR(50) NOT NULL, -- oak, rose, mushroom, etc.
    position_x DECIMAL(8,2) NOT NULL,
    position_y DECIMAL(8,2) NOT NULL,
    size DECIMAL(6,2) NOT NULL DEFAULT 5.0, -- current size/radius
    max_size DECIMAL(6,2) NOT NULL DEFAULT 20.0,
    growth_rate DECIMAL(4,2) NOT NULL DEFAULT 0.01, -- size increase per tick
    health DECIMAL(5,2) NOT NULL DEFAULT 100.0,
    age_ticks INTEGER NOT NULL DEFAULT 0,
    root_radius DECIMAL(6,2) NOT NULL DEFAULT 15.0, -- nutrient absorption area
    canopy_radius DECIMAL(6,2) NOT NULL DEFAULT 10.0, -- shade/protection area
    fruit_production_rate DECIMAL(4,2) DEFAULT 0, -- food units per tick when mature
    oxygen_production DECIMAL(4,2) DEFAULT 0.1, -- environmental benefit
    water_requirement DECIMAL(4,2) NOT NULL DEFAULT 1.0, -- water needed per tick
    nutrient_requirements JSONB, -- {nitrogen: 0.5, phosphorus: 0.3, potassium: 0.2}
    symbiotic_species VARCHAR(50)[], -- species that benefit this plant
    seasonal_behavior JSONB, -- {spring: "flowering", summer: "fruit", fall: "dormant"}
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Decomposer organisms (bacteria, fungi, etc.)
CREATE TABLE decomposers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    decomposer_type VARCHAR(30) NOT NULL, -- bacteria, fungi, earthworm
    position_x DECIMAL(8,2) NOT NULL,
    position_y DECIMAL(8,2) NOT NULL,
    radius DECIMAL(6,2) NOT NULL DEFAULT 3.0, -- area of effect
    efficiency DECIMAL(3,2) NOT NULL DEFAULT 0.1, -- decomposition rate
    nutrient_output JSONB, -- {nitrogen: 0.8, phosphorus: 0.6, carbon: 1.2}
    target_material VARCHAR(30), -- dead_ant, dead_plant, organic_waste
    population INTEGER NOT NULL DEFAULT 100,
    optimal_temperature DECIMAL(4,1), -- celsius
    optimal_ph DECIMAL(3,1), -- soil pH
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Other species in the ecosystem
CREATE TABLE species (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    species_type VARCHAR(30) NOT NULL, -- aphid, bird, beetle, spider
    species_name VARCHAR(50) NOT NULL,
    position_x DECIMAL(8,2) NOT NULL,
    position_y DECIMAL(8,2) NOT NULL,
    population INTEGER NOT NULL DEFAULT 1,
    mobility VARCHAR(20) NOT NULL, -- stationary, slow, medium, fast, flying
    diet_type VARCHAR(20) NOT NULL, -- herbivore, carnivore, omnivore, parasite
    symbiotic_relationships JSONB, -- {ant_colonies: ["mutualism"], plants: ["commensalism"]}
    territory_radius DECIMAL(6,2) DEFAULT 20.0,
    reproduction_rate DECIMAL(4,2) DEFAULT 0.001,
    mortality_rate DECIMAL(4,2) DEFAULT 0.001,
    food_requirements JSONB, -- daily nutritional needs
    environmental_preferences JSONB, -- temperature, humidity, pH ranges
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Disease and pathogen system
CREATE TABLE diseases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    disease_name VARCHAR(50) NOT NULL,
    pathogen_type VARCHAR(30) NOT NULL, -- virus, bacteria, fungus, parasite
    transmission_method VARCHAR(30) NOT NULL, -- contact, airborne, vector, soil
    transmission_rate DECIMAL(4,2) NOT NULL DEFAULT 0.1, -- probability per contact
    incubation_period INTEGER NOT NULL DEFAULT 100, -- ticks before symptoms
    mortality_rate DECIMAL(3,2) NOT NULL DEFAULT 0.05,
    recovery_rate DECIMAL(3,2) NOT NULL DEFAULT 0.1,
    immunity_duration INTEGER, -- ticks of immunity after recovery (null = permanent)
    affected_species VARCHAR(30)[], -- which species can get this disease
    symptoms JSONB, -- {speed_reduction: 0.5, carrying_reduction: 0.3}
    environmental_survival INTEGER DEFAULT 1000, -- ticks pathogen survives outside host
    mutation_rate DECIMAL(6,4) DEFAULT 0.0001,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Individual infection tracking
CREATE TABLE infections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    disease_id UUID NOT NULL REFERENCES diseases(id) ON DELETE CASCADE,
    host_id UUID NOT NULL, -- ant_id, species_id, etc.
    host_type VARCHAR(20) NOT NULL, -- ant, aphid, plant
    infection_stage VARCHAR(20) NOT NULL, -- incubating, symptomatic, recovering, immune
    infected_at_tick INTEGER NOT NULL,
    symptoms_start_tick INTEGER,
    recovery_tick INTEGER,
    transmission_events INTEGER DEFAULT 0, -- how many others this host infected
    strain_mutations JSONB, -- genetic variations of the pathogen
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ===== COMPLEX SOCIAL STRUCTURES =====

-- Specialized ant castes beyond basic types
CREATE TABLE ant_castes (
    id SERIAL PRIMARY KEY,
    caste_name VARCHAR(50) NOT NULL UNIQUE,
    specialization VARCHAR(50) NOT NULL, -- architect, farmer, guard, scout, diplomat, spy
    base_attributes JSONB NOT NULL, -- enhanced stats for specialization
    special_abilities JSONB, -- {can_build: true, stealth_bonus: 0.3, negotiation_skill: 0.8}
    training_requirements JSONB, -- {experience_ticks: 5000, mentor_required: true}
    population_cap_percentage DECIMAL(4,2) DEFAULT 0.1, -- max % of colony that can be this caste
    unlock_conditions JSONB, -- {colony_size: 100, tech_level: 2}
    maintenance_cost JSONB -- {food_per_tick: 1.5, special_resources: {...}}
);

-- Diplomatic relationships between colonies
CREATE TABLE colony_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    colony1_id UUID NOT NULL REFERENCES colonies(id) ON DELETE CASCADE,
    colony2_id UUID NOT NULL REFERENCES colonies(id) ON DELETE CASCADE,
    relationship_type VARCHAR(30) NOT NULL, -- allied, neutral, hostile, trading, vassal
    trust_level DECIMAL(4,2) NOT NULL DEFAULT 0.0, -- -1.0 to 1.0
    trade_agreements JSONB, -- {food_exchange_rate: 1.2, territory_access: true}
    military_pacts JSONB, -- {mutual_defense: true, joint_operations: false}
    territorial_agreements JSONB, -- {shared_foraging_areas: [...], buffer_zones: [...]}
    last_interaction_tick INTEGER,
    relationship_history JSONB[], -- array of historical events
    established_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(colony1_id, colony2_id)
);

-- Cultural traits and traditions that colonies develop
CREATE TABLE colony_culture (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    colony_id UUID NOT NULL REFERENCES colonies(id) ON DELETE CASCADE,
    cultural_trait VARCHAR(50) NOT NULL, -- aggressive_expansion, peaceful_trading, technological_focus
    trait_strength DECIMAL(3,2) NOT NULL DEFAULT 1.0, -- how strong this trait is
    origin_story JSONB, -- how this trait developed
    behavioral_modifiers JSONB, -- {aggression_bonus: 0.2, trade_efficiency: 1.3}
    ritual_behaviors JSONB, -- {food_ceremonies: true, war_dances: false}
    knowledge_traditions JSONB, -- {oral_history: true, landmark_memory: true}
    innovation_rate DECIMAL(4,2) DEFAULT 0.1, -- how quickly culture changes
    influence_radius DECIMAL(6,2) DEFAULT 50.0, -- how far culture spreads
    developed_at_tick INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Espionage and intelligence operations
CREATE TABLE espionage_missions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spy_ant_id UUID NOT NULL REFERENCES ants(id) ON DELETE CASCADE,
    origin_colony_id UUID NOT NULL REFERENCES colonies(id) ON DELETE CASCADE,
    target_colony_id UUID NOT NULL REFERENCES colonies(id) ON DELETE CASCADE,
    mission_type VARCHAR(30) NOT NULL, -- reconnaissance, sabotage, theft, assassination
    mission_status VARCHAR(20) NOT NULL DEFAULT 'planning', -- planning, active, completed, failed, compromised
    objectives JSONB NOT NULL, -- {steal_food: 100, map_defenses: true, eliminate_target: "queen"}
    cover_identity VARCHAR(50), -- how spy is disguised
    discovery_risk DECIMAL(3,2) NOT NULL DEFAULT 0.1, -- chance of being caught per tick
    intelligence_gathered JSONB, -- information collected during mission
    resources_stolen JSONB, -- actual loot obtained
    started_at_tick INTEGER,
    completed_at_tick INTEGER,
    success_rating DECIMAL(3,2), -- 0-1 based on objectives achieved
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Colony migration patterns and seasonal movements
CREATE TABLE migration_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    colony_id UUID NOT NULL REFERENCES colonies(id) ON DELETE CASCADE,
    pattern_type VARCHAR(30) NOT NULL, -- seasonal, resource_depletion, threat_avoidance
    trigger_conditions JSONB NOT NULL, -- {temperature_below: 5, food_scarcity: 0.1, predator_pressure: 0.8}
    destination_preferences JSONB, -- {near_water: true, elevation_range: [100, 300], soil_ph: [6, 8]}
    migration_routes JSONB[], -- array of waypoints: [{x: 100, y: 200, rest_duration: 500}]
    preparation_time INTEGER DEFAULT 1000, -- ticks needed to prepare for migration
    migration_speed DECIMAL(4,2) DEFAULT 0.5, -- movement speed during migration
    survival_rate DECIMAL(3,2) DEFAULT 0.9, -- percentage of colony that survives migration
    last_migration_tick INTEGER,
    seasonal_schedule JSONB, -- when migrations typically occur
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ===== ENVIRONMENTAL COMPLEXITY =====

-- Water system (rivers, ponds, rain puddles)
CREATE TABLE water_bodies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    water_type VARCHAR(30) NOT NULL, -- river, pond, puddle, stream, flood_zone
    shape VARCHAR(20) NOT NULL, -- circle, rectangle, polygon, line
    center_x DECIMAL(8,2) NOT NULL,
    center_y DECIMAL(8,2) NOT NULL,
    width DECIMAL(6,2),
    length DECIMAL(6,2),
    radius DECIMAL(6,2),
    polygon_points JSONB, -- for complex shapes
    depth DECIMAL(4,2) NOT NULL DEFAULT 1.0, -- affects crossability
    flow_direction DECIMAL(5,2), -- radians, for rivers/streams
    flow_speed DECIMAL(4,2) DEFAULT 0, -- affects ant movement
    water_quality DECIMAL(3,2) DEFAULT 1.0, -- 0-1, affects health
    evaporation_rate DECIMAL(6,4) DEFAULT 0.001, -- for puddles
    is_seasonal BOOLEAN DEFAULT false, -- disappears in dry season
    temperature DECIMAL(4,1), -- affects nearby micro-climate
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Fire and burning mechanics
CREATE TABLE fire_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    center_x DECIMAL(8,2) NOT NULL,
    center_y DECIMAL(8,2) NOT NULL,
    radius DECIMAL(6,2) NOT NULL DEFAULT 10.0,
    intensity DECIMAL(3,2) NOT NULL DEFAULT 1.0, -- 0-1, affects spread rate
    fuel_remaining DECIMAL(6,2) NOT NULL DEFAULT 100.0, -- how long fire can burn
    spread_rate DECIMAL(4,2) NOT NULL DEFAULT 0.1, -- radius increase per tick
    wind_influence DECIMAL(3,2) DEFAULT 0.5, -- how much wind affects spread
    started_at_tick INTEGER NOT NULL,
    extinguished_at_tick INTEGER,
    ignition_source VARCHAR(30), -- lightning, human, spontaneous, other_fire
    suppression_efforts JSONB, -- {water_applied: 50, firebreaks: [...]}
    casualties JSONB, -- {ants_killed: 15, plants_destroyed: 8}
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Soil composition and chemistry
CREATE TABLE soil_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    zone_name VARCHAR(50),
    center_x DECIMAL(8,2) NOT NULL,
    center_y DECIMAL(8,2) NOT NULL,
    radius DECIMAL(6,2) NOT NULL DEFAULT 50.0,
    soil_type VARCHAR(30) NOT NULL, -- clay, sand, loam, rocky, organic
    ph_level DECIMAL(3,1) NOT NULL DEFAULT 7.0, -- 0-14 scale
    nutrients JSONB NOT NULL, -- {nitrogen: 0.8, phosphorus: 0.6, potassium: 0.7, carbon: 1.2}
    moisture_content DECIMAL(3,2) NOT NULL DEFAULT 0.5, -- 0-1 scale
    compaction DECIMAL(3,2) NOT NULL DEFAULT 0.3, -- affects digging difficulty
    temperature DECIMAL(4,1), -- soil temperature
    microbial_activity DECIMAL(3,2) DEFAULT 0.5, -- affects decomposition rates
    drainage_rate DECIMAL(4,2) DEFAULT 0.1, -- water absorption/runoff
    contamination_level DECIMAL(3,2) DEFAULT 0.0, -- pollutants/toxins
    fertility_score DECIMAL(3,2), -- calculated overall fertility
    last_updated_tick INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Micro-climate zones with different environmental conditions
CREATE TABLE climate_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    zone_name VARCHAR(50),
    center_x DECIMAL(8,2) NOT NULL,
    center_y DECIMAL(8,2) NOT NULL,
    radius DECIMAL(6,2) NOT NULL DEFAULT 75.0,
    temperature DECIMAL(4,1) NOT NULL, -- celsius
    humidity DECIMAL(3,2) NOT NULL, -- 0-1 scale
    wind_speed DECIMAL(4,2) DEFAULT 0, -- affects evaporation, fire spread
    wind_direction DECIMAL(5,2) DEFAULT 0, -- radians
    light_level DECIMAL(3,2) NOT NULL DEFAULT 1.0, -- 0-1, affects plant growth
    air_pressure DECIMAL(6,2) DEFAULT 1013.25, -- millibars
    seasonal_variations JSONB, -- how conditions change with seasons
    elevation DECIMAL(6,2) DEFAULT 0, -- meters above sea level
    vegetation_cover DECIMAL(3,2) DEFAULT 0.5, -- affects local conditions
    created_by VARCHAR(30), -- plant_canopy, water_body, elevation, artificial
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Weather patterns and atmospheric events
CREATE TABLE weather_systems (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    simulation_id UUID NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    weather_type VARCHAR(30) NOT NULL, -- rain, storm, drought, wind, fog
    center_x DECIMAL(8,2),
    center_y DECIMAL(8,2),
    radius DECIMAL(8,2), -- area of effect (null for global weather)
    intensity DECIMAL(3,2) NOT NULL DEFAULT 1.0,
    movement_vector_x DECIMAL(4,2) DEFAULT 0, -- movement direction/speed
    movement_vector_y DECIMAL(4,2) DEFAULT 0,
    duration_remaining INTEGER NOT NULL, -- ticks until weather ends
    effects JSONB NOT NULL, -- {visibility_reduction: 0.3, movement_penalty: 0.2, water_added: 10}
    pressure_change DECIMAL(5,2), -- affects animal behavior
    started_at_tick INTEGER NOT NULL,
    forecast_accuracy DECIMAL(3,2) DEFAULT 1.0, -- for realistic weather prediction
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Enhanced indexes for new tables
CREATE INDEX idx_plants_position ON plants(position_x, position_y);
CREATE INDEX idx_plants_simulation ON plants(simulation_id, plant_type);
CREATE INDEX idx_decomposers_position ON decomposers(position_x, position_y);
CREATE INDEX idx_species_position ON species(position_x, position_y);
CREATE INDEX idx_infections_host ON infections(host_id, host_type, infection_stage);
CREATE INDEX idx_colony_relations_colonies ON colony_relations(colony1_id, colony2_id);
CREATE INDEX idx_espionage_missions_active ON espionage_missions(mission_status, target_colony_id);
CREATE INDEX idx_water_bodies_position ON water_bodies(center_x, center_y);
CREATE INDEX idx_fire_zones_active ON fire_zones(simulation_id) WHERE extinguished_at_tick IS NULL;
CREATE INDEX idx_soil_zones_position ON soil_zones(center_x, center_y);
CREATE INDEX idx_climate_zones_position ON climate_zones(center_x, center_y);

-- Enhanced view for ecosystem health
CREATE VIEW ecosystem_health AS
SELECT 
    s.id as simulation_id,
    s.name,
    COUNT(DISTINCT c.id) as active_colonies,
    COUNT(DISTINCT a.id) as total_ants,
    COUNT(DISTINCT p.id) as plant_count,
    COUNT(DISTINCT sp.id) as other_species_count,
    COUNT(DISTINCT d.id) as active_diseases,
    AVG(sz.fertility_score) as avg_soil_fertility,
    COUNT(DISTINCT wb.id) as water_sources,
    COUNT(DISTINCT fz.id) as active_fires,
    AVG(cz.temperature) as avg_temperature,
    AVG(cz.humidity) as avg_humidity
FROM simulations s
LEFT JOIN colonies c ON s.id = c.simulation_id AND c.is_active = true
LEFT JOIN ants a ON c.id = a.colony_id AND a.state != 'dead'
LEFT JOIN plants p ON s.id = p.simulation_id
LEFT JOIN species sp ON s.id = sp.simulation_id
LEFT JOIN diseases d ON s.id = d.simulation_id
LEFT JOIN soil_zones sz ON s.id = sz.simulation_id
LEFT JOIN water_bodies wb ON s.id = wb.simulation_id
LEFT JOIN fire_zones fz ON s.id = fz.simulation_id AND fz.extinguished_at_tick IS NULL
LEFT JOIN climate_zones cz ON s.id = cz.simulation_id
GROUP BY s.id, s.name;

-- Example view for colony performance analytics
CREATE VIEW colony_performance AS
SELECT 
    c.id,
    c.name,
    c.population,
    c.resources,
    COUNT(DISTINCT fs.id) as nearby_food_sources,
    AVG(a.health) as avg_ant_health,
    COUNT(CASE WHEN a.state = 'carrying_food' THEN 1 END) as ants_with_food,
    COUNT(DISTINCT pt.id) as active_pheromone_trails
FROM colonies c
LEFT JOIN ants a ON c.id = a.colony_id AND a.state != 'dead'
LEFT JOIN food_sources fs ON fs.simulation_id IN (
    SELECT simulation_id FROM colonies WHERE id = c.id
) AND sqrt(power(fs.position_x - c.center_x, 2) + power(fs.position_y - c.center_y, 2)) < c.territory_radius
LEFT JOIN pheromone_trails pt ON c.id = pt.colony_id AND pt.strength > 0.1
GROUP BY c.id, c.name, c.population, c.resources;