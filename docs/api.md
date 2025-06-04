# API

This document describes the API of the Docutrack canisters. The API is divided into two parts: the Orchestrator canister and the User canister.

## Orchestrator Canister

```did
service : (OrchestratorInstallArgs) -> {
  get_user : (principal) -> (opt PublicUser) query;
  get_users : (Pagination, opt text) -> (GetUsersResponse) query;
  orbit_station : () -> (principal) query;
  retry_user_canister_creation : () -> (RetryUserCanisterCreationResponse);
  revoke_share_file : (principal, nat64) -> (RevokeShareFileResponse);
  revoke_share_file_for_users : (vec principal, nat64) -> (
      RevokeShareFileResponse,
    );
  set_user : (text, blob) -> (SetUserResponse);
  share_file : (principal, nat64, ShareFileMetadata) -> (ShareFileResponse);
  share_file_with_users : (vec principal, nat64, ShareFileMetadata) -> (
      ShareFileResponse,
    );
  shared_files : () -> (SharedFilesResponse) query;
  user_canister : () -> (UserCanisterResponse) query;
  username_exists : (text) -> (bool) query;
  who_am_i : () -> (WhoamiResponse) query;
}
```

### get_user

Returns the public information of a user by their user ID.

Arguments:

- `user_id`: The user ID of the user to retrieve.

Returns:

- `opt PublicUser`: An optional `PublicUser` object containing the user's public information, or `null` if the user does not exist.

### get_users

Returns a paginated list of users.

Arguments:

- `Pagination`: The pagination parameters to use for the query.
- `opt text`: An optional search term to filter users by username.

Returns:

- `GetUsersResponse`: A response object containing a list of users and pagination information.

### orbit_station

Returns the principal of the Orbit Station canister.

Returns:

- `principal`: The principal of the Orbit Station canister.

### retry_user_canister_creation

Retries the creation of a user canister for the current user.

Returns:

`RetryUserCanisterCreationResponse`: A response object indicating the result of the retry operation.

### revoke_share_file

Revoke access to a shared file for a specific user.

Can only be called by the user canister.

Arguments:

- `user_id`: The user ID of the user to revoke access from.
- `file_id`: The ID of the file to revoke access to.

Returns:

`RevokeShareFileResponse`: A response object indicating the result of the revocation operation.

### revoke_share_file_for_users

Revoke access to a shared file for multiple users.

Can only be called by the user canister.

Arguments:

- `user_ids`: A vector of user IDs to revoke access from.
- `file_id`: The ID of the file to revoke access to.

Returns:

`RevokeShareFileResponse`: A response object indicating the result of the revocation operation.

### set_user

Sign up with internet identity by providing a username. This call causes the orchestrator to start the worker which will result in the creation of the user canister for the user.

Arguments:

- `username`: The username to set for the user.
- `blob`: The blob containing the user's Public Key

Returns:

`SetUserResponse`: A response object indicating the result of the user creation operation.

### share_file

Share a file with a specific user.

Can only be called by the user canister.

Arguments:

- `user_id`: The user ID of the user to share the file with.
- `file_id`: The ID of the file to share.
- `ShareFileMetadata`: Metadata about the file to share, such as the file name and description.

Returns:

`ShareFileResponse`: A response object indicating the result of the share operation.

### share_file_with_users

Share a file with multiple users.

Can only be called by the user canister.

Arguments:

- `user_ids`: A vector of user IDs to share the file with.
- `file_id`: The ID of the file to share.
- `ShareFileMetadata`: Metadata about the file to share, such as the file name and description.

Returns:

`ShareFileResponse`: A response object indicating the result of the share operation.

### shared_files

Returns a list of files shared with the current user.

Returns:

- `SharedFilesResponse`: A response object containing a list of files shared with the user.

### user_canister

Returns the principal of the user canister or its creation state if not created yet/pending/failed for the current user.

Returns:

- `UserCanisterResponse`: A response object containing the principal of the user canister or its creation state.

### username_exists

Returns whether a username already exists.

Arguments:

- `username`: The username to check for existence.

Returns:

- `bool`: `true` if the username exists, `false` otherwise.

### who_am_i

Returns the public information of the current user.

Returns:

- `WhoamiResponse`: A response object containing the public information of the current user, including their user ID and username.

## User Canister

```did
service : (UserCanisterInstallArgs) -> {
  delete_file : (nat64) -> (DeleteFileResponse);
  download_file : (nat64, nat64) -> (FileDownloadResponse) query;
  get_alias_info : (text) -> (Result) query;
  get_requests : () -> (vec PublicFileMetadata) query;
  get_shared_files : (principal) -> (vec PublicFileMetadata) query;
  public_key : () -> (blob) query;
  request_file : (text) -> (RequestFileResponse);
  revoke_share : (principal, nat64) -> ();
  set_public_key : (blob) -> ();
  share_file : (principal, nat64, blob) -> (FileSharingResponse);
  share_file_with_users : (vec principal, nat64, vec blob) -> ();
  upload_file : (UploadFileRequest) -> (Result_1);
  upload_file_atomic : (UploadFileAtomicRequest) -> (UploadFileAtomicResponse);
  upload_file_continue : (UploadFileContinueRequest) -> (
      UploadFileContinueResponse,
    );
}
```

### delete_file

Deletes a file from the user's storage canister.

Arguments:

- `file_id`: The ID of the file to delete.

Returns:

`DeleteFileResponse`: A response object indicating the result of the delete operation.

### download_file

Downloads a file from the user's storage canister.

Arguments:

- `file_id`: The ID of the file to download.
- `chunk`: The chunk number to download.

Returns:

`FileDownloadResponse`: A response object containing the file data and metadata.

### get_alias_info

Returns information about a file alias that is being uploaded.

Arguments:

- `alias`: The alias of the file to retrieve information for.

Returns:

`Result`: A response object containing the result of the alias lookup, including the file ID and metadata if found.

### get_requests

Returns a list of file requests made by the user.

Returns:

`vec PublicFileMetadata`: A vector of `PublicFileMetadata` objects containing information about the file requests.

### get_shared_files

Returns a list of files shared with a specific user.

Arguments:

- `user_id`: The user ID of the user to retrieve shared files for.

Returns:

`vec PublicFileMetadata`: A vector of `PublicFileMetadata` objects containing information about the shared files.

### public_key

Returns the public key of the user.

Returns:

`blob`: The public key of the user in binary format.

### request_file

Creates a new file request for the user for uploading a file.

Arguments:

- `path`: The path where the file will be uploaded.

Returns:

`RequestFileResponse`: A response object containing the opreation result. In case of success, it contains the file alias (UUIDv7) that can be used to upload the file.

### revoke_share

Revokes access to a shared file for a specific user.

Arguments:

- `user_id`: The user ID of the user to revoke access from.
- `file_id`: The ID of the file to revoke access to.

### set_public_key

Updates the public key of the user.

Arguments:

- `blob`: The new public key of the user in binary format.

### share_file (2)

Shares a file with a specific user.

Arguments:

- `user_id`: The user ID of the user to share the file with.
- `file_id`: The ID of the file to share.
- `blob`: file key encrypted with the user's public key.

### share_file_with_users (2)

Shares a file with multiple users.

Arguments:

- `user_ids`: A vector of user IDs to share the file with.
- `file_id`: The ID of the file to share.
- `vec blob`: A vector of file keys encrypted with the users' public keys.

### upload_file

Uploads the first chunk of a file to the user's storage canister.

Arguments:

- `UploadFileRequest`: An object containing the file data and metadata to upload.

Returns:

A response object indicating the result of the upload operation.

### upload_file_atomic

Uploads a file atomically to the user's storage canister. This means that the file is either created and uploaded in one go, or not created at all.

Arguments:

- `UploadFileAtomicRequest`: An object containing the file data and metadata to upload.

Returns:

`UploadFileAtomicResponse`: A response object indicating the result of the atomic upload operation.

### upload_file_continue

Upload any chunk after the first one of a file that was started with `upload_file`.

All chunks after the first one must be uploaded with this method.

Arguments:

- `UploadFileContinueRequest`: An object containing the file data and metadata to continue the upload.

Returns:

`UploadFileContinueResponse`: A response object indicating the result of the continued upload operation.
