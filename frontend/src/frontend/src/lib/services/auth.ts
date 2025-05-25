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
  actor_user: ActorTypeUserCanister;
  actor_orchestrator: ActorTypeOrchestrator;
  authClient: AuthClient;
  userService: UserService;
  filesService: FilesService;
  requestService: RequestsService;
  uploadService: UploadService;
};

export type AuthStateUnauthenticated = {
  state: "unauthenticated";
  authClient: AuthClient;
  actor_user: ActorTypeUserCanister;
  actor_orchestrator: ActorTypeOrchestrator;
  uploadService: UploadService;
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
      actor_user: ActorTypeUserCanister,
      actor_orchestrator: ActorTypeOrchestrator,
      authClient: AuthClient,
      userService: UserService,
      filesService: FilesService,
      requestService: RequestsService,
      uploadService: UploadService
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
      });
    },
    setLoggedout: (
      actor_user: ActorTypeUserCanister,
      actor_orchestrator: ActorTypeOrchestrator,
      authClient: AuthClient,
      uploadService: UploadService
    ) => {
      set({
        state: "unauthenticated",
        actor_user,
        actor_orchestrator,
        authClient,
        uploadService,
      });
    },
  };
}

export const authStore = createAuthStore();
export const isAuthenticated = derived(
  authStore,
  (store) => store.state === "authenticated"
);

function createServices(
  actor_user: ActorTypeUserCanister,
  actorOrchestrator: ActorTypeOrchestrator
) {
  const userService = new UserService(actorOrchestrator);
  userService.init();
  const filesService = new FilesService(actor_user, actorOrchestrator);
  const requestsService = new RequestsService(actor_user);
  const uploadService = new UploadService(actor_user);

  return {
    userService,
    filesService,
    requestsService,
    uploadService,
  };
}

export class AuthService {
  constructor(
    private canisterIdUser: string,
    private canisterIdOrchestrator: string,
    private host: string,
    private iiUrl: string
  ) {}

  async init() {
    const authClient = await AuthClient.create();
    if (await authClient.isAuthenticated()) {
      const actor_user = createActorUser(this.canisterIdUser, {
        agentOptions: { host: this.host, identity: authClient.getIdentity() },
      });
      const actor_orchestrator = createActorOrchestrator(
        this.canisterIdOrchestrator,
        {
          agentOptions: { host: this.host, identity: authClient.getIdentity() },
        }
      );

      const { userService, filesService, requestsService, uploadService } =
        createServices(actor_user, actor_orchestrator);

      authStore.setLoggedin(
        actor_user,
        actor_orchestrator,
        authClient,
        userService,
        filesService,
        requestsService,
        uploadService
      );
    } else {
      const actor_user = createActorUser(this.canisterIdUser, {
        agentOptions: {
          host: this.host,
          identity: authClient.getIdentity(),
        },
      });
      const actor_orchestrator = createActorOrchestrator(
        this.canisterIdOrchestrator,
        {
          agentOptions: {
            host: this.host,
            identity: authClient.getIdentity(),
          },
        }
      );
      const uploadService = new UploadService(actor_user);

      authStore.setLoggedout(
        actor_user,
        actor_orchestrator,
        authClient,
        uploadService
      );
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
        const actor_user = createActorUser(this.canisterIdUser, {
          agentOptions: {
            host: this.host,
            identity: store.authClient.getIdentity(),
          },
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
        const { userService, filesService, requestsService, uploadService } =
          createServices(actor_user, actor_orchestrator);

        authStore.setLoggedin(
          actor_user,
          actor_orchestrator,
          store.authClient,
          userService,
          filesService,
          requestsService,
          uploadService
        );
      } catch (e) {
        const actor_user = createActorUser(this.canisterIdUser, {
          agentOptions: {
            host: this.host,
            identity: store.authClient.getIdentity(),
          },
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
        const uploadService = new UploadService(actor_user);

        authStore.setLoggedout(
          actor_user,
          actor_orchestrator,
          store.authClient,
          uploadService
        );
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
        store.userService.reset();
        const actor_user = createActorUser(this.canisterIdUser, {
          agentOptions: {
            host: this.host,
            identity: store.authClient.getIdentity(),
          },
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
        const uploadService = new UploadService(actor_user);
        authStore.setLoggedout(
          actor_user,
          actor_orchestrator,
          store.authClient,
          uploadService
        );
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
  import.meta.env.VITE_USER_CANISTER_CANISTER_ID,
  import.meta.env.VITE_ORCHESTRATOR_CANISTER_ID,
  import.meta.env.VITE_HOST,
  import.meta.env.VITE_II_URL
);
