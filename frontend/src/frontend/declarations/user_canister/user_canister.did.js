export const idlFactory = ({ IDL }) => {
  const UserCanisterInitArgs = IDL.Record({
    'owner' : IDL.Principal,
    'orchestrator' : IDL.Principal,
  });
  const UserCanisterInstallArgs = IDL.Variant({
    'Upgrade' : IDL.Null,
    'Init' : UserCanisterInitArgs,
  });
  const DeleteFileResponse = IDL.Variant({
    'Ok' : IDL.Null,
    'FailedToRevokeShare' : IDL.Text,
    'FileNotFound' : IDL.Null,
  });
  const FileData = IDL.Record({
    'contents' : IDL.Vec(IDL.Nat8),
    'owner_key' : IDL.Vec(IDL.Nat8),
    'file_type' : IDL.Text,
    'num_chunks' : IDL.Nat64,
  });
  const FileDownloadResponse = IDL.Variant({
    'found_file' : FileData,
    'permission_error' : IDL.Null,
    'not_uploaded_file' : IDL.Null,
    'not_found_file' : IDL.Null,
  });
  const AliasInfo = IDL.Record({
    'file_name' : IDL.Text,
    'file_id' : IDL.Nat64,
  });
  const GetAliasInfoError = IDL.Variant({ 'not_found' : IDL.Null });
  const Result = IDL.Variant({ 'Ok' : AliasInfo, 'Err' : GetAliasInfoError });
  const FileStatus = IDL.Variant({
    'partially_uploaded' : IDL.Null,
    'pending' : IDL.Record({ 'alias' : IDL.Text, 'requested_at' : IDL.Nat64 }),
    'uploaded' : IDL.Record({
      'document_key' : IDL.Vec(IDL.Nat8),
      'uploaded_at' : IDL.Nat64,
    }),
  });
  const PublicFileMetadata = IDL.Record({
    'file_status' : FileStatus,
    'file_name' : IDL.Text,
    'shared_with' : IDL.Vec(IDL.Principal),
    'file_id' : IDL.Nat64,
  });
  const FileSharingResponse = IDL.Variant({
    'ok' : IDL.Null,
    'permission_error' : IDL.Null,
    'pending_error' : IDL.Null,
    'file_not_found' : IDL.Null,
  });
  const UploadFileRequest = IDL.Record({
    'owner_key' : IDL.Vec(IDL.Nat8),
    'file_type' : IDL.Text,
    'num_chunks' : IDL.Nat64,
    'file_content' : IDL.Vec(IDL.Nat8),
    'file_id' : IDL.Nat64,
  });
  const UploadFileError = IDL.Variant({
    'not_requested' : IDL.Null,
    'already_uploaded' : IDL.Null,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : UploadFileError });
  const UploadFileAtomicRequest = IDL.Record({
    'content' : IDL.Vec(IDL.Nat8),
    'owner_key' : IDL.Vec(IDL.Nat8),
    'name' : IDL.Text,
    'file_type' : IDL.Text,
    'num_chunks' : IDL.Nat64,
  });
  const UploadFileContinueRequest = IDL.Record({
    'contents' : IDL.Vec(IDL.Nat8),
    'chunk_id' : IDL.Nat64,
    'file_id' : IDL.Nat64,
  });
  const UploadFileContinueResponse = IDL.Variant({
    'ok' : IDL.Null,
    'file_not_found' : IDL.Null,
    'file_already_uploaded' : IDL.Null,
    'chunk_already_uploaded' : IDL.Null,
    'chunk_out_of_bounds' : IDL.Null,
  });
  return IDL.Service({
    'delete_file' : IDL.Func([IDL.Nat64], [DeleteFileResponse], []),
    'download_file' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [FileDownloadResponse],
        ['query'],
      ),
    'get_alias_info' : IDL.Func([IDL.Text], [Result], ['query']),
    'get_requests' : IDL.Func([], [IDL.Vec(PublicFileMetadata)], ['query']),
    'get_shared_files' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(PublicFileMetadata)],
        ['query'],
      ),
    'public_key' : IDL.Func([], [IDL.Vec(IDL.Nat8)], ['query']),
    'request_file' : IDL.Func([IDL.Text], [IDL.Text], []),
    'revoke_share' : IDL.Func([IDL.Principal, IDL.Nat64], [], []),
    'set_public_key' : IDL.Func([IDL.Vec(IDL.Nat8)], [], []),
    'share_file' : IDL.Func(
        [IDL.Principal, IDL.Nat64, IDL.Vec(IDL.Nat8)],
        [FileSharingResponse],
        [],
      ),
    'share_file_with_users' : IDL.Func(
        [IDL.Vec(IDL.Principal), IDL.Nat64, IDL.Vec(IDL.Vec(IDL.Nat8))],
        [],
        [],
      ),
    'upload_file' : IDL.Func([UploadFileRequest], [Result_1], []),
    'upload_file_atomic' : IDL.Func([UploadFileAtomicRequest], [IDL.Nat64], []),
    'upload_file_continue' : IDL.Func(
        [UploadFileContinueRequest],
        [UploadFileContinueResponse],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const UserCanisterInitArgs = IDL.Record({
    'owner' : IDL.Principal,
    'orchestrator' : IDL.Principal,
  });
  const UserCanisterInstallArgs = IDL.Variant({
    'Upgrade' : IDL.Null,
    'Init' : UserCanisterInitArgs,
  });
  return [UserCanisterInstallArgs];
};
