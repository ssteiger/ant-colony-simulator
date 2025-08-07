use bevy::prelude::*;
use crate::models::*;

use bevy::asset::AssetServer;
use bevy::ecs::system::Resource;
use bevy::input::mouse::{MouseWheel, MouseMotion};
use bevy::render::camera::RenderTarget;
use bevy::window::WindowRef;

/// Marker component for the main game camera
#[derive(Component)]
pub struct MainCamera;

/// Resource to store the ant texture handle
#[derive(Resource, Clone)]
pub struct AntTexture(pub Handle<Image>);

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
                cleanup_territory_indicators,
            ).after(crate::managers::ant_behavior::despawn_dead_ants_system));
    }
}

/// Setup rendering components and camera
fn setup_rendering(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    // Get the window size for proper scaling
    let window = windows.single();
    let window_width = window.width();
    let window_height = window.height();
    
    // Main game camera - this is the camera that will be controlled
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0), // Start at world origin
            camera: Camera {
                order: 0, // Main camera renders first
                target: RenderTarget::Window(WindowRef::Primary),
                ..default()
            },
            projection: OrthographicProjection {
                scale: 1.0,
                near: -1000.0,
                far: 1000.0,
                ..default()
            },
            ..default()
        },
        MainCamera, // Marker component
    ));

    // Add background that fills the entire window
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::WHITE, // White background
            custom_size: Some(Vec2::new(window_width * 2.0, window_height * 2.0)), // Make it larger to prevent edge artifacts
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        ..default()
    });

    // Add UI root - this will use the same camera
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        z_index: ZIndex::Global(1000), // Ensure UI is on top
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
            background_color: Color::rgba(1.0, 1.0, 1.0, 0.9).into(),
            z_index: ZIndex::Local(1),
            ..default()
        }).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Ant Colony Simulation ",
                TextStyle {
                    font_size: 16.0,
                    color: Color::BLACK,
                    ..default()
                }
            ));
            
            parent.spawn(TextBundle::from_section(
                "Tick: 0 | Ants: 0 | Colonies: 0",
                TextStyle {
                    font_size: 16.0,
                    color: Color::BLACK,
                    ..default()
                }
            ));
        });
    });

    // Load ant.png texture and insert as resource
    let ant_texture: Handle<Image> = asset_server.load("ant.png");
    info!("Loading ant texture: ant.png");
    info!("Ant texture handle: {:?}", ant_texture);
    commands.insert_resource(AntTexture(ant_texture));
    
    // Force asset server to start loading the texture immediately
    let _: Handle<Image> = asset_server.load("ant.png");
}

/// Camera system for zooming and panning
fn camera_system(
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    time: Res<Time>,
    windows: Query<&Window>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_query.get_single_mut() else {
        warn!("Camera system: No main camera found!");
        return;
    };
    
    let _window = windows.single();
    
    // Handle mouse wheel zooming
    for wheel_event in mouse_wheel.iter() {
        let zoom_delta = wheel_event.y * 0.1; // Scale the zoom speed
        let zoom_factor = if zoom_delta > 0.0 { 0.9 } else { 1.1 };
        let old_scale = projection.scale;
        projection.scale = (projection.scale * zoom_factor).clamp(0.1, 10.0);
        
        /*
        // Log zoom changes for debugging
        if (old_scale - projection.scale).abs() > 0.001 {
            info!("Zoom changed from {:.2} to {:.2} (wheel delta: {:.2})", old_scale, projection.scale, wheel_event.y);
        }
        */
    }
    
    // Handle mouse panning (when left mouse button is held)
    if mouse_input.pressed(MouseButton::Left) {
        for motion_event in mouse_motion.iter() {
            // Pan speed should be relative to zoom level
            let pan_speed = projection.scale * 1.0;
            let delta_x = -motion_event.delta.x * pan_speed;
            let delta_y = motion_event.delta.y * pan_speed; // Y is inverted in screen space
            
            camera_transform.translation.x += delta_x;
            camera_transform.translation.y += delta_y;
            
            /*
            info!("Pan delta: ({:.1}, {:.1}), Camera position: ({:.0}, {:.0})", 
                  delta_x, delta_y, camera_transform.translation.x, camera_transform.translation.y);
            */
        }
    }
    
    // Manual camera movement with WASD keys
    let mut movement = Vec3::ZERO;
    let speed = 400.0 * time.delta_seconds() * projection.scale; // Speed relative to zoom

    if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
        movement.y += speed;
    }
    if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
        movement.y -= speed;
    }
    if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
        movement.x -= speed;
    }
    if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
        movement.x += speed;
    }

    if movement != Vec3::ZERO {
        camera_transform.translation += movement;
        info!("Keyboard movement: ({:.1}, {:.1}), Camera position: ({:.0}, {:.0})", 
              movement.x, movement.y, camera_transform.translation.x, camera_transform.translation.y);
    }
    
    // Optional: Reset camera to center with R key
    if keyboard_input.just_pressed(KeyCode::R) {
        camera_transform.translation.x = 0.0;
        camera_transform.translation.y = 0.0;
        projection.scale = 1.0;
        info!("Camera reset to center (0, 0) with scale 1.0");
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
                "Tick: {} | Ants: {} | Colonies: {} | Food: {:.1} | Pheromones: {}",
                simulation_stats.current_tick,
                simulation_stats.total_ants,
                simulation_stats.active_colonies,
                simulation_stats.total_food_collected,
                simulation_stats.pheromone_trail_count
            );
        }
    }
}

/// Update ant visual components
fn update_ant_rendering(
    mut commands: Commands,
    ants: Query<(Entity, &AntPhysics, &AntType, &AntState), (With<Ant>, Without<Sprite>)>,
    ant_texture: Res<AntTexture>,
    asset_server: Res<AssetServer>,
) {
    for (entity, physics, ant_type, _state) in ants.iter() {
        // Check if ant texture is loaded
        let load_state = asset_server.get_load_state(ant_texture.0.clone());
        
        match load_state {
            bevy::asset::LoadState::Loaded => {
                // Texture is loaded - use ant.png for all ants
                // Apply different tints based on ant role while keeping the texture
                let tint_color = match ant_type.role.as_str() {
                    "worker" => Color::rgb(1.0, 0.9, 0.7),   // Slight warm tint
                    "soldier" => Color::rgb(1.0, 0.8, 0.8), // Slight red tint
                    "scout" => Color::rgb(0.8, 0.9, 1.0),   // Slight blue tint
                    "queen" => Color::rgb(1.0, 0.8, 1.0),   // Slight purple tint
                    _ => Color::WHITE,                       // Default white (no tint)
                };
                
                commands.entity(entity).insert(SpriteBundle {
                    texture: ant_texture.0.clone(),
                    sprite: Sprite {
                        color: tint_color, // Apply role-based tint to the texture
                        custom_size: Some(Vec2::new(16.0, 16.0)), // Made ants half the original size
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        physics.position.x,
                        physics.position.y,
                        50.0, // Higher Z to ensure it's above background
                    )).with_rotation(Quat::from_rotation_z(physics.rotation)),
                    ..default()
                });
            }
            bevy::asset::LoadState::Loading => {
                // Texture is still loading - don't render anything yet, wait for next frame
                debug!("Ant texture still loading, waiting...");
            }
            bevy::asset::LoadState::Failed => {
                // Texture failed to load - try to reload it
                warn!("Ant texture failed to load, attempting to reload ant.png");
                let new_texture: Handle<Image> = asset_server.load("ant.png");
                commands.insert_resource(AntTexture(new_texture));
            }
            _ => {
                // Other states (NotLoaded, etc.) - texture might not be available yet
                debug!("Ant texture not available, state: {:?}", load_state);
            }
        }
    }
}

/// Update colony visual components
fn update_colony_rendering(
    mut commands: Commands,
    colonies: Query<(Entity, &ColonyProperties), (With<Colony>, Without<Sprite>)>,
) {
    for (entity, properties) in colonies.iter() {
        // Create circular sprite for colony with rounded corners effect
        let sprite = Sprite {
            color: Color::hsl(properties.color_hue, 0.9, 0.5),
            custom_size: Some(Vec2::new(properties.radius * 2.0, properties.radius * 2.0)),
            ..default()
        };
        
        // Use multiple smaller sprites to create a circular appearance
        let main_size = properties.radius * 2.0;
        let overlay_size = properties.radius * 1.8;
        
        commands.entity(entity).insert(SpriteBundle {
            sprite: Sprite {
                color: Color::hsl(properties.color_hue, 0.9, 0.5),
                custom_size: Some(Vec2::new(main_size, main_size)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                properties.center.x,
                properties.center.y,
                5.0,
            )),
            ..default()
        });
        
        // Add a slightly smaller, more opaque overlay to create circular effect
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::hsl(properties.color_hue, 0.9, 0.6).with_a(0.8),
                custom_size: Some(Vec2::new(overlay_size, overlay_size)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                properties.center.x,
                properties.center.y,
                5.1,
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

        // Create circular-looking sprites with layered effect
        let main_size = size;
        let overlay_size = size * 0.8;
        let center_size = size * 0.6;
        
        commands.entity(entity).insert(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(main_size, main_size)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                transform.translation.x,
                transform.translation.y,
                5.0,
            )),
            ..default()
        });
        
        // Add lighter overlay for circular effect
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: color.with_a(0.7),
                custom_size: Some(Vec2::new(overlay_size, overlay_size)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                transform.translation.x,
                transform.translation.y,
                5.1,
            )),
            ..default()
        });
        
        // Add bright center highlight
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE.with_a(0.3),
                custom_size: Some(Vec2::new(center_size, center_size)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                transform.translation.x,
                transform.translation.y,
                5.2,
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
        // Calculate intensity based on strength (0.0 to 1.0)
        let intensity = (properties.strength / properties.max_strength).clamp(0.0, 1.0);
        
        // Choose color based on pheromone type with improved visibility
        let base_color = match properties.trail_type {
            PheromoneType::Food => Color::rgb(0.2, 0.9, 0.2),     // Bright Green
            PheromoneType::Danger => Color::rgb(0.9, 0.1, 0.1),   // Bright Red
            PheromoneType::Home => Color::rgb(0.1, 0.4, 0.9),     // Bright Blue
            PheromoneType::Exploration => Color::rgb(0.9, 0.9, 0.1), // Bright Yellow
        };
        
        // Apply alpha based on intensity for fading effect
        let alpha = (intensity * 0.8 + 0.2).min(1.0); // Alpha from 0.2 to 1.0
        let color = Color::rgba(base_color.r(), base_color.g(), base_color.b(), alpha);

        // Size based on strength and type - make pheromones more visible
        let base_size = match properties.trail_type {
            PheromoneType::Food => 6.0,        // Food trails are larger
            PheromoneType::Home => 5.0,        // Home trails medium
            PheromoneType::Exploration => 3.0, // Exploration trails smaller
            PheromoneType::Danger => 8.0,      // Danger trails largest for visibility
        };
        
        let size = (base_size * intensity).max(1.5); // Minimum size for visibility

        commands.entity(entity).insert(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                transform.translation.x,
                transform.translation.y,
                10.0, // Place pheromones above background but below ants
            )),
            ..default()
        });
    }
}

/// Clean up orphaned territory indicators
fn cleanup_territory_indicators(
    mut commands: Commands,
    colonies: Query<Entity, With<Colony>>,
    territory_indicators: Query<Entity, (With<Sprite>, Without<Colony>, Without<Ant>, Without<FoodSource>)>,
) {
    // This system would clean up territory indicators that are no longer needed
    // For now, we'll just log if we find any orphaned sprites
    for entity in territory_indicators.iter() {
        debug!("Found orphaned sprite entity {:?}, will be cleaned up by Bevy", entity);
    }
}

 