// Copyright 2023 Simo Sorce
// See LICENSE.txt file for terms

use super::drbg;
use super::err_rv;
use super::error;
use super::interface;
use super::mechanism;

use error::{KError, KResult};
use interface::*;

#[derive(Debug)]
pub struct RNG {
    drbg: Box<dyn mechanism::DRBG>,
}

impl RNG {
    pub fn new(alg: &str) -> KResult<RNG> {
        match alg {
            "HMAC DRBG SHA256" => Ok(RNG {
                drbg: Box::new(drbg::HmacSha256Drbg::new()?),
            }),
            "HMAC DRBG SHA512" => Ok(RNG {
                drbg: Box::new(drbg::HmacSha512Drbg::new()?),
            }),
            _ => err_rv!(CKR_RANDOM_NO_RNG),
        }
    }

    pub fn generate_random(&mut self, buffer: &mut [u8]) -> KResult<()> {
        let noaddtl: [u8; 0] = [];
        self.drbg.generate(&noaddtl, buffer)
    }
}
