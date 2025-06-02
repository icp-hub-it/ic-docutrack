import type {
  ActorTypeOrchestrator,
  ActorTypeUserCanister,
} from "$lib/shared/actor";
import { formatUploadDate, formatUploadDateShort } from "$lib/shared/dates";
import { enumIs } from "$lib/shared/enums";
import { unreachable } from "$lib/shared/unreachable";
import type { Principal } from "@dfinity/principal";
import { get, writable } from "svelte/store";
import type { PublicFileMetadata } from "../../../declarations/orchestrator/orchestrator.did";
import type { PublicFileMetadata as PublicFileMetadataCAN } from "../../../declarations/user_canister/user_canister.did";

export interface ExternalFileMetadata extends PublicFileMetadata {
  file_id: bigint;
  file_name: string;
  user_canister_id: Principal;
}

export function isPublicFileMetadata(
  file: ExternalFileMetadata | PublicFileMetadata
): file is PublicFileMetadata {
  return (file as PublicFileMetadata).file_id !== undefined;
}

export type UploadedFile = {
  path: string;
  access: string;
  uploadedAt: string;
  uploadedAtShort: string;
  file_id: bigint;
  metadata: PublicFileMetadataCAN | ExternalFileMetadata;
  user_canister_id?: string;
  external?: boolean; // Indicates if the file is external
};

export type FilesState =
  | {
      state: "idle";
    }
  | {
      state: "loading";
    }
  | {
      state: "error";
      error: string;
    }
  | {
      state: "loaded";
      files: UploadedFile[];
      reloading: boolean;
    };

function createFilesStore() {
  const { subscribe, set } = writable<FilesState>({
    state: "idle",
  });

  return {
    subscribe,
    set,
    setLoaded: (files: UploadedFile[], reloading: boolean) => {
      set({
        state: "loaded",
        files,
        reloading,
      });
    },
    setLoading: () => {
      set({
        state: "loading",
      });
    },
    setError: (error: string) => {
      set({
        state: "error",
        error,
      });
    },
    reset: () => set({ state: "idle" }),
  };
}

export const filesStore = createFilesStore();

export class FilesService {
  constructor(
    private actorUserCanister: ActorTypeUserCanister,
    private actorOrchestrator: ActorTypeOrchestrator
  ) {}

  async init() {
    filesStore.setLoading();
    try {
      const files = await this.loadFiles();

      filesStore.setLoaded(files, false);
    } catch (e: unknown) {
      filesStore.setError("Failed to load files");
    }
  }

  async reload() {
    const store = get(filesStore);
    if (store.state === "loading") {
      return;
    } else if (store.state === "loaded") {
      filesStore.setLoaded(store.files, true);
    } else if (store.state === "error" || store.state === "idle") {
      filesStore.setLoading();
    } else {
      unreachable(store);
    }
    try {
      const files = await this.loadFiles();
      filesStore.setLoaded(files, false);
    } catch (e: unknown) {
      filesStore.setError("Failed to load files");
    }
  }

  private async loadFiles(): Promise<UploadedFile[]> {
    // Fetch oowned files
    const files: PublicFileMetadataCAN[] =
      await this.actorUserCanister.get_requests();

    const uploadedFiles: UploadedFile[] = [];

    for (const file of files) {
      if (enumIs(file.file_status, "uploaded")) {
        // Determine the sharing status
        let nShared = file.shared_with ? file.shared_with.length : 0;
        let accessMessage = "";
        switch (nShared) {
          case 0:
            accessMessage = "Only You";
            break;
          case 1:
            accessMessage = "You & 1 other";
            break;
          default:
            accessMessage = "You & " + nShared + " others";
        }

        uploadedFiles.push({
          path: file.file_path,
          access: accessMessage,
          uploadedAt: formatUploadDate(file.file_status.uploaded.uploaded_at),
          uploadedAtShort: formatUploadDateShort(
            file.file_status.uploaded.uploaded_at
          ),
          file_id: file.file_id,
          metadata: file,
        });
      }
    }

    // Fetch shared files
    // Note: This will add external files with external metadata
    const resp = await this.actorOrchestrator.shared_files();
    let res_unwrapped: Array<[Principal, Array<PublicFileMetadata>]> = [];
    if (enumIs(resp, "AnonymousUser") || enumIs(resp, "NoSuchUser"))
      throw new Error("Failed to load shared Anonymous User or No Such User");

    if (enumIs(resp, "SharedFiles")) res_unwrapped = resp.SharedFiles;
    // adding external files with external metadata
    for (const external of res_unwrapped) {
      const user_canister_id = external[0];
      const files_metadata = external[1];
      for (const file of files_metadata) {
        let nShared = file.shared_with ? file.shared_with.length : 0;
        let accessMessage = "";
        switch (nShared) {
          case 0:
            accessMessage = "Only You";
            break;
          case 1:
            accessMessage = "You & 1 other";
            break;
          default:
            accessMessage = "You & " + nShared + " others";
        }
        uploadedFiles.push({
          path: file.file_name,
          access: accessMessage,
          uploadedAt: "Unknown",
          uploadedAtShort: "Unknown",
          file_id: file.file_id,
          external: true,
          user_canister_id: user_canister_id.toText(),
          metadata: {
            file_id: file.file_id,
            file_name: file.file_name,
            user_canister_id: user_canister_id,
            shared_with: file.shared_with,
          },
        });
      }
    }
    return uploadedFiles;
  }

  async remove_file(file_id: bigint): Promise<void> {
    try {
      const deleteResp = await this.actorUserCanister.delete_file(file_id);
      if (enumIs(deleteResp, "FileNotFound")) {
        throw new Error("File not found");
      } else if (enumIs(deleteResp, "FailedToRevokeShare")) {
        throw new Error(deleteResp.FailedToRevokeShare);
      } else if (enumIs(deleteResp, "Ok")) {
        // Reload the file list to reflect the deletion
        await this.reload();
      }
    } catch (e: unknown) {
      filesStore.setError(
        `Failed to remove file: ${e instanceof Error ? e.message : String(e)}`
      );
      throw e;
    }
  }
}
