import File from "$lib/file";
import type {
  ActorTypeOrchestrator,
  ActorTypeUserCanister,
} from "$lib/shared/actor";
import { formatUploadDate } from "$lib/shared/dates";
import { enumIs } from "$lib/shared/enums";
import { flatten } from "$lib/shared/flatten";
import { unreachable } from "$lib/shared/unreachable";
import { Principal } from "@dfinity/principal";
import { writable } from "svelte/store";
import { canisterId as userCanisterId } from "../../../declarations/user_canister";
import type { PublicFileMetadata } from "../../../declarations/user_canister/user_canister.did";
import type { ExternalFileMetadata } from "./files";

type ProgressStore = {
  step: "initializing" | "downloading" | "decrypting";
  totalChunks: number;
  currentChunk: number;
};

const PROGRESS_INITIAL: ProgressStore = {
  step: "initializing",
  totalChunks: 0,
  currentChunk: 0,
};

export class DecryptService {
  aborted = false;
  progress = writable<ProgressStore>(PROGRESS_INITIAL);
  constructor(
    private actorUserCanister: ActorTypeUserCanister,
    private actorOrchestrator: ActorTypeOrchestrator
  ) {}

  reset() {
    this.aborted = false;
    this.progress.set(PROGRESS_INITIAL);
  }

  async decryptFile({
    fileId,
    fileCanisterId,
  }: {
    fileId: bigint;
    fileCanisterId: string;
  }): Promise<
    | {
        name: string;
        dataType: string;
        uploadDate: string;
        contents: ArrayBuffer;
        originalMetadata: any;
      }
    | "aborted"
  > {
    this.progress.set(PROGRESS_INITIAL);

    let maybeFile: PublicFileMetadata | ExternalFileMetadata | undefined;
    // for the momentfirs aanalyse owned files
    if (!fileCanisterId || fileCanisterId === userCanisterId) {
      let files = flatten(
        await Promise.all([
          // this.actor.get_shared_files(),
          this.actorUserCanister.get_requests(),
          //TODO add get_owned_files
          // this.actor.get_owned_files(),
        ])
      );
      maybeFile = files.find((entry) => entry.file_id == BigInt(fileId));
      if (maybeFile && enumIs(maybeFile.file_status, "pending")) {
        throw new Error("Error: File not uploaded");
      }

      if (maybeFile && enumIs(maybeFile.file_status, "partially_uploaded")) {
        throw new Error("Error: File partially uploaded");
      }
    }

    if (!maybeFile) {
      //file not founded within the user canister check shared files
      const external_files_resp = await this.actorOrchestrator.shared_files();
      const external_files =
        "SharedFiles" in external_files_resp
          ? external_files_resp.SharedFiles
          : [];

      const external_public_file_metadata = external_files.reduce(
        (acc, entry) => {
          entry[1].forEach((file) => {
            acc.push({
              file_id: file.file_id,
              file_name: file.file_name,
              user_canister_id: entry[0],
            });
          });
          return acc;
        },
        [] as ExternalFileMetadata[]
      );

      maybeFile = external_public_file_metadata.find(
        (file) =>
          file.file_id == BigInt(fileId) &&
          file.user_canister_id === Principal.fromText(fileCanisterId)
      );

      if (!maybeFile) {
        throw new Error("Error: File not found");
      }
    }

    if (this.aborted) return "aborted";

    this.progress.update((v) => ({
      ...v,
      step: "downloading",
    }));

    let downloadedFile = await this.actorUserCanister.download_file(
      BigInt(fileId),
      0n
    );

    if (this.aborted) return "aborted";

    if (enumIs(downloadedFile, "found_file")) {
      const totalChunks = Number(downloadedFile.found_file.num_chunks);
      this.progress.update((v) => ({
        ...v,
        totalChunks,
        currentChunk: 1,
      }));
      for (let i = 1; i < downloadedFile.found_file.num_chunks; i++) {
        const downloadedChunk = await this.actorUserCanister.download_file(
          BigInt(fileId),
          BigInt(i)
        );
        if (this.aborted) return "aborted";

        if (enumIs(downloadedChunk, "found_file")) {
          this.progress.update((v) => ({
            ...v,
            currentChunk: i + 1,
          }));
          const chunk = downloadedChunk.found_file.contents;

          const mergedArray = new Uint8Array(
            downloadedFile.found_file.contents.length + chunk.length
          );
          mergedArray.set(downloadedFile.found_file.contents, 0);
          mergedArray.set(chunk, downloadedFile.found_file.contents.length);

          downloadedFile.found_file.contents = mergedArray;
        } else if (enumIs(downloadedChunk, "not_found_file")) {
          throw new Error("Error: Chunk not found");
        } else if (enumIs(downloadedChunk, "permission_error")) {
          throw new Error("Permission error");
        }
      }
      this.progress.update((v) => ({
        ...v,
        step: "decrypting",
      }));

      let decryptedFile: File;
      try {
        function convertToArrayBuffer(buffer: ArrayBufferLike): ArrayBuffer {
          if (buffer instanceof ArrayBuffer) {
            return buffer;
          }
          // Create a new ArrayBuffer and copy data
          const arrayBuffer = new ArrayBuffer(buffer.byteLength);
          new Uint8Array(arrayBuffer).set(new Uint8Array(buffer));
          return arrayBuffer;
        }
        decryptedFile = await File.fromEncrypted(
          maybeFile.file_name,
          convertToArrayBuffer(
            (downloadedFile.found_file.contents as Uint8Array).buffer
          ),
          convertToArrayBuffer(
            (downloadedFile.found_file.owner_key as Uint8Array).buffer
          )
        );
      } catch {
        throw new Error(
          "Failed to decrypt file: " +
            ((maybeFile.file_name || "unnamed file") +
              ". You may be able to access this file with a different browser, as the decryption key is stored in the browser.")
        );
      }

      return {
        name: decryptedFile.name,
        dataType: downloadedFile.found_file.file_type,
        //FIXME:
        uploadDate: formatUploadDate(0n),
        contents: decryptedFile.contents,
        originalMetadata: maybeFile,
      };
    } else if (enumIs(downloadedFile, "not_found_file")) {
      throw new Error("File not found");
    } else if (enumIs(downloadedFile, "permission_error")) {
      throw new Error("Permission error");
    } else if (enumIs(downloadedFile, "not_uploaded_file")) {
      throw new Error("File not uploaded");
    } else {
      unreachable(downloadedFile);
    }
  }

  abort() {
    this.aborted = true;
  }

  subscribe = this.progress.subscribe;
  set = this.progress.set;
}
