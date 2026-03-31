import { pgTable, serial, text, integer, bigint, boolean, timestamp, jsonb, index, foreignKey, real } from "drizzle-orm/pg-core"
import { sql } from "drizzle-orm"

// ── App tables ───────────────────────────────────────────────────────

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
	state_blob: text().notNull(),
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

// ── Better Auth tables ───────────────────────────────────────────────

export const user = pgTable("user", {
	id: text().primaryKey().notNull(),
	name: text().notNull(),
	email: text().notNull().unique(),
	emailVerified: boolean().notNull().default(false),
	image: text(),
	createdAt: timestamp().notNull().defaultNow(),
	updatedAt: timestamp().notNull().defaultNow(),
});

export const session = pgTable("session", {
	id: text().primaryKey().notNull(),
	expiresAt: timestamp().notNull(),
	token: text().notNull().unique(),
	createdAt: timestamp().notNull().defaultNow(),
	updatedAt: timestamp().notNull().defaultNow(),
	ipAddress: text(),
	userAgent: text(),
	userId: text().notNull(),
}, (table) => ({
	idx_session_userId: index("idx_session_userId").on(table.userId),
	session_userId_fkey: foreignKey({
		columns: [table.userId],
		foreignColumns: [user.id],
		name: "session_userId_fkey"
	}).onDelete("cascade"),
}));

export const account = pgTable("account", {
	id: text().primaryKey().notNull(),
	accountId: text().notNull(),
	providerId: text().notNull(),
	userId: text().notNull(),
	accessToken: text(),
	refreshToken: text(),
	idToken: text(),
	accessTokenExpiresAt: timestamp(),
	refreshTokenExpiresAt: timestamp(),
	scope: text(),
	password: text(),
	createdAt: timestamp().notNull().defaultNow(),
	updatedAt: timestamp().notNull().defaultNow(),
}, (table) => ({
	idx_account_userId: index("idx_account_userId").on(table.userId),
	account_userId_fkey: foreignKey({
		columns: [table.userId],
		foreignColumns: [user.id],
		name: "account_userId_fkey"
	}).onDelete("cascade"),
}));

export const verification = pgTable("verification", {
	id: text().primaryKey().notNull(),
	identifier: text().notNull(),
	value: text().notNull(),
	expiresAt: timestamp().notNull(),
	createdAt: timestamp().notNull().defaultNow(),
	updatedAt: timestamp().notNull().defaultNow(),
}, (table) => ({
	idx_verification_identifier: index("idx_verification_identifier").on(table.identifier),
}));

export const passkey = pgTable("passkey", {
	id: text().primaryKey().notNull(),
	name: text(),
	publicKey: text().notNull(),
	userId: text().notNull(),
	credentialID: text().notNull(),
	counter: integer().notNull(),
	deviceType: text().notNull(),
	backedUp: boolean().notNull(),
	transports: text(),
	createdAt: timestamp(),
	aaguid: text(),
}, (table) => ({
	idx_passkey_userId: index("idx_passkey_userId").on(table.userId),
	idx_passkey_credentialID: index("idx_passkey_credentialID").on(table.credentialID),
	passkey_userId_fkey: foreignKey({
		columns: [table.userId],
		foreignColumns: [user.id],
		name: "passkey_userId_fkey"
	}).onDelete("cascade"),
}));
