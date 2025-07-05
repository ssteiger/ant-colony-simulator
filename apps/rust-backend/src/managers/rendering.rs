use bevy::prelude::*;
use crate::models::*;

/// Plugin for rendering ant colony simulation entities
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_rendering)
            .add_systems(Update, (
                update_ant_rendering,
                update_colony_rendering,
                update_food_source_rendering,
                update_pheromone_rendering,
                camera_system,
                ui_system,
            ));
    }
}

/// Setup rendering components and camera
fn setup_rendering(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0)),
        ..default()
    });

    // Add background
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.2, 0.4, 0.1), // Dark green background
            custom_size: Some(Vec2::new(1000.0, 1000.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        ..default()
    });

    // Add UI camera
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1001.0)),
        ..default()
    });

    // Add UI root
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        // Stats panel
        parent.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
            ..default()
        }).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Ant Colony Simulation",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                }
            ));
            
            parent.spawn(TextBundle::from_section(
                "Tick: 0 | Ants: 0 | Colonies: 0",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                }
            ));
        });
    });
}

/// Camera system for following and zooming
fn camera_system(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        let mut movement = Vec3::ZERO;
        let speed = 200.0 * time.delta_seconds();

        if keyboard_input.pressed(KeyCode::W) {
            movement.y += speed;
        }
        if keyboard_input.pressed(KeyCode::S) {
            movement.y -= speed;
        }
        if keyboard_input.pressed(KeyCode::A) {
            movement.x -= speed;
        }
        if keyboard_input.pressed(KeyCode::D) {
            movement.x += speed;
        }

        camera_transform.translation += movement;
    }
}

/// UI system to update stats display
fn ui_system(
    simulation_stats: Res<SimulationStats>,
    mut text_query: Query<&mut Text>,
) {
    for mut text in text_query.iter_mut() {
        if text.sections.len() > 1 {
            text.sections[1].value = format!(
                "Tick: {} | Ants: {} | Colonies: {} | Food: {:.1}",
                simulation_stats.current_tick,
                simulation_stats.total_ants,
                simulation_stats.active_colonies,
                simulation_stats.total_food_collected
            );
        }
    }
}

/// Update ant visual components
fn update_ant_rendering(
    mut commands: Commands,
    ants: Query<(Entity, &AntPhysics, &AntType, &AntState), (With<Ant>, Without<Sprite>)>,
) {
    for (entity, physics, ant_type, _state) in ants.iter() {
        // Add sprite component to ant
        commands.entity(entity).insert(SpriteBundle {
            sprite: Sprite {
                color: Color::hsl(ant_type.color_hue, 0.8, 0.6),
                custom_size: Some(Vec2::new(4.0, 6.0)), // Small ant size
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                physics.position.x,
                physics.position.y,
                10.0, // Above background
            )).with_rotation(Quat::from_rotation_z(physics.rotation)),
            ..default()
        });
    }
}

/// Update colony visual components
fn update_colony_rendering(
    mut commands: Commands,
    colonies: Query<(Entity, &ColonyProperties), (With<Colony>, Without<Sprite>)>,
) {
    for (entity, properties) in colonies.iter() {
        // Add colony sprite
        commands.entity(entity).insert(SpriteBundle {
            sprite: Sprite {
                color: Color::hsl(properties.color_hue, 0.9, 0.5),
                custom_size: Some(Vec2::new(properties.radius * 2.0, properties.radius * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                properties.center.x,
                properties.center.y,
                5.0,
            )),
            ..default()
        });

        // Add territory indicator (larger, semi-transparent circle)
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::hsla(properties.color_hue, 0.3, 0.3, 0.2),
                custom_size: Some(Vec2::new(properties.territory_radius * 2.0, properties.territory_radius * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                properties.center.x,
                properties.center.y,
                5.0,
            )),
            ..default()
        });
    }
}

/// Update food source visual components
fn update_food_source_rendering(
    mut commands: Commands,
    food_sources: Query<(Entity, &FoodSourceProperties, &Transform), (With<FoodSource>, Without<Sprite>)>,
) {
    for (entity, properties, transform) in food_sources.iter() {
        // Calculate food source size based on amount
        let size = (properties.amount / properties.max_amount * 20.0).max(5.0);
        
        // Choose color based on food type
        let color = match properties.food_type.as_str() {
            "berries" => Color::rgb(0.8, 0.2, 0.2), // Red
            "leaves" => Color::rgb(0.2, 0.8, 0.2),  // Green
            "seeds" => Color::rgb(0.8, 0.8, 0.2),   // Yellow
            _ => Color::rgb(0.6, 0.4, 0.2),         // Brown
        };

        commands.entity(entity).insert(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                transform.translation.x,
                transform.translation.y,
                5.0,
            )),
            ..default()
        });
    }
}

/// Update pheromone trail visual components
fn update_pheromone_rendering(
    mut commands: Commands,
    pheromones: Query<(Entity, &PheromoneProperties, &Transform), (With<PheromoneTrail>, Without<Sprite>)>,
) {
    for (entity, properties, transform) in pheromones.iter() {
        // Choose color based on pheromone type
        let color = match properties.trail_type {
            PheromoneType::Food => Color::rgba(0.2, 0.8, 0.2, 0.3),     // Green
            PheromoneType::Danger => Color::rgba(0.8, 0.2, 0.2, 0.3),   // Red
            PheromoneType::Home => Color::rgba(0.2, 0.2, 0.8, 0.3),     // Blue
            PheromoneType::Exploration => Color::rgba(0.8, 0.8, 0.2, 0.3), // Yellow
        };

        // Size based on strength
        let size = (properties.strength / properties.max_strength * 8.0).max(1.0);

        commands.entity(entity).insert(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            transform: transform.clone(),
            ..default()
        });
    }
} 