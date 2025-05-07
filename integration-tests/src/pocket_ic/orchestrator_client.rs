use candid::Principal;
use did::orchestrator::{GetUsersResponse, PublicKey, SetUserResponse, WhoamiResponse};

use super::PocketIcTestEnv;
use crate::TestEnv as _;
use crate::actor::admin;

pub struct OrchestratorClient<'a> {
    pic: &'a PocketIcTestEnv,
}

impl<'a> From<&'a PocketIcTestEnv> for OrchestratorClient<'a> {
    fn from(pic: &'a PocketIcTestEnv) -> Self {
        Self { pic }
    }
}

impl OrchestratorClient<'_> {
    pub async fn orchestrator_client(&self) -> Principal {
        self.pic
            .query::<Principal>(self.pic.orchestrator(), admin(), "orbit_station", vec![])
            .await
            .expect("Failed to get orbit station")
    }

    pub async fn get_users(&self, caller: Principal) -> GetUsersResponse {
        let payload = candid::encode_args(()).unwrap();
        self.pic
            .query::<GetUsersResponse>(self.pic.orchestrator(), caller, "get_users", payload)
            .await
            .expect("Failed to get users")
    }

    pub async fn set_user(
        &self,
        caller: Principal,
        username: String,
        public_key: PublicKey,
    ) -> SetUserResponse {
        let payload = candid::encode_args((username, public_key)).unwrap();
        self.pic
            .update::<SetUserResponse>(self.pic.orchestrator(), caller, "set_user", payload)
            .await
            .expect("Failed to set user")
    }

    pub async fn who_am_i(&self, caller: Principal) -> WhoamiResponse {
        let payload = candid::encode_args(()).unwrap();
        self.pic
            .query::<WhoamiResponse>(self.pic.orchestrator(), caller, "who_am_i", payload)
            .await
            .expect("Failed to get who am i")
    }

    pub async fn username_exists(&self, username: String) -> bool {
        let payload = candid::encode_args((username,)).unwrap();
        self.pic
            .query::<bool>(self.pic.orchestrator(), admin(), "username_exists", payload)
            .await
            .expect("Failed to check if username exists")
    }
}
