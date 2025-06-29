# Ant Colony Simulator - Bevy + Big Brain Backend

A high-performance ant colony simulation built with Rust, Bevy game engine, and Big Brain AI system.

## Features

- **Bevy ECS**: Entity-Component-System architecture for efficient simulation
- **Big Brain AI**: Advanced behavior tree system for ant decision making
- **Real-time WebSocket**: Live simulation updates to connected clients
- **Database Persistence**: PostgreSQL backend for simulation state
- **Modular Design**: Separate systems for ant behavior, colony management, environment, and pheromones

## Architecture

### Core Components

- **Ant**: Individual ant entities with physics, health, and AI
- **Colony**: Ant colonies with resources, population, and territory
- **Food Source**: Renewable food sources that ants can collect
- **Pheromone Trail**: Chemical trails that guide ant behavior

### Systems

1. **Ant Behavior Systems** (`managers/ant_behavior.rs`)
   - Movement and physics
   - Health and energy management
   - Big Brain AI decision making
   - Food and colony interactions

2. **Colony Management** (`managers/colony.rs`)
   - Population spawning
   - Resource management
   - Colony upgrades
   - Territory conflicts

3. **Environment Systems** (`managers/environment.rs`)
   - Food regeneration
   - Weather effects
   - Day/night cycles
   - Seasonal changes

4. **Pheromone Systems** (`managers/pheromone.rs`)
   - Trail creation and decay
   - Pheromone detection
   - Trail merging
   - Danger avoidance

## Big Brain AI

The simulation uses Big Brain for sophisticated ant behavior:

### Actions
- `SeekFoodAction`: Find and collect food
- `FollowPheromoneAction`: Follow chemical trails
- `ReturnToColonyAction`: Return to colony with resources
- `ExploreAction`: Explore new areas

### Scorers
- `FoodSeekingScorer`: Evaluates hunger and carrying capacity
- `PheromoneFollowingScorer`: Evaluates pheromone sensitivity
- `ReturnToColonyScorer`: Evaluates energy and resource load
- `ExplorationScorer`: Evaluates energy levels for exploration

### Behavior Tree
```rust
Thinker::new()
    .picker(FirstToScore { threshold: 0.8 })
    .when("Seek Food", FoodSeekingScorer, SeekFoodAction)
    .when("Follow Pheromone", PheromoneFollowingScorer, FollowPheromoneAction)
    .when("Return to Colony", ReturnToColonyScorer, ReturnToColonyAction)
    .when("Explore", ExplorationScorer, ExploreAction)
```

## Installation

### Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- Cargo

### Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd ant-colony-simulator/apps/rust-backend
   ```

2. **Install dependencies**
   ```bash
   cargo build
   ```

3. **Set up database**
   ```bash
   # Set your database URL
   export DATABASE_URL="postgresql://username:password@localhost/ant_colony_db"
   
   # Run database migrations (if any)
   # sqlx migrate run
   ```

4. **Run the simulation**
   ```bash
   # Run with default settings
   cargo run
   
   # Run with specific simulation ID
   cargo run -- --simulation-id 1
   
   # Run with custom database URL
   cargo run -- --database-url "postgresql://user:pass@localhost/db"
   
   # Run with debug logging
   cargo run -- --log-level debug
   ```

## Configuration

### Command Line Arguments

- `--simulation-id`: ID of the simulation to run
- `--database-url`: PostgreSQL connection string
- `--log-level`: Logging level (trace, debug, info, warn, error)
- `--server-addr`: WebSocket server address (default: 127.0.0.1:8080)

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string
- `RUST_LOG`: Logging level configuration

## WebSocket API

The simulation broadcasts real-time updates via WebSocket:

### Connection
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
```

### Subscribe to Simulation
```javascript
ws.send(JSON.stringify({
    type: 'Subscribe',
    simulation_id: 1
}));
```

### Message Types

#### FullState
Complete simulation state (sent on initial connection):
```json
{
    "type": "FullState",
    "simulation_id": 1,
    "tick": 1234,
    "ants": [...],
    "colonies": [...],
    "food_sources": [...],
    "pheromone_trails": [...]
}
```

#### DeltaUpdate
Incremental updates with only changed entities:
```json
{
    "type": "DeltaUpdate",
    "simulation_id": 1,
    "tick": 1234,
    "updated_ants": [...],
    "updated_colonies": [...],
    "updated_food_sources": [...],
    "new_pheromone_trails": [...],
    "removed_ant_ids": [...],
    "removed_food_source_ids": [...]
}
```

## Performance

The Bevy-based simulation offers significant performance improvements:

- **ECS Architecture**: Efficient entity processing
- **Parallel Systems**: Concurrent execution of independent systems
- **Memory Management**: Optimized component storage
- **Big Brain AI**: Fast behavior tree evaluation

## Development

### Adding New Systems

1. Create a new system function:
   ```rust
   pub fn my_system(
       mut query: Query<(&mut MyComponent, &OtherComponent)>,
   ) {
       for (mut my_comp, other_comp) in query.iter_mut() {
           // System logic
       }
   }
   ```

2. Add it to a plugin:
   ```rust
   impl Plugin for MyPlugin {
       fn build(&self, app: &mut App) {
           app.add_systems(Update, my_system);
       }
   }
   ```

3. Register the plugin in the main simulation:
   ```rust
   app.add_plugins(MyPlugin);
   ```

### Adding New Components

1. Define the component:
   ```rust
   #[derive(Component, Debug, Clone)]
   pub struct MyComponent {
       pub value: f32,
   }
   ```

2. Add it to entities:
   ```rust
   commands.spawn((
       MyComponent { value: 42.0 },
       // Other components...
   ));
   ```

### Adding New Big Brain Actions

1. Implement the Action trait:
   ```rust
   #[derive(Clone, Component, Debug)]
   pub struct MyAction;

   impl Action for MyAction {
       fn is_valid(&self, _actor: &Actor) -> bool {
           true
       }

       fn execute(&self, actor: &Actor, ctx: &Context) -> ActionResult {
           // Action logic
           ActionResult::Success
       }
   }
   ```

2. Add it to the behavior tree:
   ```rust
   Thinker::new()
       .when("My Action", MyScorer, MyAction)
   ```

## Monitoring

### Health Check
```bash
curl http://localhost:8080/health
```

### Server Stats
```bash
curl http://localhost:8080/stats
```

### Logging
The simulation uses structured logging with different levels:
- `TRACE`: Detailed system execution
- `DEBUG`: System state and performance
- `INFO`: General simulation events
- `WARN`: Potential issues
- `ERROR`: Errors and failures

## Troubleshooting

### Common Issues

1. **Database Connection Failed**
   - Check `DATABASE_URL` environment variable
   - Ensure PostgreSQL is running
   - Verify database exists and is accessible

2. **WebSocket Connection Failed**
   - Check if port 8080 is available
   - Verify firewall settings
   - Check server logs for errors

3. **Simulation Performance Issues**
   - Reduce entity count
   - Adjust system update frequency
   - Monitor memory usage

### Debug Mode

Run with debug logging for detailed information:
```bash
cargo run -- --log-level debug
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request
