mod create_user;

use candid::Principal;
use create_user::CreateUserStateMachine;
use did::orchestrator::{
    FileId, GetUsersResponse, GetUsersResponseUsers, MAX_USERNAME_SIZE, OrchestratorInstallArgs,
    Pagination, PublicFileMetadata, PublicKey, PublicUser, RetryUserCanisterCreationResponse,
    RevokeShareFileResponse, SetUserResponse, ShareFileMetadata, ShareFileResponse,
    SharedFilesResponse, User, UserCanisterResponse, WhoamiResponse,
};

use crate::debug;
use crate::storage::config::Config;
use crate::storage::shared_files::SharedFilesStorage;
use crate::storage::user_canister::{UserCanisterCreateState, UserCanisterStorage};
use crate::storage::users::UserStorage;
use crate::utils::{msg_caller, trap};

/// Maximum number of users to retrieve at once.
const MAX_GET_USERS_LIMIT: u64 = 128;
/// Minimum length of the query string for getting users.
const GET_USERS_QUERY_MIN_LENGTH: usize = 4;

/// API for Business Logic
pub struct Canister;

impl Canister {
    /// Initialize the canister with the given arguments.
    pub fn init(args: OrchestratorInstallArgs) {
        let OrchestratorInstallArgs::Init(args) = args else {
            trap("Invalid arguments");
        };

        debug!("Initializing canister with args: {:?}", args);

        Config::set_orbit_station(args.orbit_station);
        Config::set_orbit_station_admin(args.orbit_station_admin);
    }

    /// Get the users from the storage as [`GetUsersResponse`].
    ///
    /// If the caller is anonymous, it returns [`GetUsersResponse::PermissionError`].
    ///
    /// Up to 128 users can be retrieved at once.
    ///
    /// FIXME: this function should be protected.
    ///
    /// # Arguments
    ///
    /// - `Pagination`: The pagination parameters, including `offset` and `limit`.
    /// - `query`: An optional query string to filter users by username. It has a minimum length of [`GET_USERS_QUERY_MIN_LENGTH`].
    pub fn get_users(
        Pagination { offset, limit }: Pagination,
        query: Option<&str>,
    ) -> GetUsersResponse {
        let limit = limit.min(MAX_GET_USERS_LIMIT);
        debug!("Getting users with offset: {offset}, limit: {limit}, query: {query:?}",);

        let caller = msg_caller();
        if caller == Principal::anonymous() {
            return GetUsersResponse::PermissionError;
        }

        // Validate the query length.
        if query.is_some_and(|q| q.len() < GET_USERS_QUERY_MIN_LENGTH) {
            debug!("Query is too short: {query:?}",);
            return GetUsersResponse::InvalidQuery;
        }

        let filter = |user: &User| {
            if let Some(query) = query {
                user.username.contains(query)
            } else {
                true
            }
        };

        let users = UserStorage::get_users_in_range(offset, limit, filter)
            .into_iter()
            .map(|(principal, user)| PublicUser::new(user, principal))
            .collect::<Vec<_>>();

        let max_users = UserStorage::count(filter);
        let next = if offset + limit < max_users {
            Some(offset + limit)
        } else {
            None
        };

        GetUsersResponse::Users(GetUsersResponseUsers {
            users,
            total: max_users,
            next,
        })
    }

    /// Get a user from the storage as [`PublicUser`].
    pub fn get_user(principal: Principal) -> Option<PublicUser> {
        debug!("Getting user with principal: {principal}",);
        UserStorage::get_user(&principal).map(|user| PublicUser::new(user, principal))
    }

    /// Retry the user canister creation for the current caller.
    ///
    /// # Returns
    ///
    /// - [`RetryUserCanisterCreationResponse::Ok`] if the user canister creation is retried.
    /// - [`RetryUserCanisterCreationResponse::Created`] if the user canister already exists.
    /// - [`RetryUserCanisterCreationResponse::AnonymousCaller`]: The caller is anonymous.
    /// - [`RetryUserCanisterCreationResponse::CreationPending`]: The user canister creation is already in progress.
    /// - [`RetryUserCanisterCreationResponse::UserNotFound`]: The user doesn't exist. In that case, the caller should call `set_user` first.
    pub fn retry_user_canister_creation() -> RetryUserCanisterCreationResponse {
        debug!(
            "Retrying user canister creation for caller: {}",
            msg_caller()
        );
        let caller = msg_caller();
        if caller == Principal::anonymous() {
            return RetryUserCanisterCreationResponse::AnonymousCaller;
        }

        // check if the user exists
        if UserStorage::get_user(&caller).is_none() {
            return RetryUserCanisterCreationResponse::UserNotFound;
        }

        // check if the user canister already exists
        if let Some(canister) = UserCanisterStorage::get_user_canister(caller) {
            return RetryUserCanisterCreationResponse::Created(canister);
        }

        // check the current state of the user canister creation
        match UserCanisterStorage::get_create_state(caller) {
            Some(UserCanisterCreateState::Ok { user_canister }) => {
                RetryUserCanisterCreationResponse::Created(user_canister)
            }
            Some(UserCanisterCreateState::Failed { .. }) | None => {
                if cfg!(target_family = "wasm") {
                    CreateUserStateMachine::start(Config::get_orbit_station(), caller);
                }
                RetryUserCanisterCreationResponse::Ok
            }
            Some(_) => RetryUserCanisterCreationResponse::CreationPending,
        }
    }

    /// Revoke the share of a file for a user.
    ///
    /// # Returns
    ///
    /// - [`RevokeShareFileResponse::Ok`] if the file was unshared successfully.
    /// - [`RevokeShareFileResponse::NoSuchUser`] if the user doesn't exist.
    /// - [`RevokeShareFileResponse::Unauthorized`] if the caller is not a user canister.
    pub fn revoke_share_file(user: Principal, file_id: FileId) -> RevokeShareFileResponse {
        debug!("Revoking share for user: {user}, file_id: {file_id}",);
        let user_canister = msg_caller();
        // check if the caller is a user canister
        if !UserCanisterStorage::is_user_canister(user_canister) {
            return RevokeShareFileResponse::Unauthorized;
        }

        // Revoke share for the user
        SharedFilesStorage::revoke_share(user, user_canister, file_id);

        RevokeShareFileResponse::Ok
    }

    /// Revoke the share of a file for a list of users.
    ///
    /// # Returns
    ///
    /// - [`RevokeShareFileResponse::Ok`] if the file was unshared successfully.
    /// - [`RevokeShareFileResponse::NoSuchUser`] if the user doesn't exist.
    /// - [`RevokeShareFileResponse::Unauthorized`] if the caller is not a user canister.
    pub fn revoke_share_file_for_users(
        users: Vec<Principal>,
        file_id: FileId,
    ) -> RevokeShareFileResponse {
        debug!(
            "Revoking share for users: {:?}, file_id: {}",
            users, file_id
        );
        let user_canister = msg_caller();
        // check if the caller is a user canister
        if !UserCanisterStorage::is_user_canister(user_canister) {
            return RevokeShareFileResponse::Unauthorized;
        }

        // Revoke share for the user
        for user in users {
            SharedFilesStorage::revoke_share(user, user_canister, file_id);
        }

        RevokeShareFileResponse::Ok
    }

    /// Set a new user in the storage.
    ///
    /// # Returns
    ///
    /// - [`SetUserResponse::Ok`] if the user was set successfully.
    /// - [`SetUserResponse::AnonymousCaller`] if the caller is anonymous.
    /// - [`SetUserResponse::UsernameTooLong`] if the username is too long.
    /// - [`SetUserResponse::UsernameExists`] if the username already exists.
    /// - [`SetUserResponse::CallerHasAlreadyAUser`] if the caller already has a user.
    pub fn set_user(username: String, public_key: PublicKey) -> SetUserResponse {
        debug!("Setting user with username: {username}, public_key: {public_key:?}",);
        // Check if the caller is anonymous.
        let caller = msg_caller();
        if caller == Principal::anonymous() {
            return SetUserResponse::AnonymousCaller;
        }

        // Check if the username is too long.
        if username.len() > MAX_USERNAME_SIZE {
            return SetUserResponse::UsernameTooLong;
        }

        // check if username already exists
        if UserStorage::username_exists(&username) {
            return SetUserResponse::UsernameExists;
        }

        // check if the caller already has a user
        if UserStorage::get_user(&caller).is_some() {
            return SetUserResponse::CallerHasAlreadyAUser;
        }

        // Add the user to the storage and return Ok.
        UserStorage::add_user(
            caller,
            User {
                username,
                public_key,
            },
        );

        // start state machine to create user canister
        if cfg!(target_family = "wasm") {
            CreateUserStateMachine::start(Config::get_orbit_station(), caller);
        }

        SetUserResponse::Ok
    }

    /// Share a file with a user.
    ///
    /// # Returns
    ///
    /// - [`ShareFileResponse::Ok`] if the file was shared successfully.
    /// - [`ShareFileResponse::NoSuchUser`] if the user doesn't exist.
    /// - [`ShareFileResponse::Unauthorized`] if the caller is not a user canister.
    pub fn share_file(
        user: Principal,
        file_id: FileId,
        metadata: ShareFileMetadata,
    ) -> ShareFileResponse {
        debug!("Sharing file with user: {user}, file_id: {file_id}, metadata: {metadata:?}",);
        Self::share_file_with_users(vec![user], file_id, metadata)
    }

    /// Share a file with many users.
    ///
    /// # Returns
    ///
    /// - [`ShareFileResponse::Ok`] if the file was shared successfully.
    /// - [`ShareFileResponse::NoSuchUser`] if the user doesn't exist.
    /// - [`ShareFileResponse::Unauthorized`] if the caller is not a user canister.
    pub fn share_file_with_users(
        users: Vec<Principal>,
        file_id: FileId,
        metadata: ShareFileMetadata,
    ) -> ShareFileResponse {
        debug!(
            "Sharing file with users: {:?}, file_id: {}, metadata: {:?}",
            users, file_id, metadata
        );
        let user_canister = msg_caller();
        // check if the caller is a user canister
        if !UserCanisterStorage::is_user_canister(user_canister) {
            return ShareFileResponse::Unauthorized;
        }

        // check if all the users exist
        if let Some(no_such_user) = users
            .iter()
            .find(|user| UserStorage::get_user(user).is_none())
        {
            return ShareFileResponse::NoSuchUser(*no_such_user);
        }

        // share the file with all the users
        for user in users {
            SharedFilesStorage::share_file(user, user_canister, file_id, metadata.clone());
        }

        ShareFileResponse::Ok
    }

    /// Returns the list of shared files for the caller.
    ///
    /// # Returns
    ///
    /// - [`SharedFilesResponse::AnonymousUser`] if the caller is anonymous.
    /// - [`SharedFilesResponse::NoSuchUser`] if the user doesn't exist.
    /// - [`SharedFilesResponse::SharedFiles`] if the user exists and has shared files.
    pub fn shared_files() -> SharedFilesResponse {
        debug!("Getting shared files for caller: {}", msg_caller());
        let caller = msg_caller();
        if caller == Principal::anonymous() {
            return SharedFilesResponse::AnonymousUser;
        }

        // check if the user exists
        if UserStorage::get_user(&caller).is_none() {
            return SharedFilesResponse::NoSuchUser;
        }

        SharedFilesResponse::SharedFiles(
            SharedFilesStorage::get_shared_files(caller)
                .into_iter()
                .map(|(user_canister, files)| {
                    (
                        user_canister,
                        files
                            .into_iter()
                            .filter_map(|file_id| {
                                SharedFilesStorage::get_file_metadata(user_canister, file_id).map(
                                    |file_metadata| PublicFileMetadata {
                                        file_id,
                                        file_name: file_metadata.file_name,
                                        shared_with: SharedFilesStorage::shared_with(
                                            user_canister,
                                            file_id,
                                        )
                                        .into_iter()
                                        .filter_map(|principal| {
                                            UserStorage::get_user(&principal)
                                                .map(|user| PublicUser::new(user, principal))
                                        })
                                        .collect(),
                                    },
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect(),
        )
    }

    /// Checks whether a given username exists in the storage.
    pub fn username_exists(username: String) -> bool {
        debug!("Checking if username exists: {username}",);
        UserStorage::username_exists(&username)
    }

    /// Get user canister information for the current caller.
    ///
    /// # Returns
    ///
    /// - [`UserCanisterResponse::AnonymousCaller`] if the caller is anonymous.
    /// - [`UserCanisterResponse::Ok`] if the user canister is created and ready to use.
    /// - [`UserCanisterResponse::CreationPending`] if the user canister is being created.
    /// - [`UserCanisterResponse::CreationFailed`] if the user canister creation failed.
    pub fn user_canister() -> UserCanisterResponse {
        debug!("Getting user canister for caller: {}", msg_caller());
        let caller = msg_caller();
        if caller == Principal::anonymous() {
            return UserCanisterResponse::AnonymousCaller;
        }

        if let Some(canister) = UserCanisterStorage::get_user_canister(caller) {
            return UserCanisterResponse::Ok(canister);
        }

        // otherwise check if it failed or it is pending
        UserCanisterStorage::get_create_state(caller)
            .map(|state| match state {
                UserCanisterCreateState::Failed { reason } => {
                    UserCanisterResponse::CreationFailed { reason }
                }
                _ => UserCanisterResponse::CreationPending,
            })
            .unwrap_or(UserCanisterResponse::Uninitialized)
    }

    /// Get [`WhoamiResponse`] for the current caller.
    ///
    /// # Returns
    ///
    /// - [`WhoamiResponse::UnknownUser`] if the caller is anonymous or doesn't exist.
    /// - [`WhoamiResponse::KnownUser`] if the caller exists.
    pub fn whoami() -> WhoamiResponse {
        debug!("Getting whoami for caller: {}", msg_caller());
        let caller = msg_caller();
        if caller == Principal::anonymous() {
            return WhoamiResponse::UnknownUser;
        }

        UserStorage::get_user(&caller)
            .map(|user| PublicUser::new(user, caller))
            .map(WhoamiResponse::from)
            .unwrap_or(WhoamiResponse::UnknownUser)
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use did::orchestrator::{OrchestratorInitArgs, User};

    use super::*;

    #[test]
    fn test_should_init_canister() {
        let orbit_station = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();
        Canister::init(OrchestratorInstallArgs::Init(OrchestratorInitArgs {
            orbit_station,
            orbit_station_admin: "admin".to_string(),
        }));

        assert_eq!(Config::get_orbit_station(), orbit_station);
    }

    #[test]
    fn test_should_get_user() {
        init_canister();

        // setup user
        let principal = msg_caller();
        UserStorage::add_user(
            principal,
            User {
                username: "test_user".to_string(),
                public_key: vec![1; 32].try_into().unwrap(),
            },
        );

        // get user
        let response = Canister::get_user(principal);
        assert_eq!(
            response,
            Some(PublicUser {
                username: "test_user".to_string(),
                public_key: vec![1; 32].try_into().unwrap(),
                ic_principal: principal,
            })
        );

        // get user with a different principal
        let principal = Principal::from_slice(&[1; 6]);
        let response = Canister::get_user(principal);
        assert!(response.is_none());
    }

    #[test]
    fn test_should_get_users() {
        init_canister();

        // setup user
        let principal = msg_caller();
        UserStorage::add_user(
            principal,
            User {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            },
        );

        // get users
        let response = Canister::get_users(
            Pagination {
                offset: 0,
                limit: 10,
            },
            None,
        );
        assert_eq!(
            response,
            GetUsersResponse::Users(GetUsersResponseUsers {
                users: vec![PublicUser {
                    username: "test_user".to_string(),
                    public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
                    ic_principal: principal,
                }],
                total: 1,
                next: None,
            })
        );
    }

    #[test]
    fn test_should_get_users_with_query() {
        init_canister();

        // setup users
        for i in 0..150 {
            UserStorage::add_user(
                Principal::from_slice(&[i; 6]),
                User {
                    username: format!("test_user_{i}",),
                    public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
                },
            );
        }

        // get users
        let response = Canister::get_users(
            Pagination {
                offset: 0,
                limit: 20,
            },
            Some("test_user_12"),
        );

        // there should be eleven users (12, 120, 121, ..., 129)
        let GetUsersResponse::Users(GetUsersResponseUsers { users, total, next }) = response else {
            panic!("Expected GetUsersResponse::Users");
        };

        assert_eq!(users.len(), 11);
        assert_eq!(total, 11);
        assert!(next.is_none());

        // with pagination
        let response = Canister::get_users(
            Pagination {
                offset: 0,
                limit: 5,
            },
            Some("test_user_12"),
        );

        // there should be eleven users (12, 120, 121, ..., 129)
        let GetUsersResponse::Users(GetUsersResponseUsers { users, total, next }) = response else {
            panic!("Expected GetUsersResponse::Users");
        };

        assert_eq!(users.len(), 5);
        assert_eq!(total, 11);
        assert_eq!(next, Some(5));
    }

    #[test]
    fn test_should_not_get_users_with_invalid_query() {
        init_canister();

        // get users
        let response = Canister::get_users(
            Pagination {
                offset: 0,
                limit: 20,
            },
            Some("aa"),
        );
        assert_eq!(response, GetUsersResponse::InvalidQuery);
    }

    #[test]
    fn test_should_get_paginated_users() {
        init_canister();

        // setup users
        for i in 0..9 {
            UserStorage::add_user(
                Principal::from_slice(&[i; 6]),
                User {
                    username: format!("test_user_{i}",),
                    public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
                },
            );
        }

        // get users
        let response = Canister::get_users(
            Pagination {
                offset: 0,
                limit: 5,
            },
            None,
        );

        let GetUsersResponse::Users(GetUsersResponseUsers { total, users, next }) = response else {
            panic!("Expected GetUsersResponse::Users");
        };
        assert_eq!(users.len(), 5);
        assert_eq!(total, 9);
        assert_eq!(next, Some(5));

        let response = Canister::get_users(
            Pagination {
                offset: 5,
                limit: 8,
            },
            None,
        );

        let GetUsersResponse::Users(GetUsersResponseUsers { total, users, next }) = response else {
            panic!("Expected GetUsersResponse::Users");
        };
        assert_eq!(total, 9);
        assert_eq!(users.len(), 4);
        assert!(next.is_none());
    }

    #[test]
    fn test_should_get_capped_paginated_users() {
        init_canister();

        // setup users
        for i in 0..150 {
            UserStorage::add_user(
                Principal::from_slice(&[i; 6]),
                User {
                    username: format!("test_user_{i}",),
                    public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
                },
            );
        }

        // get users
        let response = Canister::get_users(
            Pagination {
                offset: 0,
                limit: 150,
            },
            None,
        );

        let GetUsersResponse::Users(GetUsersResponseUsers { total, users, next }) = response else {
            panic!("Expected GetUsersResponse::Users");
        };
        assert_eq!(users.len() as u64, MAX_GET_USERS_LIMIT);
        assert_eq!(total, 150);
        assert_eq!(next, Some(MAX_GET_USERS_LIMIT));
    }

    #[test]
    fn test_should_retry_user_canister_creation() {
        init_canister();

        // let's setup a user
        let principal = msg_caller();
        UserStorage::add_user(
            principal,
            User {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            },
        );

        // of course this won't start the state machine on test unit; let's set the state to failed
        UserCanisterStorage::set_create_state(
            principal,
            UserCanisterCreateState::Failed {
                reason: "test".to_string(),
            },
        );

        // we can retry now :D
        let response = Canister::retry_user_canister_creation();
        assert_eq!(response, RetryUserCanisterCreationResponse::Ok);
    }

    #[test]
    fn test_should_not_retry_user_canister_creation_if_user_does_not_exist() {
        init_canister();

        // let's setup another user
        UserStorage::add_user(
            Principal::management_canister(),
            User {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            },
        );

        // user does not exist
        let response = Canister::retry_user_canister_creation();
        assert_eq!(response, RetryUserCanisterCreationResponse::UserNotFound);
    }

    #[test]
    fn test_should_not_retry_if_user_canister_exists() {
        init_canister();

        // let's setup a user
        let principal = msg_caller();
        UserStorage::add_user(
            principal,
            User {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            },
        );

        // let's set the user canister
        let user_canister = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();
        UserCanisterStorage::set_user_canister(principal, user_canister);

        // canister already exists
        let response = Canister::retry_user_canister_creation();
        assert_eq!(
            response,
            RetryUserCanisterCreationResponse::Created(user_canister)
        );
    }

    #[test]
    fn test_should_not_retry_if_pending() {
        init_canister();

        // let's setup a user
        let principal = msg_caller();
        UserStorage::add_user(
            principal,
            User {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            },
        );

        // let's set the user canister creation state to something pending
        UserCanisterStorage::set_create_state(principal, UserCanisterCreateState::CreateCanister);

        // canister already exists
        let response = Canister::retry_user_canister_creation();
        assert_eq!(response, RetryUserCanisterCreationResponse::CreationPending);
    }

    #[test]
    fn test_should_register_user_if_valid() {
        init_canister();

        // setup user
        let principal = msg_caller();
        let username = "test_user".to_string();
        let public_key = PublicKey::try_from(vec![1; 32]).expect("invalid public key");

        // register user
        let response = Canister::set_user(username.clone(), public_key);
        assert_eq!(response, SetUserResponse::Ok);

        // check if user exists
        let user = UserStorage::get_user(&principal).unwrap();
        assert_eq!(user.username, username);
        assert_eq!(user.public_key, public_key);
    }

    #[test]
    fn test_should_not_add_user_if_username_too_long() {
        init_canister();

        // setup user
        let principal = msg_caller();
        let username = "a".repeat(MAX_USERNAME_SIZE + 1);
        let public_key = PublicKey::try_from(vec![1; 32]).expect("invalid public key");

        // register user
        let response = Canister::set_user(username.clone(), public_key);
        assert_eq!(response, SetUserResponse::UsernameTooLong);

        // check if user does not exist
        let user = UserStorage::get_user(&principal);
        assert!(user.is_none());
    }

    #[test]
    fn test_should_not_add_user_if_caller_has_already_a_user() {
        init_canister();

        // setup user
        let username = "test_user".to_string();
        let public_key = PublicKey::try_from(vec![1; 32]).expect("invalid public key");

        // register user
        let response = Canister::set_user(username.clone(), public_key);
        assert_eq!(response, SetUserResponse::Ok);

        // try another username
        let response = Canister::set_user("foo".to_string(), public_key);
        assert_eq!(response, SetUserResponse::CallerHasAlreadyAUser);
    }

    #[test]
    fn test_should_tell_if_username_exists() {
        init_canister();

        // setup user
        let principal = msg_caller();
        UserStorage::add_user(
            principal,
            User {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            },
        );

        // check if username exists
        let exists = Canister::username_exists("test_user".to_string());
        assert!(exists);

        // check if non-existing username exists
        let exists = Canister::username_exists("non_existing_user".to_string());
        assert!(!exists);
    }

    #[test]
    fn test_should_tell_whoami() {
        init_canister();

        // setup user
        let principal = msg_caller();
        UserStorage::add_user(
            principal,
            User {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            },
        );

        // get whoami
        let whoami = Canister::whoami();
        assert_eq!(
            whoami,
            WhoamiResponse::KnownUser(PublicUser {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
                ic_principal: principal,
            })
        );
    }

    #[test]
    fn test_should_return_shared_files() {
        init_canister();

        // setup user
        let principal = msg_caller();
        UserStorage::add_user(
            principal,
            User {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            },
        );

        // insert shared files
        let file_id = 1;
        let user_canister = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

        SharedFilesStorage::share_file(
            principal,
            user_canister,
            file_id,
            ShareFileMetadata {
                file_name: "foo.txt".to_string(),
            },
        );

        let public_user = PublicUser {
            username: "test_user".to_string(),
            public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            ic_principal: principal,
        };

        let mut expected = HashMap::new();
        expected.insert(
            user_canister,
            vec![PublicFileMetadata {
                file_id,
                file_name: "foo.txt".to_string(),
                shared_with: vec![public_user],
            }]
            .into_iter()
            .collect(),
        );

        // get shared files
        let shared_files = Canister::shared_files();
        assert_eq!(shared_files, SharedFilesResponse::SharedFiles(expected));
    }

    #[test]
    fn test_should_return_error_on_shared_files_unexisting_user() {
        init_canister();

        // get shared files
        let shared_files = Canister::shared_files();
        assert_eq!(shared_files, SharedFilesResponse::NoSuchUser);
    }

    #[test]
    fn test_should_revoke_shared_file() {
        init_canister();

        // insert user canister
        let user_canister = msg_caller();
        let user = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();
        UserCanisterStorage::set_user_canister(user, user_canister);

        // revoke share
        let file_id = 1;
        SharedFilesStorage::share_file(
            user,
            user_canister,
            file_id,
            ShareFileMetadata {
                file_name: "foo.txt".to_string(),
            },
        );
        let response = Canister::revoke_share_file(user, file_id);
        assert_eq!(response, RevokeShareFileResponse::Ok);

        // check if the file is revoked
        let shared_files = SharedFilesStorage::get_shared_files(user);
        assert_eq!(shared_files.len(), 0);
    }

    #[test]
    fn test_should_not_revoke_shared_file_if_caller_is_not_a_user_canister() {
        init_canister();

        // insert user canister
        let user_canister = msg_caller();
        let user = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

        // revoke share
        let file_id = 1;
        SharedFilesStorage::share_file(
            user,
            user_canister,
            file_id,
            ShareFileMetadata {
                file_name: "foo.txt".to_string(),
            },
        );
        let response = Canister::revoke_share_file(user, file_id);
        assert_eq!(response, RevokeShareFileResponse::Unauthorized);

        // check if the file is NOT revoked
        let shared_files = SharedFilesStorage::get_shared_files(user);
        assert_eq!(shared_files.len(), 1);
    }

    #[test]
    fn test_should_revoke_shared_file_with_users() {
        init_canister();

        // insert user canister
        let user_canister = msg_caller();
        let user = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();
        let user_2 = Principal::from_slice(&[2; 6]);
        let user_3 = Principal::from_slice(&[3; 6]);
        UserCanisterStorage::set_user_canister(user, user_canister);
        UserCanisterStorage::set_user_canister(user_2, user_canister);
        UserCanisterStorage::set_user_canister(user_3, user_canister);

        // revoke share
        let file_id = 1;
        SharedFilesStorage::share_file(
            user,
            user_canister,
            file_id,
            ShareFileMetadata {
                file_name: "foo.txt".to_string(),
            },
        );
        SharedFilesStorage::share_file(
            user_2,
            user_canister,
            file_id,
            ShareFileMetadata {
                file_name: "foo.txt".to_string(),
            },
        );
        SharedFilesStorage::share_file(
            user_3,
            user_canister,
            file_id,
            ShareFileMetadata {
                file_name: "foo.txt".to_string(),
            },
        );
        let response = Canister::revoke_share_file_for_users(vec![user, user_2], file_id);
        assert_eq!(response, RevokeShareFileResponse::Ok);

        // check if the file is revoked
        let shared_files = SharedFilesStorage::get_shared_files(user);
        assert_eq!(shared_files.len(), 0);
        let shared_files = SharedFilesStorage::get_shared_files(user_2);
        assert_eq!(shared_files.len(), 0);
        let shared_files = SharedFilesStorage::get_shared_files(user_3);
        assert_eq!(shared_files.len(), 1);
    }

    #[test]
    fn test_should_share_file() {
        init_canister();

        // insert user canister
        let user_canister = msg_caller();
        let user = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

        // set user canister
        UserCanisterStorage::set_user_canister(user, user_canister);

        // create user
        let alice = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();
        UserStorage::add_user(
            alice,
            User {
                username: "test_user".to_string(),
                public_key: PublicKey::try_from(vec![1; 32]).expect("invalid public key"),
            },
        );

        // share file
        let file_id = 1;
        let response = Canister::share_file(
            alice,
            file_id,
            ShareFileMetadata {
                file_name: "foo.txt".to_string(),
            },
        );
        assert_eq!(response, ShareFileResponse::Ok);

        // check if the file is shared
        let shared_files = SharedFilesStorage::get_shared_files(alice);
        assert_eq!(shared_files.len(), 1);
    }

    #[test]
    fn test_should_not_share_if_not_called_by_user_canister() {
        init_canister();

        let user = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

        // share file
        let file_id = 1;
        let response = Canister::share_file(
            user,
            file_id,
            ShareFileMetadata {
                file_name: "foo.txt".to_string(),
            },
        );
        assert_eq!(response, ShareFileResponse::Unauthorized);

        // check if the file is NOT shared
        let shared_files = SharedFilesStorage::get_shared_files(user);
        assert_eq!(shared_files.len(), 0);
    }

    #[test]
    fn test_should_not_share_if_user_does_not_exist() {
        init_canister();

        // insert user canister
        let user_canister = msg_caller();
        let user = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

        // set user canister
        UserCanisterStorage::set_user_canister(user, user_canister);

        let alice = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

        // share file
        let file_id = 1;
        let response = Canister::share_file(
            alice,
            file_id,
            ShareFileMetadata {
                file_name: "foo.txt".to_string(),
            },
        );
        assert_eq!(response, ShareFileResponse::NoSuchUser(alice));

        // check if the file is NOT shared
        let shared_files = SharedFilesStorage::get_shared_files(alice);
        assert_eq!(shared_files.len(), 0);
    }

    fn init_canister() {
        let orbit_station = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();
        Canister::init(OrchestratorInstallArgs::Init(OrchestratorInitArgs {
            orbit_station,
            orbit_station_admin: "admin".to_string(),
        }));
    }
}
