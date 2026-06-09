import type { InferInsertModel, InferSelectModel } from "drizzle-orm";
import * as schema from "./generated/schema";

export { schema };

export { eq, and, or, like, not, desc, asc } from "drizzle-orm";

export type Simulation = InferSelectModel<typeof schema.simulations>;
export type NewSimulation = InferInsertModel<typeof schema.simulations>;
export type Colony = InferSelectModel<typeof schema.colonies>;
export type NewColony = InferInsertModel<typeof schema.colonies>;
export type AntType = InferSelectModel<typeof schema.ant_types>;
export type SimulationCheckpoint = InferSelectModel<typeof schema.simulation_checkpoints>;
export type SimulationStats = InferSelectModel<typeof schema.simulation_stats>;
