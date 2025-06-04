import { default as crypto } from "$lib/crypto";
import { unreachable } from "$lib/shared/unreachable";
import { AuthClient } from "@dfinity/auth-client";
import { derived, get, writable } from "svelte/store";
import { createActor as createActorOrchestrator } from "../../../declarations/orchestrator";
import { createActor as createActorUser } from "../../../declarations/user_canister";
import type {
  ActorTypeOrchestrator,
  ActorTypeUserCanister,
} from "../shared/actor";
import { FilesService } from "./files";
import { RequestsService } from "./requests";
import { UploadService } from "./upload";
import { UserService } from "./user";

type AuthStateUninitialized = {
  state: "uninitialized";
};

export type AuthStateAuthenticated = {
  state: "authenticated";
  actor_user?: ActorTypeUserCanister;
  actor_orchestrator: ActorTypeOrchestrator;
  authClient: AuthClient;
  userService?: UserService;
  filesService?: FilesService;
  requestService?: RequestsService;
  uploadService?: UploadService;
  userCanisterId?: string;
  canisterRetrievalState: "retrieved" | "pending" | "failed" | "uninitialized";
};

export type AuthStateUnauthenticated = {
  state: "unauthenticated";
  authClient: AuthClient;
  actor_orchestrator: ActorTypeOrchestrator;
};

export type AuthState =
  | AuthStateUninitialized
  | AuthStateAuthenticated
  | AuthStateUnauthenticated;

function createAuthStore() {
  const { subscribe, set } = writable<AuthState>({
    state: "uninitialized",
  });

  return {
    subscribe,
    set,
    setLoggedin: (
      actor_orchestrator: ActorTypeOrchestrator,
      authClient: AuthClient,
      actor_user?: ActorTypeUserCanister,
      userService?: UserService,
      filesService?: FilesService,
      requestService?: RequestsService,
      uploadService?: UploadService,
      userCanisterId?: string,
      canisterRetrievalState:
        | "retrieved"
        | "pending"
        | "failed"
        | "uninitialized" = "uninitialized"
    ) => {
      set({
        state: "authenticated",
        actor_user,
        actor_orchestrator,
        authClient,
        userService,
        filesService,
        requestService,
        uploadService,
        userCanisterId,
        canisterRetrievalState,
      });
    },
    setLoggedout: (
      actor_orchestrator: ActorTypeOrchestrator,
      authClient: AuthClient
    ) => {
      set({
        state: "unauthenticated",
        actor_orchestrator,
        authClient,
      });
    },
    getAuthClient: () => {
      const store = get(authStore);
      if (store.state === "uninitialized") {
        throw new Error(
          "AuthClient is not available. User must be authenticated."
        );
      } else if (
        store.state === "unauthenticated" ||
        store.state === "authenticated"
      ) {
        return store.authClient;
      }
      unreachable(store);
    },
  };
}

export const authStore = createAuthStore();
export const isAuthenticated = derived(
  authStore,
  (store) => store.state === "authenticated"
);

function createServices(
  actorUser: ActorTypeUserCanister,
  actorOrchestrator: ActorTypeOrchestrator
) {
  const userService = new UserService(actorOrchestrator, actorUser);
  userService.init();
  const filesService = new FilesService(actorUser, actorOrchestrator);
  const requestsService = new RequestsService(actorUser);
  const uploadService = new UploadService(actorUser);

  return {
    userService,
    filesService,
    requestsService,
    uploadService,
  };
}

async function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export class AuthService {
  constructor(
    private canisterIdOrchestrator: string,
    private host: string,
    private iiUrl: string
  ) {}

  initClient(canisterId: string) {
    const store = get(authStore);
    const identity =
      store.state === "authenticated" || store.state === "unauthenticated"
        ? store.authClient.getIdentity()
        : undefined; /// should not  be reached. type guard
    if (!identity) {
      throw new Error("Identity is not available. User must be authenticated.");
    }
    console.log("Creating actor client for:", canisterId);
    const actor_user = createActorUser(canisterId, {
      agentOptions: {
        host: this.host,
        identity,
      },
    });
    const uploadService = new UploadService(actor_user); /// Maybe not here..
    return { actor_user, uploadService };
  }

  async tryRetrieveUserCanister(
    actor_orchestrator: ActorTypeOrchestrator,
    authClient: AuthClient,
    maxRetries: number = 5,
    retryDelayMs: number = 2000
  ): Promise<AuthStateAuthenticated> {
    // console.log("Trying to retrieve user canister...");
    let retries = 0;
    while (retries < maxRetries) {
      try {
        const response = await actor_orchestrator.user_canister();
        if ("Ok" in response) {
          const userCanisterId = response.Ok.toText();
          // console.log("User canister ID retrieved:", userCanisterId);
          const actor_user = createActorUser(userCanisterId, {
            agentOptions: {
              host: this.host,
              identity: authClient.getIdentity(),
            },
          });

          try {
            const currentPublicKey = await actor_user.public_key();
            const localPublicKey = new Uint8Array(
              await crypto.getLocalUserPublicKey()
            );
            if (!currentPublicKey || currentPublicKey.length === 0) {
              console.log("Setting public key for user canister");
              await actor_user.set_public_key(localPublicKey);
            } else {
              // console.log("Public key already set for user canister");
            }
          } catch (error) {
            console.error("Failed to get or set public key:", error);
          }
          const { userService, filesService, requestsService, uploadService } =
            createServices(actor_user, actor_orchestrator);
          return {
            state: "authenticated",
            actor_user,
            actor_orchestrator,
            authClient,
            userService,
            filesService,
            requestService: requestsService,
            uploadService,
            userCanisterId,
            canisterRetrievalState: "retrieved",
          };
        } else if ("CreationPending" in response) {
          retries++;
          await delay(retryDelayMs);
          continue;
        } else if ("Uninitialized" in response) {
          // For "Uninitialized", create UserService without actor_user
          const userService = new UserService(actor_orchestrator);
          userService.init(); // Initialize UserService to check user status
          return {
            state: "authenticated",
            actor_orchestrator,
            authClient,
            userService,
            canisterRetrievalState: "uninitialized",
          };
        } else if ("CreationFailed" in response) {
          console.warn("User canister creation failed, retrying...");

          const retryResponse =
            await actor_orchestrator.retry_user_canister_creation();
          if ("Created" in retryResponse) {
            const userCanisterId = retryResponse.Created.toText();
            const actor_user = createActorUser(userCanisterId, {
              agentOptions: {
                host: this.host,
                identity: authClient.getIdentity(),
              },
            });
            try {
              const currentPublicKey = await actor_user.public_key();
              const localPublicKey = new Uint8Array(
                await crypto.getLocalUserPublicKey()
              );
              if (!currentPublicKey || currentPublicKey.length === 0) {
                console.log("Setting public key for user canister (retry)");
                await actor_user.set_public_key(localPublicKey);
              } else {
                console.log("Public key already set for user canister (retry)");
              }
            } catch (error) {
              console.error("Failed to get or set public key:", error);
            }
            const {
              userService,
              filesService,
              requestsService,
              uploadService,
            } = createServices(actor_user, actor_orchestrator);
            return {
              state: "authenticated",
              actor_user,
              actor_orchestrator,
              authClient,
              userService,
              filesService,
              requestService: requestsService,
              uploadService,
              userCanisterId,
              canisterRetrievalState: "retrieved",
            };
          } else if (
            "Ok" in retryResponse ||
            "CreationPending" in retryResponse
          ) {
            retries++;
            await delay(retryDelayMs);
            continue;
          } else if ("AnonymousCaller" in retryResponse) {
            authStore.setLoggedout(actor_orchestrator, authClient);
            return {
              state: "unauthenticated",
              actor_orchestrator,
              authClient,
            } as any; // Type cast to satisfy return type
          } else if ("UserNotFound" in retryResponse) {
            // createServices(null, actor_orchestrator); // Ensure services are created
            return {
              state: "authenticated",
              actor_orchestrator,
              authClient,
              canisterRetrievalState: "uninitialized",
            };
          } else {
            unreachable(retryResponse);
          }
        } else if ("AnonymousCaller" in response) {
          authStore.setLoggedout(actor_orchestrator, authClient);
          return {
            state: "unauthenticated",
            actor_orchestrator,
            authClient,
          } as any; // Type cast to satisfy return type
        } else {
          unreachable(response);
        }
      } catch (e) {
        console.error("Error", e);
        retries++;
        await delay(retryDelayMs);
      }
    }
    return {
      state: "authenticated",
      actor_orchestrator,
      authClient,
      canisterRetrievalState: "failed",
    };
  }

  async init() {
    const authClient = await AuthClient.create();

    const actor_orchestrator = createActorOrchestrator(
      this.canisterIdOrchestrator,
      {
        agentOptions: { host: this.host, identity: authClient.getIdentity() },
      }
    );

    if (await authClient.isAuthenticated()) {
      // console.log(
      //   "User is authenticated",
      //   authClient.getIdentity().getPrincipal().toText()
      // );
      const authState = await this.tryRetrieveUserCanister(
        actor_orchestrator,
        authClient
      );
      // console.log(authState);
      authStore.set(authState);
    } else {
      authStore.setLoggedout(actor_orchestrator, authClient);
    }
  }

  async login() {
    const store = get(authStore);

    if (store.state === "authenticated") {
      return;
    } else if (store.state === "uninitialized") {
      return;
    } else if (store.state === "unauthenticated") {
      try {
        await new Promise<void>((resolve, reject) => {
          store.authClient.login({
            identityProvider: this.iiUrl,
            onSuccess: resolve,
            onError: reject,
          });
        });

        const actor_orchestrator = createActorOrchestrator(
          this.canisterIdOrchestrator,
          {
            agentOptions: {
              host: this.host,
              identity: store.authClient.getIdentity(),
            },
          }
        );

        const authState = await this.tryRetrieveUserCanister(
          actor_orchestrator,
          store.authClient
        );
        authStore.set(authState);
      } catch (e) {
        console.error("Login failed", e);
        const actor_orchestrator = createActorOrchestrator(
          this.canisterIdOrchestrator,
          {
            agentOptions: {
              host: this.host,
              identity: store.authClient.getIdentity(),
            },
          }
        );
        authStore.setLoggedout(actor_orchestrator, store.authClient);
      }
    } else {
      unreachable(store);
    }
  }

  async logout() {
    const store = get(authStore);
    if (store.state === "authenticated") {
      try {
        await store.authClient.logout();
        if (store.userService) {
          store.userService.reset();
        }

        const actor_orchestrator = createActorOrchestrator(
          this.canisterIdOrchestrator,
          {
            agentOptions: {
              host: this.host,
              identity: store.authClient.getIdentity(),
            },
          }
        );

        authStore.setLoggedout(actor_orchestrator, store.authClient);
      } catch (e) {}
    } else if (store.state === "uninitialized") {
      return;
    } else if (store.state === "unauthenticated") {
      return;
    } else {
      unreachable(store);
    }
  }
}

export const authService = new AuthService(
  import.meta.env.VITE_ORCHESTRATOR_CANISTER_ID,
  import.meta.env.VITE_HOST,
  import.meta.env.VITE_II_URL
);
