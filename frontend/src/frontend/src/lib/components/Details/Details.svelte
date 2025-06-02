<script lang="ts">
  import { page } from "$app/stores";
  import FilePreview from "$lib/components/FilePreview.svelte";
  import ShareModal from "$lib/components/ShareModal.svelte";
  import BackIcon from "$lib/components/icons/BackIcon.svelte";
  import DownloadIcon from "$lib/components/icons/DownloadIcon.svelte";
  import ShareIcon from "$lib/components/icons/ShareIcon.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { DecryptService } from "$lib/services/decrypt";
  import { ObjectUrlManager } from "$lib/services/objectUrls";
  import { authStore } from "$lib/services/auth";
  import { unreachable } from "$lib/shared/unreachable";
  import { onDestroy, onMount } from "svelte";
  import type { PublicFileMetadata } from "../../../../declarations/user_canister/user_canister.did";
  import ErrorMessage from "../ErrorMessage.svelte";
  import DecryptProgress from "./DecryptProgress.svelte";

  export let auth: AuthStateAuthenticated;

  let decryptService: DecryptService | null = null;
  const objectUrls = new ObjectUrlManager();

  function getFileId() {
    return parseInt($page.url.searchParams.get("fileId") || "");
  }
  function getFileCanisterId() {
    return $page.url.searchParams.get("fileCanisterId") || "";
  }
  type State =
    | {
        type: "uninitialized";
      }
    | { type: "initializing" }
    | {
        type: "loading";
      }
    | {
        type: "loaded";
        name: string;
        dataType: string;
        uploadDate: string;
        downloadUrl: string;
        isOpenShareModal: boolean;
        originalMetadata: PublicFileMetadata;
      }
    | {
        type: "error";
        error: string;
      };

  let state: State = {
    type: "uninitialized",
  };

  onMount(async () => {
    if (
      auth.canisterRetrievalState === "retrieved" &&
      auth.actor_user &&
      auth.actor_orchestrator
    ) {
      decryptService = new DecryptService(
        auth.actor_user,
        auth.actor_orchestrator
      );
      initialize();
    } else if (auth.canisterRetrievalState === "failed") {
      state = {
        type: "error",
        error: "Unable to initialize due to a system error.",
      };
    } else {
      state = { type: "initializing" };
      const unsubscribe = authStore.subscribe((authState) => {
        if (authState.state === "authenticated") {
          if (
            authState.canisterRetrievalState === "retrieved" &&
            authState.actor_user &&
            authState.actor_orchestrator
          ) {
            decryptService = new DecryptService(
              authState.actor_user,
              authState.actor_orchestrator
            );
            initialize();
          } else if (authState.canisterRetrievalState === "failed") {
            state = {
              type: "error",
              error: "Unable to initialize due to a system error.",
            };
          }
        }
      });
      return () => unsubscribe();
    }
  });

  onDestroy(() => {
    if (decryptService) decryptService.abort();
    objectUrls.clear();
  });

  function openShareDialog() {
    if (state.type === "loaded") {
      state = {
        ...state,
        isOpenShareModal: true,
      };
    }
  }

  async function initialize() {
    if (!decryptService) {
      state = {
        type: "error",
        error: "System not ready. Please try again later.",
      };
      return;
    }
    decryptService.reset();
    state = { type: "loading" };

    const fileId = BigInt(getFileId());
    const fileCanisterId = getFileCanisterId();
    try {
      const file = await decryptService.decryptFile({
        fileId,
        fileCanisterId,
      });

      if (file === "aborted") {
        console.log("file download/decrypt aborted");

        state = {
          type: "error",
          error: "File not found.",
        };
        return;
      }

      state = {
        type: "loaded",
        downloadUrl: objectUrls.createObjectURLFromArrayBuffer(
          file.contents,
          file.dataType
        ),
        dataType: file.dataType,
        name: file.name,
        uploadDate: file.uploadDate,
        originalMetadata: file.originalMetadata,
        isOpenShareModal: false,
      };
    } catch (e: unknown) {
      state = {
        type: "error",
        error:
          e instanceof Error
            ? e.message || "Error decrypting file."
            : "Error opening file: " + e,
      };
    }
  }
</script>

<section>
  <a href="/" class="btn btn-ghost">
    <BackIcon /> Back to files
  </a>
  {#if state.type === "uninitialized" || state.type === "initializing"}
    <div class="title-1 mb-2 mt-3 text-text-200">Initializing...</div>
  {:else if state.type === "loading"}
    <div class="title-1 mb-2 mt-3 text-text-200">Loading...</div>

    <DecryptProgress
      progress={$decryptService || {
        step: "initializing",
        totalChunks: 0,
        currentChunk: 0,
      }}
    />
  {:else if state.type === "error"}
    <ErrorMessage class="mt-6">{state.error}</ErrorMessage>
  {:else if state.type === "loaded"}
    <h1 class="title-1 mb-2 mt-3">
      {#if state.name}
        {state.name}
      {:else}
        <span class="opacity-50">Unnamed file</span>
      {/if}
    </h1>
    <p class="mb-6 text-text-200">Uploaded: {state.uploadDate}</p>
    <div class="mb-6 flex gap-2">
      <a
        href={state.downloadUrl}
        class="btn btn-accent md:w-64"
        download={state.name}
      >
        <DownloadIcon />
        Download</a
      >

      <button class="btn btn-accent md:w-64" on:click={openShareDialog}>
        <ShareIcon /> Share
      </button>
    </div>
    <FilePreview
      file={{
        objectUrl: state.downloadUrl,
        dataType: state.dataType,
      }}
    />
    <ShareModal
      {auth}
      bind:isOpen={state.isOpenShareModal}
      bind:fileData={state.originalMetadata}
    />
  {:else}
    {unreachable(state)}
  {/if}
</section>
