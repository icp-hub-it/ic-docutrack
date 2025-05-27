import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AliasInfo { 'file_name' : string, 'file_id' : bigint }
export type DeleteFileResponse = { 'Ok' : null } |
  { 'FailedToRevokeShare' : string } |
  { 'FileNotFound' : null };
export interface FileData {
  'contents' : Uint8Array | number[],
  'owner_key' : Uint8Array | number[],
  'file_type' : string,
  'num_chunks' : bigint,
}
export type FileDownloadResponse = { 'found_file' : FileData } |
  { 'permission_error' : null } |
  { 'not_uploaded_file' : null } |
  { 'not_found_file' : null };
export type FileSharingResponse = { 'ok' : null } |
  { 'permission_error' : null } |
  { 'pending_error' : null } |
  { 'file_not_found' : null };
export type FileStatus = { 'partially_uploaded' : null } |
  { 'pending' : { 'alias' : string, 'requested_at' : bigint } } |
  {
    'uploaded' : {
      'document_key' : Uint8Array | number[],
      'uploaded_at' : bigint,
    }
  };
export type GetAliasInfoError = { 'not_found' : null };
export interface PublicFileMetadata {
  'file_status' : FileStatus,
  'file_name' : string,
  'shared_with' : Array<Principal>,
  'file_id' : bigint,
}
export type Result = { 'Ok' : AliasInfo } |
  { 'Err' : GetAliasInfoError };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : UploadFileError };
export interface UploadFileAtomicRequest {
  'content' : Uint8Array | number[],
  'owner_key' : Uint8Array | number[],
  'name' : string,
  'file_type' : string,
  'num_chunks' : bigint,
}
export interface UploadFileContinueRequest {
  'contents' : Uint8Array | number[],
  'chunk_id' : bigint,
  'file_id' : bigint,
}
export type UploadFileContinueResponse = { 'ok' : null } |
  { 'file_not_found' : null } |
  { 'file_already_uploaded' : null } |
  { 'chunk_already_uploaded' : null } |
  { 'chunk_out_of_bounds' : null };
export type UploadFileError = { 'not_requested' : null } |
  { 'already_uploaded' : null };
export interface UploadFileRequest {
  'owner_key' : Uint8Array | number[],
  'file_type' : string,
  'num_chunks' : bigint,
  'file_content' : Uint8Array | number[],
  'file_id' : bigint,
}
export interface UserCanisterInitArgs {
  'owner' : Principal,
  'orchestrator' : Principal,
}
export type UserCanisterInstallArgs = { 'Upgrade' : null } |
  { 'Init' : UserCanisterInitArgs };
export interface _SERVICE {
  'delete_file' : ActorMethod<[bigint], DeleteFileResponse>,
  'download_file' : ActorMethod<[bigint, bigint], FileDownloadResponse>,
  'get_alias_info' : ActorMethod<[string], Result>,
  'get_requests' : ActorMethod<[], Array<PublicFileMetadata>>,
  'get_shared_files' : ActorMethod<[Principal], Array<PublicFileMetadata>>,
  'public_key' : ActorMethod<[], Uint8Array | number[]>,
  'request_file' : ActorMethod<[string], string>,
  'revoke_share' : ActorMethod<[Principal, bigint], undefined>,
  'set_public_key' : ActorMethod<[Uint8Array | number[]], undefined>,
  'share_file' : ActorMethod<
    [Principal, bigint, Uint8Array | number[]],
    FileSharingResponse
  >,
  'share_file_with_users' : ActorMethod<
    [Array<Principal>, bigint, Array<Uint8Array | number[]>],
    undefined
  >,
  'upload_file' : ActorMethod<[UploadFileRequest], Result_1>,
  'upload_file_atomic' : ActorMethod<[UploadFileAtomicRequest], bigint>,
  'upload_file_continue' : ActorMethod<
    [UploadFileContinueRequest],
    UploadFileContinueResponse
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
