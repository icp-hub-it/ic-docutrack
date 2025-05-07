use candid::CandidType;
use serde::{Deserialize, Serialize};

use super::PublicUser;

#[derive(CandidType, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum WhoamiResponse {
    #[serde(rename = "known_user")]
    KnownUser(PublicUser),
    #[serde(rename = "unknown_user")]
    UnknownUser,
}

impl From<PublicUser> for WhoamiResponse {
    fn from(user: PublicUser) -> Self {
        WhoamiResponse::KnownUser(user)
    }
}

#[cfg(test)]
mod test {
    use candid::Principal;

    use super::*;

    #[test]
    fn test_should_create_whoami_response_from_public_user() {
        let user = PublicUser {
            username: "test_user".to_string(),
            public_key: [1; 32],
            ic_principal: Principal::from_slice(&[2; 29]),
        };

        let response = WhoamiResponse::from(user.clone());

        assert_eq!(response, WhoamiResponse::KnownUser(user));
    }
}
