<script lang="ts">
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import { authStore } from "$lib/services/auth";
  import { userStore } from "$lib/services/user";
  import { unreachable } from "$lib/shared/unreachable";
  import { onMount, onDestroy } from "svelte";
  import { writable } from "svelte/store";
  import ErrorMessage from "./ErrorMessage.svelte";
  import Modal from "./Modal.svelte";

  export let isOpen = false;
  export let authenticatedStore: AuthStateAuthenticated;
  let usernameValue: string = "";
  let loadingState: "loading" | "error" | "ready" = "loading";

  const messages = [
    "Verifing username...",
    "Scheduling canister creation...",
    "Creating canister...",
    "Waiting for canister to be created...",
    "Installing software...",
    "Registering canister in index...",
    "Please wait...",
  ];
  let messageIndex = 0;
  const currentMessage = writable(messages[messageIndex]);
  let intervalId: NodeJS.Timeout | null = null;

  function startMessageRotation() {
    if (intervalId) return;
    intervalId = setInterval(() => {
      messageIndex = (messageIndex + 1) % messages.length;
      currentMessage.set(messages[messageIndex]);
      // Stop rotation at "Please wait..." (last message)
      if (messageIndex === messages.length - 1 && intervalId) {
        clearInterval(intervalId);
        intervalId = null;
      }
    }, 3000);
  }

  function stopMessageRotation() {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
    messageIndex = 0;
    currentMessage.set(messages[0]);
  }

  onMount(() => {
    if (
      authenticatedStore.canisterRetrievalState === "retrieved" &&
      authenticatedStore.userService
    ) {
      loadingState = "ready";
    } else if (
      authenticatedStore.canisterRetrievalState === "uninitialized" &&
      authenticatedStore.userService
    ) {
      loadingState = "ready";
    } else if (authenticatedStore.canisterRetrievalState === "failed") {
      loadingState = "error";
    } else {
      // pending or uninitialized: wait for authStore to update
      const unsubscribe = authStore.subscribe((state) => {
        if (state.state === "authenticated") {
          if (
            state.canisterRetrievalState === "retrieved" &&
            state.userService
          ) {
            loadingState = "ready";
          } else if (state.canisterRetrievalState === "failed") {
            loadingState = "error";
          }
        }
      });
      return () => unsubscribe();
    }

    const unsubscribeUserStore = userStore.subscribe((store) => {
      if (
        store.state === "unregistered" &&
        store.registrationState.state === "registering"
      ) {
        startMessageRotation();
      } else {
        stopMessageRotation();
      }
    });

    return () => {
      unsubscribeUserStore();
      stopMessageRotation();
    };
  });

  onDestroy(() => {
    stopMessageRotation();
  });

  function register() {
    if ($userStore.state === "unregistered" && authenticatedStore.userService) {
      authenticatedStore.userService.register(usernameValue);
    }
  }
</script>

<div>
  <Modal {isOpen} title="Register Yourself" mandatory>
    {#if loadingState === "loading"}
      <p class="body-1 text-text-200 mb-4">Initializing...</p>
    {:else if loadingState === "error"}
      <ErrorMessage class="mb-4">
        Unable to initialize registration due to a system error.
      </ErrorMessage>
    {:else}
      <form class="" on:submit|preventDefault={() => register()}>
        <p class="body-1 text-text-200 mb-4">
          Your Internet Identity is not connected with a username yet. Choose a
          username to setup an account on DocuTrack. Your username will be
          publicly visible
        </p>
        <div class="mb-4">
          <label for="username" class="input-label">Username</label>
          {#if $userStore.state === "unregistered"}
            {#if $userStore.registrationState.state === "registering"}
              <input
                disabled
                type="text"
                required
                class="input"
                bind:value={usernameValue}
                placeholder="Username"
              />
            {:else}
              <input
                type="text"
                required
                class="input"
                bind:value={usernameValue}
                placeholder="Username"
              />
            {/if}
          {/if}
        </div>
        <div class="mt-10">
          {#if $userStore.state === "unregistered"}
            {#if $userStore.registrationState.state === "registering"}
              <button type="button" class="btn btn-full btn-accent" disabled
                >{$currentMessage}</button
              >
            {:else if $userStore.registrationState.state === "error"}
              <ErrorMessage class="mb-4">
                {$userStore.registrationState.errorMessage}
              </ErrorMessage>
              <button type="submit" class="btn btn-full btn-accent"
                >Submit</button
              >
            {:else if $userStore.registrationState.state === "idle"}
              <button type="submit" class="btn btn-full btn-accent"
                >Submit</button
              >
            {:else}
              {unreachable($userStore.registrationState)}
            {/if}
          {:else}
            <button type="submit" class="btn btn-full btn-accent">Submit</button
            >
          {/if}
        </div>
      </form>
    {/if}
  </Modal>
</div>
