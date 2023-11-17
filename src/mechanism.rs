// Copyright 2023 Simo Sorce
// See LICENSE.txt file for terms

use std::collections::BTreeMap;

use super::err_rv;
use super::error;
use super::interface;
use super::object;
use super::rng;
use error::{KError, KResult};
use interface::*;
use object::Object;
use rng::RNG;

use std::fmt::Debug;

pub trait Mechanism: Debug + Send + Sync {
    fn info(&self) -> &CK_MECHANISM_INFO;
    fn encryption_new(
        &self,
        _: &CK_MECHANISM,
        _: &object::Object,
    ) -> KResult<Box<dyn Encryption>> {
        err_rv!(CKR_MECHANISM_INVALID)
    }
    fn decryption_new(
        &self,
        _: &CK_MECHANISM,
        _: &object::Object,
    ) -> KResult<Box<dyn Decryption>> {
        err_rv!(CKR_MECHANISM_INVALID)
    }
    fn digest_new(&self, _: &CK_MECHANISM) -> KResult<Box<dyn Digest>> {
        err_rv!(CKR_MECHANISM_INVALID)
    }
    fn sign_new(
        &self,
        _: &CK_MECHANISM,
        _: &object::Object,
    ) -> KResult<Box<dyn Sign>> {
        err_rv!(CKR_MECHANISM_INVALID)
    }
    fn verify_new(
        &self,
        _: &CK_MECHANISM,
        _: &object::Object,
    ) -> KResult<Box<dyn Verify>> {
        err_rv!(CKR_MECHANISM_INVALID)
    }

    fn generate_key(
        &self,
        _: &mut rng::RNG,
        _: &CK_MECHANISM,
        _: &[CK_ATTRIBUTE],
    ) -> KResult<Object> {
        err_rv!(CKR_MECHANISM_INVALID)
    }

    fn generate_keypair(
        &self,
        _: &mut rng::RNG,
        _: &CK_MECHANISM,
        _pubkey_template: &[CK_ATTRIBUTE],
        _prikey_template: &[CK_ATTRIBUTE],
    ) -> KResult<(Object, Object)> {
        err_rv!(CKR_MECHANISM_INVALID)
    }
}

#[derive(Debug)]
pub struct Mechanisms {
    tree: BTreeMap<CK_MECHANISM_TYPE, Box<dyn Mechanism>>,
}

impl Mechanisms {
    pub fn new() -> Mechanisms {
        Mechanisms {
            tree: BTreeMap::new(),
        }
    }

    pub fn add_mechanism(
        &mut self,
        typ: CK_MECHANISM_TYPE,
        info: Box<dyn Mechanism>,
    ) {
        self.tree.insert(typ, info);
    }

    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn list(&self) -> Vec<CK_MECHANISM_TYPE> {
        self.tree.keys().cloned().collect()
    }

    pub fn info(&self, typ: CK_MECHANISM_TYPE) -> Option<&CK_MECHANISM_INFO> {
        match self.tree.get(&typ) {
            Some(m) => Some(m.info()),
            None => None,
        }
    }

    pub fn get(&self, typ: CK_MECHANISM_TYPE) -> KResult<&Box<dyn Mechanism>> {
        match self.tree.get(&typ) {
            Some(m) => Ok(m),
            None => err_rv!(CKR_MECHANISM_INVALID),
        }
    }
}

pub trait MechOperation: Debug + Send + Sync {
    fn mechanism(&self) -> CK_MECHANISM_TYPE;
    fn in_use(&self) -> bool;
    fn finalized(&self) -> bool;
    fn reset(&mut self) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
}

pub trait Encryption: MechOperation {
    fn encrypt(
        &mut self,
        _rng: &mut RNG,
        _plain: &[u8],
        _cipher: CK_BYTE_PTR,
        _cipher_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn encrypt_update(
        &mut self,
        _rng: &mut RNG,
        _plain: &[u8],
        _cipher: CK_BYTE_PTR,
        _cipher_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn encrypt_final(
        &mut self,
        _rng: &mut RNG,
        _cipher: CK_BYTE_PTR,
        _cipher_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn encryption_len(&self) -> KResult<usize> {
        err_rv!(CKR_GENERAL_ERROR)
    }
}

pub trait Decryption: MechOperation {
    fn decrypt(
        &mut self,
        _rng: &mut RNG,
        _cipher: &[u8],
        _plain: CK_BYTE_PTR,
        _plain_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn decrypt_update(
        &mut self,
        _rng: &mut RNG,
        _cipher: &[u8],
        _plain: CK_BYTE_PTR,
        _plain_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn decrypt_final(
        &mut self,
        _rng: &mut RNG,
        _plain: CK_BYTE_PTR,
        _plain_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn decryption_len(&self) -> KResult<usize> {
        err_rv!(CKR_GENERAL_ERROR)
    }
}

pub trait SearchOperation: Debug + Send + Sync {
    fn finalized(&self) -> bool;
    fn results(&mut self, _max: usize) -> KResult<Vec<CK_OBJECT_HANDLE>> {
        err_rv!(CKR_GENERAL_ERROR)
    }
}

pub trait Digest: MechOperation {
    fn digest(&mut self, _data: &[u8], _digest: &mut [u8]) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn digest_update(&mut self, _data: &[u8]) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn digest_final(&mut self, _digest: &mut [u8]) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn digest_len(&self) -> KResult<usize> {
        err_rv!(CKR_GENERAL_ERROR)
    }
}

pub trait Sign: MechOperation {
    fn sign(
        &mut self,
        _rng: &mut RNG,
        _data: &[u8],
        _signature: &mut [u8],
    ) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn sign_update(&mut self, _data: &[u8]) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn sign_final(
        &mut self,
        _rng: &mut RNG,
        _signature: &mut [u8],
    ) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn signature_len(&self) -> KResult<usize> {
        err_rv!(CKR_GENERAL_ERROR)
    }
}

pub trait Verify: MechOperation {
    fn verify(&mut self, _data: &[u8], _signature: &[u8]) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn verify_update(&mut self, _data: &[u8]) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn verify_final(&mut self, _signature: &[u8]) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }

    fn signature_len(&self) -> KResult<usize> {
        err_rv!(CKR_GENERAL_ERROR)
    }
}

#[derive(Debug)]
pub enum Operation {
    Empty,
    Search(Box<dyn SearchOperation>),
    Encryption(Box<dyn Encryption>),
    Decryption(Box<dyn Decryption>),
    Digest(Box<dyn Digest>),
    Sign(Box<dyn Sign>),
    Verify(Box<dyn Verify>),
}

impl Operation {
    pub fn finalized(&self) -> bool {
        match self {
            Operation::Empty => true,
            Operation::Search(op) => op.finalized(),
            Operation::Encryption(op) => op.finalized(),
            Operation::Decryption(op) => op.finalized(),
            Operation::Digest(op) => op.finalized(),
            Operation::Sign(op) => op.finalized(),
            Operation::Verify(op) => op.finalized(),
        }
    }
}

pub trait DRBG: Debug + Send + Sync {
    fn init(
        &mut self,
        _entropy: &[u8],
        _nonce: &[u8],
        _pers: &[u8],
    ) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn reseed(&mut self, _entropy: &[u8], _addtl: &[u8]) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
    fn generate(&mut self, _addtl: &[u8], _output: &mut [u8]) -> KResult<()> {
        err_rv!(CKR_GENERAL_ERROR)
    }
}
