import { relations } from "drizzle-orm/relations";
import {
	simulations, colonies, simulation_checkpoints, simulation_stats,
	user, session, account, passkey,
} from "./schema";

// ── App relations ────────────────────────────────────────────────────

export const simulationsRelations = relations(simulations, ({ many }) => ({
	colonies: many(colonies),
	checkpoints: many(simulation_checkpoints),
	stats: many(simulation_stats),
}));

export const coloniesRelations = relations(colonies, ({ one }) => ({
	simulation: one(simulations, {
		fields: [colonies.simulation_id],
		references: [simulations.id],
	}),
}));

export const simulationCheckpointsRelations = relations(simulation_checkpoints, ({ one }) => ({
	simulation: one(simulations, {
		fields: [simulation_checkpoints.simulation_id],
		references: [simulations.id],
	}),
}));

export const simulationStatsRelations = relations(simulation_stats, ({ one }) => ({
	simulation: one(simulations, {
		fields: [simulation_stats.simulation_id],
		references: [simulations.id],
	}),
}));

// ── Auth relations ───────────────────────────────────────────────────

export const userRelations = relations(user, ({ many }) => ({
	sessions: many(session),
	accounts: many(account),
	passkeys: many(passkey),
}));

export const sessionRelations = relations(session, ({ one }) => ({
	user: one(user, {
		fields: [session.userId],
		references: [user.id],
	}),
}));

export const accountRelations = relations(account, ({ one }) => ({
	user: one(user, {
		fields: [account.userId],
		references: [user.id],
	}),
}));

export const passkeyRelations = relations(passkey, ({ one }) => ({
	user: one(user, {
		fields: [passkey.userId],
		references: [user.id],
	}),
}));
