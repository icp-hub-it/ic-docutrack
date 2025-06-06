<script lang="ts">
  import RequestModal from "../RequestModal.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { authStore } from "$lib/services/auth";
  import { onMount } from "svelte";
  import { filesStore, type ExternalFileMetadata } from "$lib/services/files";
  import { unreachable } from "$lib/shared/unreachable";
  import { goto } from "$app/navigation";
  import ShareIcon from "../icons/ShareIcon.svelte";
  import PlaceholderLogo from "../icons/PlaceholderLogo.svelte";
  import ShareModal from "../ShareModal.svelte";
  import type { PublicFileMetadata } from "../../../../declarations/user_canister/user_canister.did";
  import DeleteIcon from "../icons/DeleteIcon.svelte";

  export let auth: AuthStateAuthenticated;
  let isOpenRequestModal = false;
  let isOpenShareModal = false;
  let shareFileData: PublicFileMetadata | ExternalFileMetadata | null;
  let loadingState: "loading" | "error" | "ready" = "loading";

  onMount(() => {
    if (auth.canisterRetrievalState === "retrieved" && auth.filesService) {
      auth.filesService.reload();
      loadingState = "ready";
    } else if (auth.canisterRetrievalState === "failed") {
      loadingState = "error";
    } else {
      // pending or uninitialized: wait for authStore to update
      const unsubscribe = authStore.subscribe((state) => {
        if (state.state === "authenticated") {
          if (
            state.canisterRetrievalState === "retrieved" &&
            state.filesService
          ) {
            state.filesService.reload();
            loadingState = "ready";
          } else if (state.canisterRetrievalState === "failed") {
            loadingState = "error";
          }
        }
      });
      return () => unsubscribe();
    }
  });

  function goToDetails(file_id: bigint, user_canister: string = "") {
    if (user_canister) {
      goto(`/details?fileId=${file_id}&fileCanisterId=${user_canister}`);
    } else {
      goto(`/details?fileId=${file_id}`);
    }
  }

  function openShareModal(
    file: PublicFileMetadata | ExternalFileMetadata | null
  ) {
    shareFileData = file;
    isOpenShareModal = true;
  }

  async function removeFile(file_id: bigint) {
    if (!auth.filesService) {
      filesStore.setError("Files service not initialized");
      return;
    }
    if (!confirm("Are you sure you want to delete this file?")) {
      return;
    }
    try {
      await auth.filesService.remove_file(file_id);
    } catch (e) {
      console.error("Failed to remove file:", e);
    }
  }
</script>

{#if loadingState === "loading"}
  <h1 class="title-1">Initializing...</h1>
{:else if loadingState === "error"}
  <div class="">
    <h1 class="title-1">My Files</h1>
    <p>Error: Unable to load files due to initialization failure.</p>
  </div>
{:else if $filesStore.state === "idle" || $filesStore.state === "loading"}
  <h1 class="title-1">Loading...</h1>
{:else if $filesStore.state === "error"}
  <div class="">
    <h1 class="title-1">My Files</h1>
    <p>Error loading files: {$filesStore.error}</p>
  </div>
{:else if $filesStore.state === "loaded"}
  <div class="flex justify-between items-center mb-6">
    <h1 class="title-1">My Files</h1>
    {#if $filesStore.files.length > 0}
      <button
        class="hidden md:inline-block btn btn-accent"
        on:click={() => (isOpenRequestModal = true)}
        >Create new file request</button
      >
    {/if}
  </div>
  {#if $filesStore.files.length > 0}
    <div class="hidden md:block bg-background-200 w-full rounded-2xl px-2">
      <table class="table-auto w-full border-spacing-y-2 border-separate">
        <thead class="">
          <tr class="body-2 text-text-200 text-left">
            <th class="body-2 pt-4 pb-2 pl-4">Name</th>
            <th class="body-2 pt-6 pb-2">Access</th>
            <th class="body-2 pt-6 pb-2">Uploaded at</th>
            <th />
          </tr>
        </thead>
        <tbody class="">
          {#each $filesStore.files as file}
            <!-- {console.log("File data:", file)} -->
            <tr
              class="hover:drop-shadow-xl cursor-pointer text-text-100"
              on:click={() => goToDetails(file.file_id, file.user_canister_id)}
            >
              <td
                class="pl-4 bg-background-100 rounded-tl-xl rounded-bl-xl body-1"
              >
                {#if file.path}
                  {file.path.slice(1)}
                {:else}
                  <span class="opacity-50">Unnamed file</span>
                {/if}
              </td>
              <td class="bg-background-100 body-1">{file.access}</td>
              <td class="bg-background-100 body-1">{file.uploadedAt}</td>
              {#if !file.external}
                <td
                  class="pr-4 bg-background-100 rounded-tr-xl rounded-br-xl body-1 w-32 text-right h-[52px]"
                >
                  <button
                    on:click|preventDefault|stopPropagation={() =>
                      openShareModal(file.metadata)}
                    class="btn btn-icon"
                  >
                    <ShareIcon />
                  </button>
                  <button
                    on:click|preventDefault|stopPropagation={() =>
                      removeFile(file.file_id)}
                    class="btn btn-icon btn-danger"
                  >
                    <DeleteIcon />
                  </button>
                </td>
              {:else}
                <td
                  class="pr-4 bg-background-100 rounded-tr-xl rounded-br-xl body-1 w-32 text-right h-[52px]"
                />
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <div class="md:hidden flex flex-col gap-2">
      {#each $filesStore.files as file}
        <a
          class="bg-white rounded-xl py-3 px-4 flex flex-col"
          href="/details?fileId={file.file_id}${file.external
            ? `&fileCanisterId=${file.user_canister_id}`
            : ''}"
        >
          <div class="flex justify-between items-center mb-3">
            <span class="text-text-100 title-2">
              {#if file.path}
                {file.path}
              {:else}
                <span class="opacity-50">Unnamed file</span>
              {/if}
            </span>
            <span>
              {#if !file.external}
                <button
                  on:click|preventDefault|stopPropagation={() =>
                    openShareModal(file.metadata)}
                  class="btn btn-icon"
                >
                  <ShareIcon />
                </button>
                <button
                  on:click|preventDefault|stopPropagation={() =>
                    removeFile(file.file_id)}
                  class="btn btn-icon btn-danger"
                >
                  <DeleteIcon />
                </button>
              {/if}
            </span>
          </div>
          <div class="flex flex-col gap-2">
            <div class="flex justify-between items-center">
              <span class="body-1 text-text-200">Access:</span>
              <span class="body-1 text-text-100">{file.access}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="body-1 text-text-200">Uploaded at:</span>
              <span class="body-1 text-text-100">{file.uploadedAtShort}</span>
            </div>
          </div>
        </a>
      {/each}
    </div>
  {:else}
    <div
      class="panel pt-10 pb-4 text-center flex flex-col items-center gap-4 mt-6"
    >
      <PlaceholderLogo />
      <h2 class="title-2 text-text-200">
        Even when you have no documents, rest assured, your data is secure.
      </h2>
      <div class="pt-4 pb-8">
        <button
          class="btn btn-accent md:w-96"
          on:click|preventDefault={() => (isOpenRequestModal = true)}
          >Create new file request</button
        >
      </div>
    </div>
  {/if}
{:else}
  {unreachable($filesStore)}
{/if}
<div class="md:hidden fixed bottom-0 left-0 right-0 bg-background-200 p-4">
  <button
    class="btn btn-accent btn-full"
    on:click={() => (isOpenRequestModal = true)}>Create new file request</button
  >
</div>
<RequestModal bind:isOpen={isOpenRequestModal} {auth} />
{#if shareFileData}
  <ShareModal
    {auth}
    bind:isOpen={isOpenShareModal}
    bind:fileData={shareFileData}
    on:shared={() => auth.filesService?.reload()}
  />
{/if}
