// Copyright 2023 Simo Sorce
// See LICENSE.txt file for terms

use super::cryptography;
use super::err_rv;
use super::error;
use super::interface;
use super::mechanism;
use super::object;
use super::sha1;
use super::sha2;
use super::token;
use cryptography::*;
use error::{KError, KResult};
use interface::*;
use mechanism::*;
use object::{Object, ObjectTemplates};
use std::fmt::Debug;
use token::RNG;
use zeroize::Zeroize;

fn check_and_fetch_key(key: &Object, keytype: CK_KEY_TYPE) -> KResult<Vec<u8>> {
    if key.get_attr_as_ulong(CKA_CLASS)? != CKO_SECRET_KEY {
        return err_rv!(CKR_KEY_TYPE_INCONSISTENT);
    }
    let t = key.get_attr_as_ulong(CKA_KEY_TYPE)?;
    if t != CKK_GENERIC_SECRET && t != keytype {
        return err_rv!(CKR_KEY_TYPE_INCONSISTENT);
    }

    Ok(key.get_attr_as_bytes(CKA_VALUE)?.clone())
}

fn check_and_fetch_param(
    mech: &CK_MECHANISM,
    min: usize,
    max: usize,
) -> KResult<usize> {
    if min == max {
        if mech.ulParameterLen != 0 {
            return err_rv!(CKR_MECHANISM_PARAM_INVALID);
        }
        return Ok(max);
    }
    if mech.ulParameterLen != std::mem::size_of::<CK_ULONG>() as CK_ULONG {
        return err_rv!(CKR_MECHANISM_PARAM_INVALID);
    }
    let genlen = unsafe {
        let val: &[CK_ULONG] =
            std::slice::from_raw_parts(mech.pParameter as *const _, 1);
        val[0] as usize
    };
    if genlen < min || genlen > max {
        return err_rv!(CKR_MECHANISM_PARAM_INVALID);
    }
    Ok(genlen)
}

#[derive(Debug)]
struct HMACMechanism {
    info: CK_MECHANISM_INFO,
    keytype: CK_KEY_TYPE,
    minlen: usize,
    maxlen: usize,
}

impl Mechanism for HMACMechanism {
    fn info(&self) -> &CK_MECHANISM_INFO {
        &self.info
    }

    fn sign_new(
        &self,
        mech: &CK_MECHANISM,
        keyobj: &Object,
    ) -> KResult<Box<dyn Sign>> {
        if self.info.flags & CKF_SIGN != CKF_SIGN {
            return err_rv!(CKR_MECHANISM_INVALID);
        }
        let output_len = check_and_fetch_param(mech, self.minlen, self.maxlen)?;
        let key = check_and_fetch_key(keyobj, self.keytype)?;
        let hash = match mech.mechanism {
            CKM_SHA_1_HMAC | CKM_SHA_1_HMAC_GENERAL => {
                Spec_Hash_Definitions_SHA1
            }
            CKM_SHA256_HMAC | CKM_SHA256_HMAC_GENERAL => {
                Spec_Hash_Definitions_SHA2_256
            }
            CKM_SHA384_HMAC | CKM_SHA384_HMAC_GENERAL => {
                Spec_Hash_Definitions_SHA2_384
            }
            CKM_SHA512_HMAC | CKM_SHA512_HMAC_GENERAL => {
                Spec_Hash_Definitions_SHA2_512
            }
            _ => return err_rv!(CKR_MECHANISM_INVALID),
        };
        Ok(Box::new(HMACOperation::init(
            mech.mechanism,
            hash,
            key,
            output_len,
        )?))
    }

    fn verify_new(
        &self,
        mech: &CK_MECHANISM,
        keyobj: &Object,
    ) -> KResult<Box<dyn Verify>> {
        if self.info.flags & CKF_VERIFY != CKF_VERIFY {
            return err_rv!(CKR_MECHANISM_INVALID);
        }
        let output_len = check_and_fetch_param(mech, self.minlen, self.maxlen)?;
        let key = check_and_fetch_key(keyobj, self.keytype)?;
        let hash = match mech.mechanism {
            CKM_SHA_1_HMAC | CKM_SHA_1_HMAC_GENERAL => {
                Spec_Hash_Definitions_SHA1
            }
            CKM_SHA256_HMAC | CKM_SHA256_HMAC_GENERAL => {
                Spec_Hash_Definitions_SHA2_256
            }
            CKM_SHA384_HMAC | CKM_SHA384_HMAC_GENERAL => {
                Spec_Hash_Definitions_SHA2_384
            }
            CKM_SHA512_HMAC | CKM_SHA512_HMAC_GENERAL => {
                Spec_Hash_Definitions_SHA2_512
            }
            _ => return err_rv!(CKR_MECHANISM_INVALID),
        };
        Ok(Box::new(HMACOperation::init(
            mech.mechanism,
            hash,
            key,
            output_len,
        )?))
    }
}

pub fn register(mechs: &mut Mechanisms, _ot: &mut ObjectTemplates) {
    mechs.add_mechanism(
        CKM_SHA_1_HMAC_GENERAL,
        Box::new(HMACMechanism {
            info: CK_MECHANISM_INFO {
                ulMinKeySize: 0,
                ulMaxKeySize: 0,
                flags: CKF_SIGN | CKF_VERIFY,
            },
            keytype: CKK_SHA_1_HMAC,
            minlen: 1,
            maxlen: sha1::SHA1Operation::hashlen(),
        }),
    );
    mechs.add_mechanism(
        CKM_SHA_1_HMAC,
        Box::new(HMACMechanism {
            info: CK_MECHANISM_INFO {
                ulMinKeySize: 0,
                ulMaxKeySize: 0,
                flags: CKF_SIGN | CKF_VERIFY,
            },
            keytype: CKK_SHA_1_HMAC,
            minlen: sha1::SHA1Operation::hashlen(),
            maxlen: sha1::SHA1Operation::hashlen(),
        }),
    );
    mechs.add_mechanism(
        CKM_SHA256_HMAC_GENERAL,
        Box::new(HMACMechanism {
            info: CK_MECHANISM_INFO {
                ulMinKeySize: 0,
                ulMaxKeySize: 0,
                flags: CKF_SIGN | CKF_VERIFY,
            },
            keytype: CKK_SHA256_HMAC,
            minlen: 1,
            maxlen: sha2::SHA256Operation::hashlen(),
        }),
    );
    mechs.add_mechanism(
        CKM_SHA256_HMAC,
        Box::new(HMACMechanism {
            info: CK_MECHANISM_INFO {
                ulMinKeySize: 0,
                ulMaxKeySize: 0,
                flags: CKF_SIGN | CKF_VERIFY,
            },
            keytype: CKK_SHA256_HMAC,
            minlen: sha2::SHA256Operation::hashlen(),
            maxlen: sha2::SHA256Operation::hashlen(),
        }),
    );
    mechs.add_mechanism(
        CKM_SHA384_HMAC_GENERAL,
        Box::new(HMACMechanism {
            info: CK_MECHANISM_INFO {
                ulMinKeySize: 0,
                ulMaxKeySize: 0,
                flags: CKF_SIGN | CKF_VERIFY,
            },
            keytype: CKK_SHA384_HMAC,
            minlen: 1,
            maxlen: sha2::SHA384Operation::hashlen(),
        }),
    );
    mechs.add_mechanism(
        CKM_SHA384_HMAC,
        Box::new(HMACMechanism {
            info: CK_MECHANISM_INFO {
                ulMinKeySize: 0,
                ulMaxKeySize: 0,
                flags: CKF_SIGN | CKF_VERIFY,
            },
            keytype: CKK_SHA384_HMAC,
            minlen: sha2::SHA384Operation::hashlen(),
            maxlen: sha2::SHA384Operation::hashlen(),
        }),
    );
    mechs.add_mechanism(
        CKM_SHA512_HMAC_GENERAL,
        Box::new(HMACMechanism {
            info: CK_MECHANISM_INFO {
                ulMinKeySize: 0,
                ulMaxKeySize: 0,
                flags: CKF_SIGN | CKF_VERIFY,
            },
            keytype: CKK_SHA512_HMAC,
            minlen: 1,
            maxlen: sha2::SHA512Operation::hashlen(),
        }),
    );
    mechs.add_mechanism(
        CKM_SHA512_HMAC,
        Box::new(HMACMechanism {
            info: CK_MECHANISM_INFO {
                ulMinKeySize: 0,
                ulMaxKeySize: 0,
                flags: CKF_SIGN | CKF_VERIFY,
            },
            keytype: CKK_SHA512_HMAC,
            minlen: sha2::SHA512Operation::hashlen(),
            maxlen: sha2::SHA512Operation::hashlen(),
        }),
    );
}

/* HMAC spec From FIPS 198-1 */
#[derive(Debug)]
struct HMACOperation {
    mech: CK_MECHANISM_TYPE,
    hashlen: usize,
    blocklen: usize,
    outputlen: usize,
    state: Vec<u8>,
    ipad: Vec<u8>,
    opad: Vec<u8>,
    inner: Operation,
    finalized: bool,
    in_use: bool,
}

impl Drop for HMACOperation {
    fn drop(&mut self) {
        self.state.zeroize();
        self.ipad.zeroize();
        self.opad.zeroize();
    }
}

impl HMACOperation {
    fn init(
        mech: CK_MECHANISM_TYPE,
        hash: Spec_Hash_Definitions_hash_alg,
        key: Vec<u8>,
        outputlen: usize,
    ) -> KResult<HMACOperation> {
        let mut hmac = HMACOperation {
            mech: mech,
            hashlen: 0usize,
            blocklen: 0usize,
            outputlen: outputlen,
            state: Vec::new(),
            ipad: Vec::new(),
            opad: Vec::new(),
            inner: Operation::Empty,
            finalized: false,
            in_use: false,
        };
        if hash == sha1::SHA1Operation::specdef() {
            hmac.hashlen = sha1::SHA1Operation::hashlen();
            hmac.blocklen = sha1::SHA1Operation::blocklen();
            hmac.inner =
                Operation::Digest(Box::new(sha1::SHA1Operation::new()));
        } else if hash == sha2::SHA256Operation::specdef() {
            hmac.hashlen = sha2::SHA256Operation::hashlen();
            hmac.blocklen = sha2::SHA256Operation::blocklen();
            hmac.inner =
                Operation::Digest(Box::new(sha2::SHA256Operation::new()));
        } else if hash == sha2::SHA384Operation::specdef() {
            hmac.hashlen = sha2::SHA384Operation::hashlen();
            hmac.blocklen = sha2::SHA384Operation::blocklen();
            hmac.inner =
                Operation::Digest(Box::new(sha2::SHA384Operation::new()));
        } else if hash == sha2::SHA512Operation::specdef() {
            hmac.hashlen = sha2::SHA512Operation::hashlen();
            hmac.blocklen = sha2::SHA512Operation::blocklen();
            hmac.inner =
                Operation::Digest(Box::new(sha2::SHA512Operation::new()));
        } else {
            return err_rv!(CKR_GENERAL_ERROR);
        };

        /* K0 */
        if key.len() <= hmac.blocklen {
            hmac.state.extend_from_slice(key.as_slice());
        } else {
            hmac.state.resize(hmac.hashlen, 0);
            match &mut hmac.inner {
                Operation::Digest(op) => {
                    op.digest(key.as_slice(), hmac.state.as_mut_slice())?
                }
                _ => return err_rv!(CKR_GENERAL_ERROR),
            }
        }
        hmac.state.resize(hmac.blocklen, 0);
        /* K0 ^ ipad */
        hmac.ipad.resize(hmac.blocklen, 0x36);
        hmac.ipad
            .iter_mut()
            .zip(hmac.state.iter())
            .for_each(|(i1, i2)| *i1 ^= *i2);
        /* K0 ^ opad */
        hmac.opad.resize(hmac.blocklen, 0x5c);
        hmac.opad
            .iter_mut()
            .zip(hmac.state.iter())
            .for_each(|(i1, i2)| *i1 ^= *i2);
        /* H((K0 ^ ipad) || .. ) */
        match &mut hmac.inner {
            Operation::Digest(op) => {
                op.reset()?;
                op.digest_update(hmac.ipad.as_slice())?;
            }
            _ => return err_rv!(CKR_GENERAL_ERROR),
        }
        Ok(hmac)
    }

    fn update(&mut self, data: &[u8]) -> KResult<()> {
        /* H( .. || text ..) */
        match &mut self.inner {
            Operation::Digest(op) => op.digest_update(data),
            _ => err_rv!(CKR_GENERAL_ERROR),
        }
    }
    fn finalize(&mut self, output: &mut [u8]) -> KResult<()> {
        self.state.resize(self.hashlen, 0);
        /* state = H((K0 ^ ipad) || text) */
        match &mut self.inner {
            Operation::Digest(op) => {
                op.digest_final(self.state.as_mut_slice())?;
            }
            _ => return err_rv!(CKR_GENERAL_ERROR),
        }
        /* state = H((K0 ^ opad) || H((K0 ^ ipad) || text)) */
        match &mut self.inner {
            Operation::Digest(op) => {
                op.reset()?;
                op.digest_update(self.opad.as_slice())?;
                op.digest_update(self.state.as_slice())?;
                op.digest_final(self.state.as_mut_slice())?;
            }
            _ => return err_rv!(CKR_GENERAL_ERROR),
        }
        /* state -> output */
        output.copy_from_slice(&self.state[..output.len()]);
        Ok(())
    }
}

impl MechOperation for HMACOperation {
    fn mechanism(&self) -> CK_MECHANISM_TYPE {
        self.mech
    }
    fn in_use(&self) -> bool {
        self.in_use
    }
    fn finalized(&self) -> bool {
        self.finalized
    }
}

impl Sign for HMACOperation {
    fn sign(
        &mut self,
        rng: &mut RNG,
        data: &[u8],
        signature: &mut [u8],
    ) -> KResult<()> {
        if self.in_use {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        match self.sign_update(data) {
            Err(e) => {
                self.finalized = true;
                return Err(e);
            }
            Ok(()) => (),
        }
        self.sign_final(rng, signature)
    }

    fn sign_update(&mut self, data: &[u8]) -> KResult<()> {
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        self.in_use = true;
        self.update(data)
    }

    fn sign_final(
        &mut self,
        _rng: &mut RNG,
        signature: &mut [u8],
    ) -> KResult<()> {
        if !self.in_use {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        self.finalized = true;
        if signature.len() != self.outputlen {
            return err_rv!(CKR_GENERAL_ERROR);
        }
        self.finalize(signature)
    }

    fn signature_len(&self) -> KResult<usize> {
        Ok(self.outputlen)
    }
}

impl Verify for HMACOperation {
    fn verify(&mut self, data: &[u8], signature: &[u8]) -> KResult<()> {
        if self.in_use {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        match self.verify_update(data) {
            Err(e) => {
                self.finalized = true;
                return Err(e);
            }
            Ok(()) => (),
        }
        self.verify_final(signature)
    }

    fn verify_update(&mut self, data: &[u8]) -> KResult<()> {
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        self.in_use = true;
        self.update(data)
    }

    fn verify_final(&mut self, signature: &[u8]) -> KResult<()> {
        if !self.in_use {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        self.finalized = true;
        if signature.len() != self.outputlen {
            return err_rv!(CKR_GENERAL_ERROR);
        }
        let mut verify: Vec<u8> = vec![0; self.outputlen];
        self.finalize(verify.as_mut_slice())?;
        if verify != signature {
            return err_rv!(CKR_SIGNATURE_INVALID);
        }
        Ok(())
    }

    fn signature_len(&self) -> KResult<usize> {
        Ok(self.outputlen)
    }
}
