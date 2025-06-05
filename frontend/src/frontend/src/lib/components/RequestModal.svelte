<script lang="ts">
  import { page } from "$app/stores";
  import { createEventDispatcher, onMount } from "svelte";
  import Modal from "./Modal.svelte";
  import CopyIcon from "./icons/CopyIcon.svelte";
  import ErrorMessage from "./ErrorMessage.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { authStore } from "$lib/services/auth";
  import { enumIs } from "$lib/shared/enums";

  export let isOpen = false;
  export let auth: AuthStateAuthenticated;

  let requestLink: URL | null = null;
  let loading: boolean = false;
  let requestName: string = "";
  let copied = false;
  let loadingState: "loading" | "error" | "ready" = "loading";
  let error: string | null = null;

  const dispatch = createEventDispatcher<{
    "request-created": void;
    "request-completed": void;
  }>();

  onMount(() => {
    if (auth.canisterRetrievalState === "retrieved" && auth.actor_user) {
      loadingState = "ready";
    } else if (auth.canisterRetrievalState === "failed") {
      loadingState = "error";
      error = "Unable to initialize due to a system error.";
    } else {
      // pending or uninitialized: wait for authStore to update
      const unsubscribe = authStore.subscribe((state) => {
        if (state.state === "authenticated") {
          if (
            state.canisterRetrievalState === "retrieved" &&
            state.actor_user
          ) {
            loadingState = "ready";
          } else if (state.canisterRetrievalState === "failed") {
            loadingState = "error";
            error = "Unable to initialize due to a system error.";
          }
        }
      });
      return () => unsubscribe();
    }
  });

  async function updateRequestUrl(e: any) {
    if (!auth.actor_user) {
      error = "System not ready. Please try again later.";
      loading = false;
      return;
    }
    loading = true;
    error = null;
    const formData = new FormData(e.target);
    const data: any = {};
    for (let field of formData) {
      const [key, value] = field;
      data[key] = value;
    }

    // Do not request new url when there is already one
    if (data.requestName && !data.requestLink) {
      requestName = data.requestName;
      const alias = await auth.actor_user.request_file("/" + data.requestName);
      if (enumIs(alias, "FileAlreadyExists")) {
        console.error("Error requesting file:", alias.FileAlreadyExists);
        error = `File already exists: ${alias.FileAlreadyExists}`;
        loading = false;
        return;
      }
      console.log("Alias received:", alias.Ok);
      requestLink = new URL($page.url.origin + "/upload");
      requestLink.searchParams.append("alias", alias.Ok);
      if (auth.userCanisterId) {
        requestLink.searchParams.append("usercanister", auth.userCanisterId);
      }
    }
    loading = false;

    dispatch("request-created");
  }

  function close() {
    if (requestLink) {
      dispatch("request-completed");
    }

    isOpen = false;
    requestName = "";
    requestLink = null;
  }

  async function copyText() {
    if (requestLink) {
      await navigator.clipboard.writeText(requestLink.toString());
      copied = true;
    }
  }
</script>

<div>
  <Modal bind:isOpen title="Create Request" on:cancelled={close}>
    {#if loadingState === "loading"}
      <p class="body-1 text-text-200 mb-4">Initializing...</p>
    {:else if loadingState === "error"}
      <ErrorMessage class="mb-4">
        {error || "Unable to initialize request creation."}
      </ErrorMessage>
    {:else}
      <form class="w-full md:w-96" on:submit|preventDefault={updateRequestUrl}>
        {#if error}
          <ErrorMessage class="mb-4">{error}</ErrorMessage>
        {/if}
        <div class="">
          <label for="requestName" class="input-label">Request Name</label>

          <input
            type="text"
            required={true}
            class="input"
            id="requestName"
            placeholder="Enter your input"
            name="requestName"
            disabled={!!requestLink || loading}
            readonly={!!requestLink}
          />
        </div>
        <div class="mt-3">
          {#if requestLink}
            <div class="flex justify-between items-center">
              <label for="requestLink" class="input-label">
                Request Link
              </label>
              {#if copied}
                <span class="text-text-100 body-1"> Copied! </span>
              {/if}
            </div>
            <div class="relative">
              <input
                type="text"
                class="input pr-10"
                id="requestLink"
                placeholder=""
                name="requestLink"
                value={requestLink}
                readonly
              />
              <button
                class="btn btn-icon absolute right-0 top-1/2 -translate-y-1/2"
                on:click={copyText}
              >
                <CopyIcon />
              </button>
            </div>
            <div class="mt-4">
              <a
                href="mailto:?subject=Share your file&body=Please share a file with me here: {requestLink}"
                class="text-accent-100">Send in email</a
              >
            </div>
          {/if}
        </div>
        <div class=" mt-10">
          {#if loading}
            <button type="submit" class="btn btn-accent btn-full btn-" disabled
              >Generating link...</button
            >
          {:else if !loading && requestLink}
            <button
              type="button"
              class="btn btn-accent btn-full"
              on:click={close}>Request sent, close this window</button
            >
          {:else}
            <button type="submit" class="btn btn-accent btn-full"
              >Generate link</button
            >
          {/if}
        </div>
      </form>
    {/if}
  </Modal>
</div>
