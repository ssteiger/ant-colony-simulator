import { pgTable, serial, text, integer, bigint, boolean, timestamp, jsonb, index, foreignKey, real, customType } from "drizzle-orm/pg-core"
import { sql } from "drizzle-orm"

const bytea = customType<{ data: Uint8Array; driverData: Uint8Array }>({
	dataType() {
		return "bytea";
	},
});

export const simulations = pgTable("simulations", {
	id: serial().primaryKey().notNull(),
	name: text().notNull(),
	world_width: integer().default(1200).notNull(),
	world_height: integer().default(800).notNull(),
	config: jsonb().default({}).notNull(),
	is_active: boolean().default(true),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`now()`),
	updated_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`now()`),
});

export const colonies = pgTable("colonies", {
	id: serial().primaryKey().notNull(),
	simulation_id: integer().notNull(),
	name: text().notNull(),
	center_x: integer().notNull(),
	center_y: integer().notNull(),
	color_hue: integer().default(30).notNull(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`now()`),
}, (table) => ({
	colonies_simulation_id_fkey: foreignKey({
		columns: [table.simulation_id],
		foreignColumns: [simulations.id],
		name: "colonies_simulation_id_fkey"
	}).onDelete("cascade"),
}));

export const ant_types = pgTable("ant_types", {
	id: serial().primaryKey().notNull(),
	name: text().notNull().unique(),
	role: text().notNull(),
	base_speed: real().default(2.0).notNull(),
	base_health: real().default(100.0).notNull(),
	color_hue: integer().default(30).notNull(),
	attributes: jsonb().default({}).notNull(),
});

export const simulation_checkpoints = pgTable("simulation_checkpoints", {
	id: serial().primaryKey().notNull(),
	simulation_id: integer().notNull(),
	tick: bigint({ mode: "number" }).notNull(),
	state_blob: bytea().notNull(),
	summary: jsonb().default({}).notNull(),
	created_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`now()`),
}, (table) => ({
	idx_checkpoints_sim_tick: index("idx_checkpoints_sim_tick").on(table.simulation_id, table.tick),
	checkpoints_simulation_id_fkey: foreignKey({
		columns: [table.simulation_id],
		foreignColumns: [simulations.id],
		name: "simulation_checkpoints_simulation_id_fkey"
	}).onDelete("cascade"),
}));

export const simulation_stats = pgTable("simulation_stats", {
	id: serial().primaryKey().notNull(),
	simulation_id: integer().notNull(),
	tick: bigint({ mode: "number" }).notNull(),
	total_ants: integer().notNull(),
	food_collected: real().default(0).notNull(),
	colony_stats: jsonb().default({}).notNull(),
	recorded_at: timestamp({ withTimezone: true, mode: 'string' }).default(sql`now()`),
}, (table) => ({
	idx_stats_sim_tick: index("idx_stats_sim_tick").on(table.simulation_id, table.tick),
	stats_simulation_id_fkey: foreignKey({
		columns: [table.simulation_id],
		foreignColumns: [simulations.id],
		name: "simulation_stats_simulation_id_fkey"
	}).onDelete("cascade"),
}));
