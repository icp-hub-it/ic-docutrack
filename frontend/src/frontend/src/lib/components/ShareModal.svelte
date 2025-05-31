<script lang="ts">
  import { default as crypto } from "$lib/crypto";
  import { enumIs } from "$lib/shared/enums";
  import type { Principal } from "@dfinity/principal";
  import { createEventDispatcher, onMount } from "svelte";
  import type { PublicFileMetadata } from "../../../declarations/user_canister/user_canister.did";
  import type { ExternalFileMetadata } from "../services/files";
  import type { PublicUser } from "../../../declarations/orchestrator/orchestrator.did";
  import Modal from "./Modal.svelte";
  import CloseIcon from "./icons/CloseIcon.svelte";
  import type { AuthStateAuthenticated } from "$lib/services/auth";
  import ErrorMessage from "./ErrorMessage.svelte";
  import ComboBox from "./ComboBox.svelte";
  import { flatten } from "$lib/shared/flatten";
  import { toArrayBuffer } from "$lib/buffer";
  import { authStore } from "$lib/services/auth";

  export let auth: AuthStateAuthenticated;

  export let isOpen = false;
  export let fileData: PublicFileMetadata | ExternalFileMetadata;

  let loadingState: "loading" | "error" | "ready" = "loading";
  let error: string | null = null;

  function isPublicFileMetadata(
    file: PublicFileMetadata | ExternalFileMetadata
  ): file is PublicFileMetadata {
    return (file as PublicFileMetadata).file_id !== undefined;
  }

  const dispatch = createEventDispatcher<{
    shared: { file_id: bigint; shared_with: PublicUser[] };
  }>();

  let expirationDate: string | null = null;
  let loading: boolean = false;
  let users: PublicUser[] = [];
  let oldSharedWith: PublicUser[] = [];
  let newSharedWith: PublicUser[] = [];
  let initError: string = "";

  function reset() {
    expirationDate = null;
    loading = false;
    error = "";
  }

  function removeItem(arr: PublicUser[], value: PublicUser) {
    var index = arr.indexOf(value);
    if (index > -1) {
      arr.splice(index, 1);
    }
    return arr;
  }

  function addPersonToShare(user: { label: string; value: Principal }) {
    const maybeUser = users.find(
      (obj) => obj.ic_principal.compareTo(user.value) === "eq"
    );

    const principalNotYetAdded = !newSharedWith.find(
      (obj) => obj.ic_principal.compareTo(user.value) === "eq"
    );

    if (!!maybeUser && principalNotYetAdded) {
      newSharedWith = [...newSharedWith, maybeUser];
    }
  }

  function removePersonFromShare(principal: Principal) {
    let user = newSharedWith.find(
      (obj) => obj.ic_principal.compareTo(principal) === "eq"
    );
    if (user !== undefined) {
      newSharedWith = removeItem(newSharedWith, user);
      // Assign to itself for reactivity purposes
      newSharedWith = newSharedWith;
    }
  }

  async function saveShare() {
    if (!auth.actor_user) {
      error = "System not ready. Please try again later.";
      loading = false;
      return;
    }

    if (
      isPublicFileMetadata(fileData) &&
      !enumIs(fileData.file_status, "uploaded")
    ) {
      return;
    }

    loading = true;
    error = "";
    // If no expiration date is used, set to -1
    let timestamp = -1;
    if (expirationDate) {
      // The expiration date is saved as timestamp in nanoseconds, convert accordingly
      timestamp = Date.parse(expirationDate) * 1e6;
    }
    // initializing empty maybe FIX TODO no need to escape ExternalFileMetadata
    let documentKey: ArrayBuffer = new ArrayBuffer(0);
    try {
      if (
        isPublicFileMetadata(fileData) &&
        enumIs(fileData.file_status, "uploaded")
      ) {
        documentKey = await crypto.decryptForUser(
          toArrayBuffer(
            (fileData.file_status.uploaded.document_key as Uint8Array).buffer
          )
        );
      }
    } catch {
      error =
        "Error: unable to access file. You may be able to access this file with a different browser, as the decryption key is stored in the browser.";
      loading = false;
      return;
    }
    for (let i = 0; i < newSharedWith.length; i++) {
      try {
        const encryptedFileKey = await crypto.encryptForUser(
          documentKey,
          toArrayBuffer((newSharedWith[i].public_key as Uint8Array).buffer)
        );
        // TODO: add expiration date to backend call
        const response = await auth.actor_user.share_file(
          newSharedWith[i].ic_principal,
          fileData.file_id,
          new Uint8Array(encryptedFileKey)
        );
        console.log(
          response ? "File shared successfully" : "File share failed"
        );
      } catch (err) {
        console.error(err);
        error = `Error: could not share file with ${newSharedWith[i].username}`;
        loading = false;
        return;
      }
    }
    // Go over all old entries and remove the ones that are no longer in the shared list
    for (let i = 0; i < oldSharedWith.length; i++) {
      try {
        let res = newSharedWith.find(
          (obj) =>
            obj.ic_principal.compareTo(oldSharedWith[i].ic_principal) === "eq"
        );
        if (!res) {
          await auth.actor_user.revoke_share(
            oldSharedWith[i].ic_principal,
            fileData.file_id
          );
        }
      } catch {
        error = `Error: could not revoke share with ${oldSharedWith[i].username}.`;
        loading = false;
        return;
      }
    }
    // Write back the new state, so the the UI updates
    fileData.shared_with = newSharedWith
      .slice()
      .map((user) => user.ic_principal);
    fileData = fileData;
    isOpen = false;
    loading = false;

    dispatch("shared", {
      file_id: fileData.file_id,
      shared_with: newSharedWith.slice(),
    });
  }

  async function onOpen(isOpen: boolean) {
    if (isOpen) {
      let sharedWith: PublicUser[] = [];
      if (!isPublicFileMetadata(fileData)) {
        sharedWith = fileData.shared_with;
      } else {
        const proms = fileData.shared_with.map((principal) =>
          auth.actor_orchestrator.get_user(principal)
        );
        sharedWith = flatten(await Promise.all(proms));
      }
      // Keep the old version of the shared users
      oldSharedWith = sharedWith.slice();
      // Copy the array and modify this list with the UI
      newSharedWith = sharedWith.slice();

      reset();
    }
  }

  // We want to ensure that `oldSharedWith` is only updated at the beginning of a new sharing
  $: onOpen(isOpen);

  $: selfPrincipal = auth.authClient.getIdentity().getPrincipal();

  $: availableUsers = users.filter(
    (obj) =>
      obj.ic_principal.compareTo(selfPrincipal) !== "eq" &&
      !newSharedWith.find(
        (obj2) => obj.ic_principal.compareTo(obj2.ic_principal) === "eq"
      )
  );

  onMount(async () => {
    let res = await auth.actor_orchestrator.get_users({
      offset: 0n,
      limit: 127n,
    });
    if (enumIs(res, "users")) {
      users = res.users.users.filter(
        (obj) => obj.ic_principal.compareTo(selfPrincipal) !== "eq"
      );
    } else {
      users = [];
    }
  });
</script>

<div>
  <Modal title={`Share "${fileData.file_name || "Unnamed file"}"`} bind:isOpen>
    <form class="flex flex-col gap-4" on:submit|preventDefault={saveShare}>
      <p class="text-body-1 text-text-100">
        Choose the people that have access to this file.
      </p>

      {#if newSharedWith.length > 0}
        <div class="flex flex-wrap gap-2">
          {#each newSharedWith as user}
            <button
              type="button"
              on:click={() => removePersonFromShare(user.ic_principal)}
              class="rounded-full bg-silver py-1 pl-2 pr-1 flex gap-2 text-body-1 text-text-200"
              >{user.username}

              <span
                class="bg-silver-700 rounded-full text-white w-4 h-4 flex items-center justify-center"
              >
                <CloseIcon />
              </span>
            </button>
          {/each}
        </div>
      {/if}

      <div class="">
        <label for="shareWith" class="input-label">Share with</label>
        <ComboBox
          notFoundMessage="No such user"
          on:select={(e) => addPersonToShare(e.detail)}
          id="shareWith"
          items={availableUsers?.map((a) => ({
            label: a.username,
            value: a.ic_principal,
          })) || []}
          disabled={availableUsers && availableUsers.length === 0}
          placeholder={availableUsers?.length === 0
            ? "No users to share with"
            : "Select a user..."}
        />
      </div>

      <div>
        <label for="expirationDate" class="input-label"
          >Expiration date (optional)</label
        >
        <input
          type="date"
          class="input"
          id="expirationDate"
          bind:value={expirationDate}
        />
      </div>

      {#if error}
        <ErrorMessage>{error}</ErrorMessage>
      {/if}

      <div class="mt-6">
        <button class="btn btn-accent btn-full" disabled={loading}
          >{#if loading}
            Saving changes...
          {:else}
            Save changes
          {/if}</button
        >
      </div>
    </form>
  </Modal>
</div>
