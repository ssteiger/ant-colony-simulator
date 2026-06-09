# Rust Backend

Runs the ant colony simulation and streams state to the browser.

- 60 Hz simulation loop (ants, pheromone grids, food sources, terrain with collision)
- Binary WebSocket protocol on `ws://127.0.0.1:8080/ws`
- Periodic checkpoints (bincode blobs) written to Postgres (`simulation_checkpoints`)

## Run

```bash
# requires a running Postgres (see apps/supabase)
sh ./run.sh        # release build + run
sh ./run.sh dev    # debug build
```

## Environment

| Variable       | Default                                                      |
| -------------- | ------------------------------------------------------------ |
| `DATABASE_URL` | `postgresql://postgres:postgres@127.0.0.1:57322/postgres`     |
| `RUST_LOG`     | `info`                                                        |

## Source layout

- `src/simulation/` — sim core: ants (SoA), pheromone grids, terrain generation, steering
- `src/server/` — axum WebSocket server + binary protocol encoding
- `src/db/` — sqlx checkpoint persistence
