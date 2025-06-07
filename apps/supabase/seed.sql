-- Seed data for Ant Colony Simulator

-- Insert base ant types
INSERT INTO ant_types (
    name, 
    base_speed, 
    base_strength, 
    base_health, 
    base_size, 
    lifespan_ticks, 
    carrying_capacity, 
    role, 
    color_hue, 
    special_abilities, 
    food_preferences
) VALUES 
-- Worker Ant: Balanced stats, good at foraging and building
(
    'Worker', 
    2, -- moderate speed
    1, -- basic strength
    80, -- moderate health
    3, -- standard size
    40000, -- moderate lifespan
    2, -- good carrying capacity
    'worker', 
    30, -- brown/orange hue
    '{"vision_range": 40, "can_build": true, "can_dig": true, "foraging_efficiency": 1.2}',
    '{"seeds": 1.0, "sugar": 1.2, "protein": 0.8, "fruit": 1.1}'
),

-- Soldier Ant: High combat stats, defensive role
(
    'Soldier', 
    1, -- slower speed
    4, -- high strength
    150, -- high health
    5, -- larger size
    35000, -- shorter lifespan due to combat
    1, -- limited carrying capacity
    'soldier', 
    0, -- red hue
    '{"vision_range": 45, "can_fight": true, "combat_bonus": 2.0, "intimidation": 1.5, "armor": 1.3}',
    '{"seeds": 0.8, "sugar": 0.9, "protein": 1.5, "fruit": 0.7}'
),

-- Scout Ant: Fast and far-seeing, exploration specialist
(
    'Scout', 
    4, -- high speed
    1, -- basic strength
    60, -- lower health
    2, -- smaller size
    30000, -- shorter lifespan due to risks
    1, -- basic carrying capacity
    'scout', 
    60, -- yellow/green hue
    '{"vision_range": 80, "stealth": 1.4, "pathfinding": 1.5, "pheromone_sensitivity": 1.3, "danger_detection": 1.6}',
    '{"seeds": 1.1, "sugar": 1.4, "protein": 0.9, "fruit": 1.2}'
),

-- Queen Ant: Reproductive, long-lived, heavily protected
(
    'Queen', 
    1, -- very slow
    2, -- moderate strength
    300, -- very high health
    8, -- largest size
    200000, -- very long lifespan
    0, -- no carrying (focused on reproduction)
    'queen', 
    270, -- purple/magenta hue
    '{"vision_range": 60, "reproduction": true, "pheromone_production": 2.0, "leadership": 2.5, "egg_laying": true}',
    '{"seeds": 1.2, "sugar": 1.8, "protein": 2.0, "fruit": 1.5}'
),

-- Nurse Ant: Care specialist, tends to young and injured
(
    'Nurse', 
    2, -- moderate speed
    1, -- basic strength
    100, -- standard health
    3, -- standard size
    45000, -- good lifespan
    1, -- basic carrying capacity
    'nurse', 
    180, -- blue/cyan hue
    '{"vision_range": 35, "healing": 1.8, "larva_care": 2.0, "food_distribution": 1.4, "disease_resistance": 1.3}',
    '{"seeds": 0.9, "sugar": 1.1, "protein": 1.3, "fruit": 1.0}'
);
