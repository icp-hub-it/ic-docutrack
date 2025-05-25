import type { createActor as createOrchestrator } from "../../../../../declarations/orchestrator";
import type { createActor as createUserCanister } from "../../../../../declarations/user_canister";

export type ActorTypeOrchestrator = ReturnType<typeof createOrchestrator>;
export type ActorTypeUserCanister = ReturnType<typeof createUserCanister>;
