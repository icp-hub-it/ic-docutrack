export const idlFactory = ({ IDL }) => {
  const OrchestratorInitArgs = IDL.Record({
    'orbit_station_admin' : IDL.Text,
    'orbit_station' : IDL.Principal,
  });
  const OrchestratorInstallArgs = IDL.Variant({
    'Upgrade' : IDL.Null,
    'Init' : OrchestratorInitArgs,
  });
  const PublicUser = IDL.Record({
    'username' : IDL.Text,
    'public_key' : IDL.Vec(IDL.Nat8),
    'ic_principal' : IDL.Principal,
  });
  const Pagination = IDL.Record({ 'offset' : IDL.Nat64, 'limit' : IDL.Nat64 });
  const GetUsersResponseUsers = IDL.Record({
    'total' : IDL.Nat64,
    'next' : IDL.Opt(IDL.Nat64),
    'users' : IDL.Vec(PublicUser),
  });
  const GetUsersResponse = IDL.Variant({
    'permission_error' : IDL.Null,
    'users' : GetUsersResponseUsers,
  });
  const RetryUserCanisterCreationResponse = IDL.Variant({
    'Ok' : IDL.Null,
    'CreationPending' : IDL.Null,
    'Created' : IDL.Principal,
    'UserNotFound' : IDL.Null,
    'AnonymousCaller' : IDL.Null,
  });
  const RevokeShareFileResponse = IDL.Variant({
    'Ok' : IDL.Null,
    'NoSuchUser' : IDL.Principal,
    'Unauthorized' : IDL.Null,
  });
  const SetUserResponse = IDL.Variant({
    'ok' : IDL.Null,
    'username_too_long' : IDL.Null,
    'username_exists' : IDL.Null,
    'caller_has_already_a_user' : IDL.Null,
    'anonymous_caller' : IDL.Null,
  });
  const ShareFileMetadata = IDL.Record({ 'file_name' : IDL.Text });
  const ShareFileResponse = IDL.Variant({
    'Ok' : IDL.Null,
    'NoSuchUser' : IDL.Principal,
    'Unauthorized' : IDL.Null,
  });
  const PublicFileMetadata = IDL.Record({
    'file_name' : IDL.Text,
    'shared_with' : IDL.Vec(PublicUser),
    'file_id' : IDL.Nat64,
  });
  const SharedFilesResponse = IDL.Variant({
    'SharedFiles' : IDL.Vec(
      IDL.Tuple(IDL.Principal, IDL.Vec(PublicFileMetadata))
    ),
    'NoSuchUser' : IDL.Null,
    'AnonymousUser' : IDL.Null,
  });
  const UserCanisterResponse = IDL.Variant({
    'Ok' : IDL.Principal,
    'CreationFailed' : IDL.Record({ 'reason' : IDL.Text }),
    'CreationPending' : IDL.Null,
    'Uninitialized' : IDL.Null,
    'AnonymousCaller' : IDL.Null,
  });
  const WhoamiResponse = IDL.Variant({
    'known_user' : PublicUser,
    'unknown_user' : IDL.Null,
  });
  return IDL.Service({
    'get_user' : IDL.Func([IDL.Principal], [IDL.Opt(PublicUser)], ['query']),
    'get_users' : IDL.Func([Pagination], [GetUsersResponse], ['query']),
    'orbit_station' : IDL.Func([], [IDL.Principal], ['query']),
    'retry_user_canister_creation' : IDL.Func(
        [],
        [RetryUserCanisterCreationResponse],
        [],
      ),
    'revoke_share_file' : IDL.Func(
        [IDL.Principal, IDL.Nat64],
        [RevokeShareFileResponse],
        [],
      ),
    'revoke_share_file_for_users' : IDL.Func(
        [IDL.Vec(IDL.Principal), IDL.Nat64],
        [RevokeShareFileResponse],
        [],
      ),
    'set_user' : IDL.Func([IDL.Text, IDL.Vec(IDL.Nat8)], [SetUserResponse], []),
    'share_file' : IDL.Func(
        [IDL.Principal, IDL.Nat64, ShareFileMetadata],
        [ShareFileResponse],
        [],
      ),
    'share_file_with_users' : IDL.Func(
        [IDL.Vec(IDL.Principal), IDL.Nat64, ShareFileMetadata],
        [ShareFileResponse],
        [],
      ),
    'shared_files' : IDL.Func([], [SharedFilesResponse], ['query']),
    'user_canister' : IDL.Func([], [UserCanisterResponse], ['query']),
    'username_exists' : IDL.Func([IDL.Text], [IDL.Bool], ['query']),
    'who_am_i' : IDL.Func([], [WhoamiResponse], ['query']),
  });
};
export const init = ({ IDL }) => {
  const OrchestratorInitArgs = IDL.Record({
    'orbit_station_admin' : IDL.Text,
    'orbit_station' : IDL.Principal,
  });
  const OrchestratorInstallArgs = IDL.Variant({
    'Upgrade' : IDL.Null,
    'Init' : OrchestratorInitArgs,
  });
  return [OrchestratorInstallArgs];
};
