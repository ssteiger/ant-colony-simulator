-- Ant Colony Simulator -- Database Schema
-- Real-time state lives in Rust memory; Postgres stores config + checkpoints + stats

CREATE TABLE simulations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    world_width INT NOT NULL DEFAULT 1200,
    world_height INT NOT NULL DEFAULT 800,
    config JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE colonies (
    id SERIAL PRIMARY KEY,
    simulation_id INT NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    center_x INT NOT NULL,
    center_y INT NOT NULL,
    color_hue INT NOT NULL DEFAULT 30,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE ant_types (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    role VARCHAR(30) NOT NULL,
    base_speed REAL NOT NULL DEFAULT 2.0,
    base_health REAL NOT NULL DEFAULT 100.0,
    color_hue INT NOT NULL DEFAULT 30,
    attributes JSONB NOT NULL DEFAULT '{}'
);

-- Binary blob of the full Rust simulation state for pause/resume
CREATE TABLE simulation_checkpoints (
    id SERIAL PRIMARY KEY,
    simulation_id INT NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    tick BIGINT NOT NULL,
    state_blob BYTEA NOT NULL,
    summary JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_checkpoints_sim_tick
    ON simulation_checkpoints(simulation_id, tick DESC);

-- Time-series stats for dashboard charts
CREATE TABLE simulation_stats (
    id SERIAL PRIMARY KEY,
    simulation_id INT NOT NULL REFERENCES simulations(id) ON DELETE CASCADE,
    tick BIGINT NOT NULL,
    total_ants INT NOT NULL,
    food_collected REAL NOT NULL DEFAULT 0,
    colony_stats JSONB NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_stats_sim_tick
    ON simulation_stats(simulation_id, tick);

-- Seed data
INSERT INTO ant_types (name, role, base_speed, base_health, color_hue, attributes) VALUES
    ('Worker',  'worker',  2.0, 80.0,  30,  '{"vision_range": 40, "carrying_capacity": 2}'),
    ('Scout',   'scout',   4.0, 60.0,  60,  '{"vision_range": 80, "pheromone_sensitivity": 1.3}'),
    ('Soldier', 'soldier', 1.5, 150.0,  0,  '{"vision_range": 45, "combat_bonus": 2.0}');

-- ── Better Auth tables ───────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS "user" (
    "id" text PRIMARY KEY NOT NULL,
    "name" text NOT NULL,
    "email" text NOT NULL UNIQUE,
    "emailVerified" boolean NOT NULL DEFAULT false,
    "image" text,
    "createdAt" timestamp NOT NULL DEFAULT now(),
    "updatedAt" timestamp NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS "session" (
    "id" text PRIMARY KEY NOT NULL,
    "expiresAt" timestamp NOT NULL,
    "token" text NOT NULL UNIQUE,
    "createdAt" timestamp NOT NULL DEFAULT now(),
    "updatedAt" timestamp NOT NULL DEFAULT now(),
    "ipAddress" text,
    "userAgent" text,
    "userId" text NOT NULL REFERENCES "user"("id") ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS "idx_session_userId" ON "session" ("userId");

CREATE TABLE IF NOT EXISTS "account" (
    "id" text PRIMARY KEY NOT NULL,
    "accountId" text NOT NULL,
    "providerId" text NOT NULL,
    "userId" text NOT NULL REFERENCES "user"("id") ON DELETE CASCADE,
    "accessToken" text,
    "refreshToken" text,
    "idToken" text,
    "accessTokenExpiresAt" timestamp,
    "refreshTokenExpiresAt" timestamp,
    "scope" text,
    "password" text,
    "createdAt" timestamp NOT NULL DEFAULT now(),
    "updatedAt" timestamp NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS "idx_account_userId" ON "account" ("userId");

CREATE TABLE IF NOT EXISTS "verification" (
    "id" text PRIMARY KEY NOT NULL,
    "identifier" text NOT NULL,
    "value" text NOT NULL,
    "expiresAt" timestamp NOT NULL,
    "createdAt" timestamp NOT NULL DEFAULT now(),
    "updatedAt" timestamp NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS "idx_verification_identifier" ON "verification" ("identifier");

CREATE TABLE IF NOT EXISTS "passkey" (
    "id" text PRIMARY KEY NOT NULL,
    "name" text,
    "publicKey" text NOT NULL,
    "userId" text NOT NULL REFERENCES "user"("id") ON DELETE CASCADE,
    "credentialID" text NOT NULL,
    "counter" integer NOT NULL,
    "deviceType" text NOT NULL,
    "backedUp" boolean NOT NULL,
    "transports" text,
    "createdAt" timestamp,
    "aaguid" text
);

CREATE INDEX IF NOT EXISTS "idx_passkey_userId" ON "passkey" ("userId");
CREATE INDEX IF NOT EXISTS "idx_passkey_credentialID" ON "passkey" ("credentialID");
