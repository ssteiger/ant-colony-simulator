import { relations } from "drizzle-orm/relations";
import {
	simulations, colonies, simulation_checkpoints, simulation_stats,
} from "./schema";

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
