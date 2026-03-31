import type { InferInsertModel, InferSelectModel } from "drizzle-orm";
import * as schema from "./generated/schema";

export { schema };

export { eq, and, or, like, not, desc, asc } from "drizzle-orm";

// App tables
export type Simulation = InferSelectModel<typeof schema.simulations>;
export type NewSimulation = InferInsertModel<typeof schema.simulations>;
export type Colony = InferSelectModel<typeof schema.colonies>;
export type NewColony = InferInsertModel<typeof schema.colonies>;
export type AntType = InferSelectModel<typeof schema.ant_types>;
export type SimulationCheckpoint = InferSelectModel<typeof schema.simulation_checkpoints>;
export type SimulationStats = InferSelectModel<typeof schema.simulation_stats>;

// Auth tables
export type User = InferSelectModel<typeof schema.user>;
export type NewUser = InferInsertModel<typeof schema.user>;
export type Session = InferSelectModel<typeof schema.session>;
export type NewSession = InferInsertModel<typeof schema.session>;
export type Account = InferSelectModel<typeof schema.account>;
export type NewAccount = InferInsertModel<typeof schema.account>;
export type Verification = InferSelectModel<typeof schema.verification>;
export type Passkey = InferSelectModel<typeof schema.passkey>;
