use candid::Principal;
use integration_tests::actor::admin;
use integration_tests::{PocketIcTestEnv, TestEnv};

#[tokio::test]
async fn test_should_get_orbit_station() {
    let env = PocketIcTestEnv::init().await;

    let orbit_station = env
        .query::<Principal>(env.orchestrator(), admin(), "orbit_station", vec![])
        .await
        .expect("Failed to get orbit station");

    assert_eq!(orbit_station, env.orbit_station());
}
