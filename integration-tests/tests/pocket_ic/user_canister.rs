use candid::Principal;
use did::user_canister::{
    ENCRYPTION_KEY_SIZE, FileStatus, UploadFileAtomicRequest, UploadFileContinueRequest,
    UploadFileRequest,
};
use integration_tests::actor::{admin, alice};
use integration_tests::{PocketIcTestEnv, UserCanisterClient};

#[tokio::test]
async fn test_should_set_and_get_public_key() {
    let env = PocketIcTestEnv::init().await;
    let client = UserCanisterClient::from(&env);
    let me = Principal::from_slice(&[1; 29]);

    let new_public_key = [1; 32];
    // set public key (only owner_can set it)
    client.set_public_key(new_public_key).await;
    // get public key
    let public_key = client.public_key(me).await;

    assert_eq!(new_public_key, public_key);

    env.stop().await;
}

#[tokio::test]
async fn test_should_request_file_and_get_requests() {
    let env = PocketIcTestEnv::init().await;
    let client = UserCanisterClient::from(&env);
    let owner = admin();

    let request_name = "test.txt".to_string();
    client.request_file(request_name.clone(), owner).await;

    assert_eq!(
        client.get_requests(owner).await.first().unwrap().file_name,
        request_name
    );

    env.stop().await;
}
#[tokio::test]
async fn test_should_upload_file() {
    let env = PocketIcTestEnv::init().await;
    let client = UserCanisterClient::from(&env);
    let owner = admin();
    let request_name = "test.txt".to_string();
    client.request_file(request_name.clone(), owner).await;
    let r = client
        .upload_file(
            UploadFileRequest {
                file_id: 1,
                file_content: vec![1, 2, 3],
                file_type: "txt".to_string(),
                owner_key: [1; 32],
                num_chunks: 1,
            },
            owner,
        )
        .await;
    assert!(r.is_ok());
    let public_metadata = client.get_requests(owner).await.first().unwrap().clone();

    match public_metadata.file_status {
        FileStatus::Uploaded { document_key, .. } => {
            assert_eq!(document_key, [1; ENCRYPTION_KEY_SIZE]);
        }
        _ => panic!("File status is not uploaded"),
    }

    env.stop().await;
}

#[tokio::test]
async fn test_should_get_alias_info() {
    let env = PocketIcTestEnv::init().await;
    let client = UserCanisterClient::from(&env);
    let owner = admin();
    let external_user = alice();
    let request_name = "test.txt".to_string();
    let alias = client.request_file(request_name.clone(), owner).await;
    let alias_info = client.get_alias_info(alias.clone(), external_user).await;

    assert_eq!(
        client
            .get_alias_info("not-an-alias".to_string(), external_user)
            .await,
        Err(did::user_canister::GetAliasInfoError::NotFound)
    );
    assert_eq!(alias_info.clone().unwrap().file_name, request_name);
    assert_eq!(alias_info.unwrap().file_id, 1);

    env.stop().await;
}

#[tokio::test]
async fn test_should_upload_file_atomic() {
    let env = PocketIcTestEnv::init().await;
    let client = UserCanisterClient::from(&env);
    let owner = admin();
    let request_name = "test.txt".to_string();
    let file_id = client
        .upload_file_atomic(
            UploadFileAtomicRequest {
                name: request_name.clone(),
                content: vec![1, 2, 3],
                file_type: "txt".to_string(),
                owner_key: [1; ENCRYPTION_KEY_SIZE],
                num_chunks: 1,
            },
            owner,
        )
        .await;
    assert_eq!(file_id, 1);
    let public_metadata = client.get_requests(owner).await.first().unwrap().clone();

    match public_metadata.file_status {
        FileStatus::Uploaded { document_key, .. } => {
            assert_eq!(document_key, [1; ENCRYPTION_KEY_SIZE]);
        }
        _ => panic!("File status is not uploaded"),
    }
    assert_eq!(public_metadata.file_id, 1);
    env.stop().await;
}

#[tokio::test]
async fn test_should_upload_file_continue() {
    let env = PocketIcTestEnv::init().await;
    let client = UserCanisterClient::from(&env);
    let owner = admin();
    let request_name = "test.txt".to_string();

    let file_id = client
        .upload_file_atomic(
            UploadFileAtomicRequest {
                name: request_name.clone(),
                content: vec![1, 2, 3],
                file_type: "txt".to_string(),
                owner_key: [1; ENCRYPTION_KEY_SIZE],
                num_chunks: 3,
            },
            owner,
        )
        .await;
    assert_eq!(file_id, 1);
    client
        .upload_file_continue(
            UploadFileContinueRequest {
                file_id,
                chunk_id: 1,
                contents: vec![4, 5, 6],
            },
            owner,
        )
        .await;

    let public_metadata = client.get_requests(owner).await.first().unwrap().clone();
    match public_metadata.file_status {
        FileStatus::PartiallyUploaded => {
            assert_eq!(public_metadata.file_id, 1);
        }
        _ => panic!("File status is not partially uploaded"),
    }

    client
        .upload_file_continue(
            UploadFileContinueRequest {
                file_id,
                chunk_id: 2,
                contents: vec![7, 8, 9],
            },
            owner,
        )
        .await;
    let public_metadata = client.get_requests(owner).await.first().unwrap().clone();
    match public_metadata.file_status {
        FileStatus::Uploaded { document_key, .. } => {
            assert_eq!(document_key, [1; ENCRYPTION_KEY_SIZE]);
        }
        _ => panic!("File status is not uploaded"),
    }

    env.stop().await;
}

#[tokio::test]
async fn test_should_download_file() {
    let env = PocketIcTestEnv::init().await;
    let client = UserCanisterClient::from(&env);
    let owner = admin();
    let request_name = "test.txt".to_string();

    let file_id = client
        .upload_file_atomic(
            UploadFileAtomicRequest {
                name: request_name.clone(),
                content: vec![1, 2, 3],
                file_type: "txt".to_string(),
                owner_key: [1; ENCRYPTION_KEY_SIZE],
                num_chunks: 3,
            },
            owner,
        )
        .await;
    assert_eq!(file_id, 1);
    client
        .upload_file_continue(
            UploadFileContinueRequest {
                file_id,
                chunk_id: 1,
                contents: vec![4, 5, 6],
            },
            owner,
        )
        .await;

    let download_response = client.download_file(file_id, 2, owner).await;
    assert_eq!(
        download_response,
        did::user_canister::FileDownloadResponse::NotUploadedFile
    );
    client
        .upload_file_continue(
            UploadFileContinueRequest {
                file_id,
                chunk_id: 2,
                contents: vec![7, 8, 9],
            },
            owner,
        )
        .await;
    let download_response = client.download_file(file_id, 2, owner).await;

    match download_response {
        did::user_canister::FileDownloadResponse::FoundFile(file_data) => {
            assert_eq!(file_data.contents, vec![7, 8, 9]);
            assert_eq!(file_data.file_type, "txt");
            assert_eq!(file_data.owner_key, [1; ENCRYPTION_KEY_SIZE]);
            assert_eq!(file_data.num_chunks, 3);
        }
        _ => panic!("File not found"),
    }

    env.stop().await;
}
// #[tokio::test]
// async fn test_should_get_shared_files() {
//     let env = PocketIcTestEnv::init().await;
//     let client = BackendClient::from(&env);
//     let external_user = alice();
//     let owner = admin();
//     let request_name = "test.txt".to_string();
//     client.request_file(request_name.clone(), owner).await;
//     client
//         .upload_file(
//             UploadFileRequest {
//                 file_id: 1,
//                 file_content: vec![1, 2, 3],
//                 file_type: "txt".to_string(),
//                 owner_key: [1; 32],
//                 num_chunks: 1,
//             },
//             external_user,
//         )
//         .await;

//     assert_eq!(
//         client
//             .get_shared_files(owner)
//             .await.get(0).unwrap().file_name,

//         request_name

//     );

//     env.stop().await;

// }
