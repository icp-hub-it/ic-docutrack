import { toArrayBuffer } from "$lib/buffer";
import crypto from "$lib/crypto";
import FileTools from "$lib/file";
import type { ActorTypeUserCanister } from "$lib/shared/actor";
import { enumIs } from "$lib/shared/enums";
import pLimit from "p-limit";
import { writable } from "svelte/store";
import type { Result as GetAliasInfoResponse } from "../../../declarations/user_canister/user_canister.did";

export const CHUNK_SIZE = 2_000_000;

export const uploadInProgress = writable(false);

export type UploadType =
  | {
      type: "request";
      fileInfo: Extract<GetAliasInfoResponse, { Ok: any }>["Ok"];
    }
  | {
      type: "self";
      filePath: string;
    };

export class UploadService {
  aborted = false;

  constructor(private actor: ActorTypeUserCanister) {}

  async uploadFile({
    uploadType,
    file,
    onChunkUploaded = () => {},
    onCompleted = () => {},
    onError = () => {},
    dataType,
    onStarted = () => {},
    onAborted = () => {},
  }: {
    uploadType: UploadType;
    file: File;
    dataType: string;
    onStarted?: (totalSizeBytes: number) => void;
    onChunkUploaded?: (chunkId: number, size: number) => void;
    onCompleted?: (file_id: bigint) => void;
    onError?: (message: string) => void;
    onAborted?: () => void;
  }) {
    const userPublicKey =
      uploadType.type === "request"
        ? toArrayBuffer((uploadType.fileInfo.public_key as Uint8Array).buffer)
        : await crypto.getLocalUserPublicKey();

    console.log("User public key:", userPublicKey);
    const filePath =
      uploadType.type === "request"
        ? uploadType.fileInfo.file_path + uploadType.fileInfo.file_name
        : uploadType.filePath;

    const fileBytes = await file.arrayBuffer();
    let fileToEncrypt = FileTools.fromUnencrypted(filePath, fileBytes);
    const encryptedFileKey = await fileToEncrypt.getEncryptedFileKey(
      userPublicKey
    );

    const encFile = await fileToEncrypt.encrypt();
    const content = new Uint8Array(encFile);

    if (content.length > 100 * 1024 * 1024) {
      onError(
        "File size is limited to 100MiB in this PoC\n(larger files could be supported in a production version)."
      );
      return;
    }

    // Split file into chunks of 2MB.
    const numChunks = Math.ceil(content.length / CHUNK_SIZE);

    try {
      onStarted(content.length);

      const firstChunk = content.subarray(0, CHUNK_SIZE);
      let fileId: bigint = 0n;
      if (uploadType.type === "request") {
        fileId = uploadType.fileInfo.file_id;
        const res = await this.actor.upload_file({
          file_id: fileId,
          file_content: firstChunk,
          owner_key: new Uint8Array(encryptedFileKey),
          file_type: dataType,
          num_chunks: BigInt(numChunks),
        });

        if (enumIs(res, "Err")) {
          onError(
            "An error occurred while uploading the file. Please try again."
          );
          return;
        }
      } else {
        const response = await this.actor.upload_file_atomic({
          content: firstChunk,
          owner_key: new Uint8Array(encryptedFileKey),
          path: filePath,
          file_type: dataType,
          num_chunks: BigInt(numChunks),
        });
        console.log("calling canister upload_file_atomic");
        if (enumIs(response, "FileAlreadyExists")) {
          onError("File already exists. Please choose a different file name.");
          return;
        } else if (enumIs(response, "Ok")) {
          fileId = response.Ok;
        }
      }

      onChunkUploaded(0, firstChunk.length);

      if (this.aborted) {
        onAborted();
        return;
      }

      await this.uploadChunks(content, fileId, onChunkUploaded);

      if (this.aborted) {
        onAborted();
        return;
      }

      onCompleted(fileId);
    } catch (err) {
      console.error(err);
      onError("An error occurred while uploading the file. Please try again.");
    }
  }

  private async uploadChunks(
    content: Uint8Array,
    fileId: bigint,
    onChunkUploaded: (chunkId: number, size: number) => void
  ) {
    const numChunks = Math.ceil(content.length / CHUNK_SIZE);

    // Create upload pool, supporting upto 5 parallel uploads.
    const uploadPool = pLimit(5);

    // Prepare upload requests.
    const uploadRequests = Array.from(
      { length: numChunks - 1 },
      (_, i) => i + 1
    ).map((i) =>
      uploadPool(async () => {
        if (this.aborted) {
          return;
        }
        const chunk = content.subarray(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE);
        await this.actor.upload_file_continue({
          file_id: fileId,
          contents: chunk,
          chunk_id: BigInt(i),
        });
        onChunkUploaded(i, chunk.length);
      })
    );

    await Promise.all(uploadRequests);
  }

  async abort() {
    this.aborted = true;
  }
}
