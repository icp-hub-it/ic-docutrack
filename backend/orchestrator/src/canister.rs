mod create_user;

use candid::Principal;
use create_user::CreateUserStateMachine;
use did::orchestrator::{
    GetUsersResponse, MAX_USERNAME_SIZE, OrchestratorInitArgs, PublicKey, PublicUser,
    SetUserResponse, User, UserCanisterResponse, WhoamiResponse,
};

use crate::storage::config::Config;
use crate::storage::user_canister::{UserCanisterCreateState, UserCanisterStorage};
use crate::storage::users::UserStorage;
use crate::utils::msg_caller;

/// API for Business Logic
pub struct Canister;

impl Canister {
    /// Initialize the canister with the given arguments.
    pub fn init(args: OrchestratorInitArgs) {
        Config::set_orbit_station(args.orbit_station);
        Config::set_orbit_station_admin(args.orbit_station_admin);
    }

    /// Get the users from the storage as [`GetUsersResponse`].
    ///
    /// If the caller is anonymous, it returns [`GetUsersResponse::PermissionError`].
    ///
    /// FIXME: this function is going to exhaust memory when called if we don't introduce pagination.
    /// There is already a task for it in the backlog.
    /// FIXME: this function should be protected.
    pub fn get_users() -> GetUsersResponse {
        let caller = msg_caller();
        if caller == Principal::anonymous() {
            return GetUsersResponse::PermissionError;
        }

        UserStorage::get_users()
            .into_iter()
            .map(|(principal, user)| PublicUser::new(user, principal))
            .collect::<Vec<_>>()
            .into()
    }

    /// Set a new user in the storage.
    pub fn set_user(username: String, public_key: PublicKey) -> SetUserResponse {
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

    /// Checks whether a given username exists in the storage.
    pub fn username_exists(username: String) -> bool {
        UserStorage::username_exists(&username)
    }

    /// Get user canister information for the current caller.
    ///
    /// Returns [`UserCanisterResponse::AnonymousCaller`] if the caller is anonymous.
    /// Returns [`UserCanisterResponse::Ok`] if the user canister is created and ready to use.
    /// Returns [`UserCanisterResponse::CreationPending`] if the user canister is being created.
    /// Returns [`UserCanisterResponse::CreationFailed`] if the user canister creation failed.
    pub fn user_canister() -> UserCanisterResponse {
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
    pub fn whoami() -> WhoamiResponse {
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

    use did::orchestrator::User;

    use super::*;

    #[test]
    fn test_should_init_canister() {
        let orbit_station = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();
        Canister::init(OrchestratorInitArgs {
            orbit_station,
            orbit_station_admin: "admin".to_string(),
        });

        assert_eq!(Config::get_orbit_station(), orbit_station);
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
                public_key: [1; 32],
            },
        );

        // get users
        let response = Canister::get_users();
        assert_eq!(
            response,
            GetUsersResponse::Users(vec![PublicUser {
                username: "test_user".to_string(),
                public_key: [1; 32],
                ic_principal: principal,
            }])
        );
    }

    #[test]
    fn test_should_register_user_if_valid() {
        init_canister();

        // setup user
        let principal = msg_caller();
        let username = "test_user".to_string();
        let public_key = [1; 32];

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
        let public_key = [1; 32];

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
        let public_key = [1; 32];

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
                public_key: [1; 32],
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
                public_key: [1; 32],
            },
        );

        // get whoami
        let whoami = Canister::whoami();
        assert_eq!(
            whoami,
            WhoamiResponse::KnownUser(PublicUser {
                username: "test_user".to_string(),
                public_key: [1; 32],
                ic_principal: principal,
            })
        );
    }

    fn init_canister() {
        let orbit_station = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();
        Canister::init(OrchestratorInitArgs {
            orbit_station,
            orbit_station_admin: "admin".to_string(),
        });
    }
}
