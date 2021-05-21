use crate::tools::UserError;
use ring::{digest, pbkdf2};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
pub type Credential = [u8; CREDENTIAL_LEN];
static SALT_COMPONENT: [u8; 16] = [
    0xd6, 0x26, 0x98, 0xda, 0xf4, 0xdc, 0x50, 0x52, 0x24, 0xf2, 0x27, 0xd1, 0xfe, 0x39, 0x01, 0x8a,
];
const PBKDF2_ITER: u32 = 100_000;

#[derive(Deserialize)]
pub struct UserReq {
    username: String,
    password: String,
}

impl UserReq {
    pub fn get_username(&self) -> String {
        self.username.clone()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    username: String,
    credential: Credential,
}

impl User {
    pub fn login(&self, user: &UserReq) -> Result<(), UserError> {
        let salt = Self::salt(&user.username);
        let iter = NonZeroU32::new(PBKDF2_ITER).unwrap();
        pbkdf2::verify(
            PBKDF2_ALG,
            iter,
            &salt,
            user.password.as_bytes(),
            &self.credential,
        )
        .map_err(|_| UserError::MismatchingCredential)?;

        Ok(())
    }

    fn salt(username: &str) -> Vec<u8> {
        let mut salt = Vec::with_capacity(SALT_COMPONENT.len() + username.as_bytes().len());
        salt.extend(SALT_COMPONENT.as_ref());
        salt.extend(username.as_bytes());
        salt
    }

    pub fn new(req: UserReq) -> Self {
        let salt = Self::salt(&req.username);
        let iter = NonZeroU32::new(PBKDF2_ITER).unwrap();
        let mut cred: Credential = [0u8; CREDENTIAL_LEN];
        pbkdf2::derive(PBKDF2_ALG, iter, &salt, req.password.as_bytes(), &mut cred);
        Self {
            username: req.username,
            credential: cred,
        }
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }
}
