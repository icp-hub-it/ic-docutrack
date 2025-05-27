import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type GetUsersResponse = { 'permission_error' : null } |
  { 'users' : GetUsersResponseUsers };
export interface GetUsersResponseUsers {
  'total' : bigint,
  'next' : [] | [bigint],
  'users' : Array<PublicUser>,
}
export interface OrchestratorInitArgs {
  'orbit_station_admin' : string,
  'orbit_station' : Principal,
}
export type OrchestratorInstallArgs = { 'Upgrade' : null } |
  { 'Init' : OrchestratorInitArgs };
export interface Pagination { 'offset' : bigint, 'limit' : bigint }
export interface PublicFileMetadata { 'file_name' : string, 'file_id' : bigint }
export interface PublicUser {
  'username' : string,
  'public_key' : Uint8Array | number[],
  'ic_principal' : Principal,
}
export type RetryUserCanisterCreationResponse = { 'Ok' : null } |
  { 'CreationPending' : null } |
  { 'Created' : Principal } |
  { 'UserNotFound' : null } |
  { 'AnonymousCaller' : null };
export type RevokeShareFileResponse = { 'Ok' : null } |
  { 'NoSuchUser' : Principal } |
  { 'Unauthorized' : null };
export type SetUserResponse = { 'ok' : null } |
  { 'username_too_long' : null } |
  { 'username_exists' : null } |
  { 'caller_has_already_a_user' : null } |
  { 'anonymous_caller' : null };
export interface ShareFileMetadata { 'file_name' : string }
export type ShareFileResponse = { 'Ok' : null } |
  { 'NoSuchUser' : Principal } |
  { 'Unauthorized' : null };
export type SharedFilesResponse = {
    'SharedFiles' : Array<[Principal, Array<PublicFileMetadata>]>
  } |
  { 'NoSuchUser' : null } |
  { 'AnonymousUser' : null };
export type UserCanisterResponse = { 'Ok' : Principal } |
  { 'CreationFailed' : { 'reason' : string } } |
  { 'CreationPending' : null } |
  { 'Uninitialized' : null } |
  { 'AnonymousCaller' : null };
export type WhoamiResponse = { 'known_user' : PublicUser } |
  { 'unknown_user' : null };
export interface _SERVICE {
  'get_user' : ActorMethod<[Principal], [] | [PublicUser]>,
  'get_users' : ActorMethod<[Pagination], GetUsersResponse>,
  'orbit_station' : ActorMethod<[], Principal>,
  'retry_user_canister_creation' : ActorMethod<
    [],
    RetryUserCanisterCreationResponse
  >,
  'revoke_share_file' : ActorMethod<
    [Principal, bigint],
    RevokeShareFileResponse
  >,
  'revoke_share_file_for_users' : ActorMethod<
    [Array<Principal>, bigint],
    RevokeShareFileResponse
  >,
  'set_user' : ActorMethod<[string, Uint8Array | number[]], SetUserResponse>,
  'share_file' : ActorMethod<
    [Principal, bigint, ShareFileMetadata],
    ShareFileResponse
  >,
  'share_file_with_users' : ActorMethod<
    [Array<Principal>, bigint, ShareFileMetadata],
    ShareFileResponse
  >,
  'shared_files' : ActorMethod<[], SharedFilesResponse>,
  'user_canister' : ActorMethod<[], UserCanisterResponse>,
  'username_exists' : ActorMethod<[string], boolean>,
  'who_am_i' : ActorMethod<[], WhoamiResponse>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
