use candid::Principal;
use did::orchestrator::{
    GetUsersResponse, Pagination, PublicKey, PublicUser, SetUserResponse, SharedFilesResponse,
    WhoamiResponse,
};
use did::user_canister::{FileSharingResponse, OwnerKey, UploadFileAtomicRequest};
use integration_tests::actor::{admin, alice};
use integration_tests::{OrchestratorClient, TestEnv, UserCanisterClient};

#[pocket_test::test]
async fn test_should_get_orbit_station(ctx: PocketIcTestEnv) {
    let orbit_station = OrchestratorClient::from(&ctx).orchestrator_client().await;

    assert_eq!(orbit_station, ctx.orbit_station());
}

#[pocket_test::test]
async fn test_should_register_user(env: PocketIcTestEnv) {
    let client = OrchestratorClient::from(&env);

    let me = Principal::from_slice(&[1; 29]);

    let username = "foo".to_string();
    let public_key = PublicKey::default();

    // we check if username is available
    assert!(!client.username_exists(username.clone()).await,);

    // register
    let response = client.set_user(me, username.clone(), public_key).await;
    assert_eq!(response, SetUserResponse::Ok);

    // check if username exists
    assert!(client.username_exists(username.clone()).await);

    // who am i
    let whoami = client.who_am_i(me).await;
    assert_eq!(
        whoami,
        WhoamiResponse::KnownUser(PublicUser {
            username,
            public_key,
            ic_principal: me,
        })
    );
}

#[pocket_test::test]
async fn test_should_not_register_user_if_anonymous(env: PocketIcTestEnv) {
    let client = OrchestratorClient::from(&env);

    let username = "foo".to_string();
    let public_key = PublicKey::default();
    let response = client
        .set_user(Principal::anonymous(), username, public_key)
        .await;
    assert_eq!(response, SetUserResponse::AnonymousCaller);
}

#[pocket_test::test]
async fn test_should_not_get_users_if_anonymous(env: PocketIcTestEnv) {
    let client = OrchestratorClient::from(&env);

    let users = client
        .get_users(
            Principal::anonymous(),
            Pagination {
                offset: 0,
                limit: 10,
            },
        )
        .await;
    assert_eq!(users, GetUsersResponse::PermissionError);
}

#[pocket_test::test]
async fn test_should_create_user_canister(env: PocketIcTestEnv) {
    let client = OrchestratorClient::from(&env);

    let me = alice();
    let username = "alice".to_string();
    let public_key = PublicKey::default();

    // create user canister
    let response = client.set_user(me, username, public_key).await;
    assert_eq!(response, SetUserResponse::Ok);

    // wait for user canister to be created
    let user_canister = client.wait_for_user_canister(me).await;
    assert_ne!(user_canister, Principal::anonymous());
}

#[pocket_test::test]
async fn test_should_not_return_shared_files_if_anonymous(env: PocketIcTestEnv) {
    let client = OrchestratorClient::from(&env);

    let shared_files = client.shared_files(Principal::anonymous()).await;
    assert_eq!(shared_files, SharedFilesResponse::AnonymousUser);
}

#[pocket_test::test]
async fn test_should_return_shared_files(env: PocketIcTestEnv) {
    let orchestrator_client = OrchestratorClient::from(&env);
    let owner = admin();
    let shared_with = alice();

    // register alice on orchestrator
    let response = orchestrator_client
        .set_user(shared_with, "alice".to_string(), PublicKey::default())
        .await;
    assert_eq!(response, SetUserResponse::Ok);

    // admin creates a file and shares it with alice
    let user_canister_client = UserCanisterClient::from(&env);
    let request_name = "test.txt".to_string();
    let file_id = user_canister_client
        .upload_file_atomic(
            UploadFileAtomicRequest {
                name: request_name.clone(),
                content: vec![1, 2, 3],
                file_type: "txt".to_string(),
                owner_key: [1; OwnerKey::KEY_SIZE].into(),
                num_chunks: 1,
            },
            owner,
        )
        .await;

    // share file with alice
    assert_eq!(
        user_canister_client
            .share_file(owner, file_id, shared_with, [1; OwnerKey::KEY_SIZE].into())
            .await,
        FileSharingResponse::Ok
    );

    let shared_files = orchestrator_client.shared_files(shared_with).await;
    let SharedFilesResponse::SharedFiles(files) = shared_files else {
        panic!("Expected SharedFiles, got: {:?}", shared_files);
    };

    assert_eq!(files.len(), 1);
    let shared_file_on_owner_canister = files
        .get(&env.user_canister())
        .expect("Expected file on owner canister");
    assert_eq!(shared_file_on_owner_canister.len(), 1);
    assert!(
        shared_file_on_owner_canister
            .iter()
            .any(|it| it.file_id == file_id)
    );
}
