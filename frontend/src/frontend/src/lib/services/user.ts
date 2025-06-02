import { default as crypto } from "$lib/crypto";
import type {
  ActorTypeOrchestrator,
  ActorTypeUserCanister,
} from "$lib/shared/actor";
import { enumIs } from "$lib/shared/enums";
import { unreachable } from "$lib/shared/unreachable";
import { writable } from "svelte/store";

type UserState =
  | {
      state: "uninitialized";
    }
  | {
      state: "error";
      error: string;
    }
  | {
      state: "registered";
      username: string;
      userCanisterId?: string; // Optional until retrieved
    }
  | {
      state: "unregistered";
      registrationState:
        | { state: "idle" }
        | { state: "registering" }
        | {
            state: "error";
            errorMessage: string;
          };
    };

type RegistrationState = Extract<
  UserState,
  { state: "unregistered" }
>["registrationState"];

function createUserStore() {
  const { subscribe, set } = writable<UserState>({
    state: "uninitialized",
  });

  return {
    subscribe,
    set,

    register: (username: string, userCanisterId?: string) => {
      set({
        state: "registered",
        username,
        userCanisterId,
      });
    },
    setUnregistered: (registrationState: RegistrationState) => {
      set({
        state: "unregistered",
        registrationState,
      });
    },
    setError: (error: string) => {
      console.log("User store error", error);

      set({
        state: "error",
        error,
      });
    },
    reset: () => set({ state: "uninitialized" }),
  };
}

export const userStore = createUserStore();

export class UserService {
  constructor(
    private actorOrchestrator: ActorTypeOrchestrator,
    private actorUser?: ActorTypeUserCanister // Optional, as it may not be available immediately
  ) {}

  async init() {
    try {
      const response = await this.actorOrchestrator.who_am_i();
      if (enumIs(response, "known_user")) {
        const orchestratorResponse =
          await this.actorOrchestrator.user_canister();
        if ("Ok" in orchestratorResponse) {
          userStore.register(
            response.known_user.username,
            orchestratorResponse.Ok.toText()
          );
        } else {
          userStore.register(response.known_user.username); // Register without canister ID
        }
      } else if (enumIs(response, "unknown_user")) {
        userStore.setUnregistered({ state: "idle" });
      } else {
        unreachable(response);
      }
    } catch (e) {
      userStore.setError("Could not get user info");
    }
  }

  async register(username: string) {
    try {
      userStore.setUnregistered({ state: "registering" });

      const response = await this.actorOrchestrator.set_user(
        username,
        new Uint8Array(await crypto.getLocalUserPublicKey())
      );

      if (enumIs(response, "username_exists")) {
        userStore.setUnregistered({
          state: "error",
          errorMessage: "Username already exists",
        });
        return;
      }
      //////
      // Retrieve user canister ID after registration
      let retries = 0;
      const maxRetries = 20; // Maximum number of retries
      const retryDelayMs = 2000;
      while (retries < maxRetries) {
        const orchestratorResponse =
          await this.actorOrchestrator.user_canister();
        console.log("Orchestrator response:", orchestratorResponse);
        if ("Ok" in orchestratorResponse) {
          const userCanisterId = orchestratorResponse.Ok.toText();
          userStore.register(username, userCanisterId);
          // // Update authStore with new canister and services
          // const authClient = get(authStore).authClient;
          // const authService = new AuthService(
          //   import.meta.env.VITE_ORCHESTRATOR_CANISTER_ID,
          //   import.meta.env.VITE_HOST,
          //   import.meta.env.VITE_II_URL
          // );
          // const authState = await authService.tryRetrieveUserCanister(
          //   this.actorOrchestrator,
          //   authClient
          // );
          // authStore.set(authState);
          return;
        } else if ("CreationPending" in orchestratorResponse) {
          retries++;
          await new Promise((resolve) => setTimeout(resolve, retryDelayMs));
          continue;
        } else if (
          "CreationFailed" in orchestratorResponse ||
          "Uninitialized" in orchestratorResponse
        ) {
          const retryResponse =
            await this.actorOrchestrator.retry_user_canister_creation();
          if ("Created" in retryResponse) {
            const userCanisterId = retryResponse.Created.toText();
            userStore.register(username, userCanisterId);
            // const authClient = get(authStore).authClient;
            // const authService = new AuthService(
            //   import.meta.env.VITE_ORCHESTRATOR_CANISTER_ID,
            //   import.meta.env.VITE_HOST,
            //   import.meta.env.VITE_II_URL
            // );
            // const authState = await authService.tryRetrieveUserCanister(
            //   this.actorOrchestrator,
            //   authClient
            // );
            // authStore.set(authState);
            return;
          } else if (
            "Ok" in retryResponse ||
            "CreationPending" in retryResponse
          ) {
            retries++;
            await new Promise((resolve) => setTimeout(resolve, retryDelayMs));
            continue;
          } else if ("AnonymousCaller" in retryResponse) {
            userStore.setError("Anonymous caller detected during retry");
            return;
          } else if ("UserNotFound" in retryResponse) {
            userStore.setUnregistered({
              state: "error",
              errorMessage: "User not found; please try registering again",
            });
            return;
          } else {
            unreachable(retryResponse);
          }
        } else if ("AnonymousCaller" in orchestratorResponse) {
          userStore.setError("Anonymous caller detected");
          return;
        } else {
          unreachable(orchestratorResponse);
        }
      }

      userStore.setUnregistered({
        state: "error",
        errorMessage: "Failed to retrieve user canister ID after retries",
      });
    } catch (e: unknown) {
      if (e instanceof Error) {
        userStore.setUnregistered({
          state: "error",
          errorMessage: e.toString(),
        });
      } else {
        userStore.setUnregistered({
          state: "error",
          errorMessage: "Unknown error",
        });
      }
    }
  }

  async reset() {
    userStore.reset();
  }
}
