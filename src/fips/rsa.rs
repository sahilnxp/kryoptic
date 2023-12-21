// Copyright 2023 Simo Sorce
// See LICENSE.txt file for terms

use super::fips;
use super::mechanism;
use super::rng;
use fips::*;
use mechanism::*;
use rng::RNG;

use std::slice;
use zeroize::Zeroize;

struct RsaDecompose {
    n: Vec<u8>,
    e: Vec<u8>,
    d: Vec<u8>,
    p: Vec<u8>,
    q: Vec<u8>,
    a: Vec<u8>,
    b: Vec<u8>,
    c: Vec<u8>,
}

impl RsaDecompose {
    pub fn new(n: &[u8], e: &[u8], d: &[u8]) -> RsaDecompose {
        RsaDecompose {
            n: n.to_vec(),
            e: e.to_vec(),
            d: d.to_vec(),
            p: Vec::with_capacity(n.len()),
            q: Vec::with_capacity(n.len()),
            a: Vec::with_capacity(n.len()),
            b: Vec::with_capacity(n.len()),
            c: Vec::with_capacity(n.len()),
        }
    }

    pub fn decompose(&mut self) -> KResult<()> {
        err_rv!(CKR_DEVICE_ERROR)
    }

    pub fn comp_slice(a: &Vec<u8>, b: &[u8]) -> i8 {
        if a.len() > b.len() {
            1
        } else if a.len() < b.len() {
            -1
        } else {
            /* FIXME: */
            0
        }
    }

    pub fn to_u8_vec(a: &Vec<u8>) -> Vec<u8> {
        a.clone()
    }
}

macro_rules! make_bn {
    ($obj:expr; $id:expr; $name:expr; $vec:expr) => {{
        let x = match $obj.get_attr_as_bytes($id) {
            Ok(b) => b,
            Err(_) => return err_rv!(CKR_DEVICE_ERROR),
        };
        let bn = unsafe {
            BN_bin2bn(x.as_ptr() as *mut u8, x.len() as i32, std::ptr::null_mut())
        };
        if bn.is_null() {
            return err_rv!(CKR_DEVICE_ERROR);
        }
        let mut param = unsafe {
            OSSL_PARAM_construct_BN($name as *const u8 as *const i8, std::ptr::null_mut(), 0)
        };
        /* calculate needed size */
        unsafe { OSSL_PARAM_set_BN(&mut param, bn); }
        $vec.resize(param.return_size, 0);
        unsafe {
            param.data = $vec.as_mut_ptr() as *mut std::os::raw::c_void;
            param.data_size = $vec.len();
            OSSL_PARAM_set_BN(&mut param, bn);
        }
        param
    }};
}

fn object_to_rsa_public_key(key: &Object) -> KResult<EvpPkey> {
    let mut nvec: Vec<u8> = Vec::new();
    let mut evec: Vec<u8> = Vec::new();
    let mut params = [
        make_bn!(key; CKA_MODULUS; OSSL_PKEY_PARAM_RSA_N; nvec),
        make_bn!(key; CKA_PUBLIC_EXPONENT; OSSL_PKEY_PARAM_RSA_E; evec),
        unsafe { OSSL_PARAM_construct_end() }
    ];

    let mut ctx = EvpPkeyCtx::from_ptr(
        unsafe {
            EVP_PKEY_CTX_new_from_name(
                get_libctx(),
                b"RSA\0".as_ptr() as *const i8,
                std::ptr::null())
        }
    )?;
    if unsafe { EVP_PKEY_fromdata_init(ctx.as_mut_ptr()) } != 1 {
        return err_rv!(CKR_DEVICE_ERROR);
    }
    let mut pkey: *mut EVP_PKEY = std::ptr::null_mut();
    if unsafe { EVP_PKEY_fromdata(
        ctx.as_mut_ptr(),
        &mut pkey,
        EVP_PKEY_PUBLIC_KEY as i32,
        params.as_mut_ptr()) } != 1 {
        return err_rv!(CKR_DEVICE_ERROR);
    }
    nvec.zeroize();
    evec.zeroize();
    EvpPkey::from_ptr(pkey)
}

fn object_to_rsa_private_key(key: &Object) -> KResult<EvpPkey> {
    let mut nvec: Vec<u8> = Vec::new();
    let mut evec: Vec<u8> = Vec::new();
    let mut dvec: Vec<u8> = Vec::new();
    let mut pvec: Vec<u8> = Vec::new();
    let mut qvec: Vec<u8> = Vec::new();
    let mut avec: Vec<u8> = Vec::new();
    let mut bvec: Vec<u8> = Vec::new();
    let mut cvec: Vec<u8> = Vec::new();
    let mut params = [
        make_bn!(key; CKA_MODULUS; OSSL_PKEY_PARAM_RSA_N; nvec),
        make_bn!(key; CKA_PUBLIC_EXPONENT; OSSL_PKEY_PARAM_RSA_E; evec),
        make_bn!(key; CKA_PRIVATE_EXPONENT; OSSL_PKEY_PARAM_RSA_D; dvec),
        unsafe { OSSL_PARAM_construct_end() },
        unsafe { OSSL_PARAM_construct_end() },
        unsafe { OSSL_PARAM_construct_end() },
        unsafe { OSSL_PARAM_construct_end() },
        unsafe { OSSL_PARAM_construct_end() },
        unsafe { OSSL_PARAM_construct_end() },
        unsafe { OSSL_PARAM_construct_end() },
    ];
    if key.get_attr(CKA_PRIME_1).is_some() &&
        key.get_attr(CKA_PRIME_2).is_some() &&
        key.get_attr(CKA_EXPONENT_1).is_some() &&
        key.get_attr(CKA_EXPONENT_2).is_some() &&
        key.get_attr(CKA_COEFFICIENT).is_some() {
        params[3] = make_bn!(key; CKA_PRIME_1; OSSL_PKEY_PARAM_RSA_FACTOR1; pvec);
        params[4] = make_bn!(key; CKA_PRIME_2; OSSL_PKEY_PARAM_RSA_FACTOR2; qvec);
        params[5] = make_bn!(key; CKA_EXPONENT_1; OSSL_PKEY_PARAM_RSA_EXPONENT1; avec);
        params[6] = make_bn!(key; CKA_EXPONENT_2; OSSL_PKEY_PARAM_RSA_EXPONENT2; bvec);
        params[7] = make_bn!(key; CKA_COEFFICIENT; OSSL_PKEY_PARAM_RSA_COEFFICIENT1; cvec);
    }

    let mut ctx = EvpPkeyCtx::from_ptr(
        unsafe {
            EVP_PKEY_CTX_new_from_name(
                get_libctx(),
                b"RSA\0".as_ptr() as *const i8,
                std::ptr::null())
        })?;
    if unsafe { EVP_PKEY_fromdata_init(ctx.as_mut_ptr()) } != 1 {
        return err_rv!(CKR_DEVICE_ERROR);
    }
    let mut pkey: *mut EVP_PKEY = std::ptr::null_mut();
    if unsafe { EVP_PKEY_fromdata(
        ctx.as_mut_ptr(),
        &mut pkey,
        EVP_PKEY_PRIVATE_KEY as i32,
        params.as_mut_ptr()) } != 1 {
        return err_rv!(CKR_DEVICE_ERROR);
    }
    nvec.zeroize();
    evec.zeroize();
    dvec.zeroize();
    pvec.zeroize();
    qvec.zeroize();
    avec.zeroize();
    bvec.zeroize();
    cvec.zeroize();
    EvpPkey::from_ptr(pkey)
}

fn empty_private_key() -> EvpPkey {
    EvpPkey::empty()
}

macro_rules! name_to_vec {
    ($name:expr) => {
        unsafe {
            slice::from_raw_parts($name.as_ptr() as *const std::os::raw::c_char, $name.len()).to_vec()
        }
    }
}

fn get_digest_name(mech: CK_MECHANISM_TYPE) -> KResult<Vec<std::os::raw::c_char>> {
    Ok(match mech {
        CKM_RSA_PKCS => Vec::new(),
        CKM_SHA1_RSA_PKCS => name_to_vec!(OSSL_DIGEST_NAME_SHA1),
        CKM_SHA224_RSA_PKCS => name_to_vec!(OSSL_DIGEST_NAME_SHA2_224),
        CKM_SHA256_RSA_PKCS => name_to_vec!(OSSL_DIGEST_NAME_SHA2_256),
        CKM_SHA384_RSA_PKCS => name_to_vec!(OSSL_DIGEST_NAME_SHA2_384),
        CKM_SHA512_RSA_PKCS => name_to_vec!(OSSL_DIGEST_NAME_SHA2_512),
        CKM_SHA3_224_RSA_PKCS => name_to_vec!(OSSL_DIGEST_NAME_SHA3_224),
        CKM_SHA3_256_RSA_PKCS => name_to_vec!(OSSL_DIGEST_NAME_SHA3_256),
        CKM_SHA3_384_RSA_PKCS => name_to_vec!(OSSL_DIGEST_NAME_SHA3_384),
        CKM_SHA3_512_RSA_PKCS => name_to_vec!(OSSL_DIGEST_NAME_SHA3_512),
        _ => return err_rv!(CKR_GENERAL_ERROR),
    })
}

static RSA_NAME: &[u8; 4] = b"RSA\0";
fn rsa_name_as_char() -> *const std::os::raw::c_char {
    RSA_NAME.as_ptr() as *const std::os::raw::c_char
}

#[derive(Debug)]
struct RsaPKCSOperation {
    mech: CK_MECHANISM_TYPE,
    max_input: usize,
    output_len: usize,
    public_key: EvpPkey,
    private_key: EvpPkey,
    finalized: bool,
    in_use: bool,
    sigctx: Option<ProviderSignatureCtx>,
    mdname: Vec<std::os::raw::c_char>,
}

impl RsaPKCSOperation {
    fn encrypt_new(
        mech: &CK_MECHANISM,
        key: &Object,
        info: &CK_MECHANISM_INFO,
    ) -> KResult<RsaPKCSOperation> {
        let modulus = key.get_attr_as_bytes(CKA_MODULUS)?;
        let modulus_bits: u64 = modulus.len() as u64 * 8;
        if modulus_bits < info.ulMinKeySize
            || (info.ulMaxKeySize != 0 && modulus_bits > info.ulMaxKeySize)
        {
            return err_rv!(CKR_KEY_SIZE_RANGE);
        }
        if mech.mechanism != CKM_RSA_PKCS {
            return err_rv!(CKR_MECHANISM_INVALID);
        }
        Ok(RsaPKCSOperation {
            mech: mech.mechanism,
            max_input: modulus.len() - 11,
            output_len: modulus.len(),
            public_key: object_to_rsa_public_key(key)?,
            private_key: empty_private_key(),
            finalized: false,
            in_use: false,
            sigctx: None,
            mdname: Vec::new(),
        })
    }

    fn decrypt_new(
        mech: &CK_MECHANISM,
        key: &Object,
        info: &CK_MECHANISM_INFO,
    ) -> KResult<RsaPKCSOperation> {
        let modulus = key.get_attr_as_bytes(CKA_MODULUS)?;
        let modulus_bits: u64 = modulus.len() as u64 * 8;
        if modulus_bits < info.ulMinKeySize
            || (info.ulMaxKeySize != 0 && modulus_bits > info.ulMaxKeySize)
        {
            return err_rv!(CKR_KEY_SIZE_RANGE);
        }
        if mech.mechanism != CKM_RSA_PKCS {
            return err_rv!(CKR_MECHANISM_INVALID);
        }
        Ok(RsaPKCSOperation {
            mech: mech.mechanism,
            max_input: modulus.len(),
            output_len: modulus.len() - 11,
            public_key: object_to_rsa_public_key(key)?,
            private_key: object_to_rsa_private_key(key)?,
            finalized: false,
            in_use: false,
            sigctx: None,
            mdname: Vec::new(),
        })
    }

    fn sign_new(
        mech: &CK_MECHANISM,
        key: &Object,
        info: &CK_MECHANISM_INFO,
    ) -> KResult<RsaPKCSOperation> {
        let modulus = key.get_attr_as_bytes(CKA_MODULUS)?;
        let modulus_bits: u64 = modulus.len() as u64 * 8;
        if modulus_bits < info.ulMinKeySize
            || (info.ulMaxKeySize != 0 && modulus_bits > info.ulMaxKeySize)
        {
            return err_rv!(CKR_KEY_SIZE_RANGE);
        }

        Ok(RsaPKCSOperation {
            mech: mech.mechanism,
            max_input: match mech.mechanism {
                CKM_RSA_PKCS => modulus.len() - 11,
                _ => 0,
            },
            output_len: modulus.len(),
            public_key: object_to_rsa_public_key(key)?,
            private_key: object_to_rsa_private_key(key)?,
            finalized: false,
            in_use: false,
            sigctx: match mech.mechanism {
                CKM_RSA_PKCS => None,
                _ => Some(ProviderSignatureCtx::new(rsa_name_as_char())?),
            },
            mdname: get_digest_name(mech.mechanism)?,
        })
    }

    fn verify_new(
        mech: &CK_MECHANISM,
        key: &Object,
        info: &CK_MECHANISM_INFO,
    ) -> KResult<RsaPKCSOperation> {
        let modulus = key.get_attr_as_bytes(CKA_MODULUS)?;
        let modulus_bits: u64 = modulus.len() as u64 * 8;
        if modulus_bits < info.ulMinKeySize
            || (info.ulMaxKeySize != 0 && modulus_bits > info.ulMaxKeySize)
        {
            return err_rv!(CKR_KEY_SIZE_RANGE);
        }

        Ok(RsaPKCSOperation {
            mech: mech.mechanism,
            max_input: match mech.mechanism {
                CKM_RSA_PKCS => modulus.len() - 11,
                _ => 0,
            },
            output_len: modulus.len(),
            public_key: object_to_rsa_public_key(key)?,
            private_key: empty_private_key(),
            finalized: false,
            in_use: false,
            sigctx: match mech.mechanism {
                CKM_RSA_PKCS => None,
                _ => Some(ProviderSignatureCtx::new(rsa_name_as_char())?),
            },
            mdname: get_digest_name(mech.mechanism)?,
        })
    }

    fn generate_keypair(
        _rng: &mut rng::RNG,
        _exponent: Vec<u8>,
        _bits: usize,
        _pubkey: &mut Object,
        _privkey: &mut Object,
    ) -> KResult<()> {
        err_rv!(CKR_DEVICE_ERROR)
    }
}

impl MechOperation for RsaPKCSOperation {
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

impl Encryption for RsaPKCSOperation {
    fn encrypt(
        &mut self,
        _rng: &mut RNG,
        plain: &[u8],
        cipher: CK_BYTE_PTR,
        cipher_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        if self.in_use {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        let mut ctx = EvpPkeyCtx::from_ptr(
            unsafe {
                EVP_PKEY_CTX_new_from_pkey(
                    get_libctx(),
                    self.public_key.as_mut_ptr(),
                    std::ptr::null_mut())
            }
        )?;
        if unsafe { EVP_PKEY_encrypt_init(ctx.as_mut_ptr()) } != 1 {
            return err_rv!(CKR_DEVICE_ERROR);
        }
        let params = [
            unsafe { OSSL_PARAM_construct_utf8_string(
                OSSL_PKEY_PARAM_PAD_MODE.as_ptr() as *const i8,
                OSSL_PKEY_RSA_PAD_MODE_PKCSV15.as_ptr() as *mut i8,
                OSSL_PKEY_RSA_PAD_MODE_PKCSV15.len()) },
            unsafe { OSSL_PARAM_construct_end() },
        ];
        if unsafe {
            EVP_PKEY_CTX_set_params(
                ctx.as_mut_ptr(),
                params.as_ptr())
        } != 1 {
            return err_rv!(CKR_DEVICE_ERROR);
        }

        let mut outlen = 0usize;
        let outlen_ptr: *mut usize = &mut outlen;
        if unsafe {
            EVP_PKEY_encrypt(
                ctx.as_mut_ptr(),
                std::ptr::null_mut(),
                outlen_ptr,
                plain.as_ptr(),
                plain.len())
        } != 1 {
            return err_rv!(CKR_DEVICE_ERROR);
        }
        if cipher.is_null() {
            unsafe { *cipher_len = outlen as CK_ULONG; }
            return Ok(());
        } else {
            unsafe {
                if (*cipher_len as usize) < outlen {
                    return err_rv!(CKR_BUFFER_TOO_SMALL);
                }
            }
        }

        self.finalized = true;

        if unsafe {
            EVP_PKEY_encrypt(
                ctx.as_mut_ptr(),
                cipher,
                outlen_ptr,
                plain.as_ptr(),
                plain.len())
        } != 1 {
            return err_rv!(CKR_DEVICE_ERROR);
        }
        unsafe { *cipher_len = outlen as CK_ULONG; }
        Ok(())
    }

    fn encrypt_update(
        &mut self,
        _rng: &mut RNG,
        _plain: &[u8],
        _cipher: CK_BYTE_PTR,
        _cipher_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        self.finalized = true;
        return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
    }

    fn encrypt_final(
        &mut self,
        _rng: &mut RNG,
        _cipher: CK_BYTE_PTR,
        _cipher_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        self.finalized = true;
        return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
    }

    fn encryption_len(&self) -> KResult<usize> {
        match self.mech {
            CKM_RSA_PKCS => Ok(self.output_len),
            _ => err_rv!(CKR_GENERAL_ERROR),
        }
    }
}

impl Decryption for RsaPKCSOperation {
    fn decrypt(
        &mut self,
        _rng: &mut RNG,
        cipher: &[u8],
        plain: CK_BYTE_PTR,
        plain_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        if self.in_use {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        unsafe {
            let mut ctx = EvpPkeyCtx::from_ptr(
                EVP_PKEY_CTX_new_from_pkey(
                    get_libctx(),
                    self.private_key.as_mut_ptr(),
                    std::ptr::null_mut()))?;
            if EVP_PKEY_decrypt_init(ctx.as_mut_ptr()) != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }
            let params = [
                OSSL_PARAM_construct_utf8_string(
                    OSSL_PKEY_PARAM_PAD_MODE.as_ptr() as *const i8,
                    OSSL_PKEY_RSA_PAD_MODE_PKCSV15.as_ptr() as *mut i8,
                    OSSL_PKEY_RSA_PAD_MODE_PKCSV15.len()),
                OSSL_PARAM_construct_end(),
            ];
            if EVP_PKEY_CTX_set_params(
                ctx.as_mut_ptr(), params.as_ptr()) != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }

            let mut outlen = 0usize;
            let outlen_ptr: *mut usize = &mut outlen;
            if EVP_PKEY_decrypt(ctx.as_mut_ptr(), std::ptr::null_mut(), outlen_ptr, cipher.as_ptr(), cipher.len()) != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }
            if plain.is_null() {
                *plain_len = outlen as CK_ULONG;
                return Ok(());
            } else {
                if (*plain_len as usize) < outlen {
                    return err_rv!(CKR_BUFFER_TOO_SMALL);
                }
            }

            self.finalized = true;

            if EVP_PKEY_decrypt(ctx.as_mut_ptr(), plain, outlen_ptr, cipher.as_ptr(), cipher.len()) != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }
            *plain_len = outlen as CK_ULONG;
        }
        Ok(())
    }

    fn decrypt_update(
        &mut self,
        _rng: &mut RNG,
        _cipher: &[u8],
        _plain: CK_BYTE_PTR,
        _plain_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        self.finalized = true;
        return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
    }

    fn decrypt_final(
        &mut self,
        _rng: &mut RNG,
        _plain: CK_BYTE_PTR,
        _plain_len: CK_ULONG_PTR,
    ) -> KResult<()> {
        self.finalized = true;
        return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
    }

    fn decryption_len(&self) -> KResult<usize> {
        match self.mech {
            CKM_RSA_PKCS => Ok(self.output_len),
            _ => err_rv!(CKR_GENERAL_ERROR),
        }
    }
}

impl Sign for RsaPKCSOperation {
    fn sign(
        &mut self,
        rng: &mut RNG,
        data: &[u8],
        signature: &mut [u8],
    ) -> KResult<()> {
        if self.in_use {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if self.mech == CKM_RSA_PKCS {
            self.finalized = true;
            if data.len() > self.max_input {
                return err_rv!(CKR_DATA_LEN_RANGE);
            }
            if signature.len() != self.output_len {
                return err_rv!(CKR_GENERAL_ERROR);
            }
            let mut ctx = EvpPkeyCtx::from_ptr(
                unsafe {
                    EVP_PKEY_CTX_new_from_pkey(
                        get_libctx(),
                        self.private_key.as_mut_ptr(),
                        std::ptr::null_mut())
                }
            )?;
            if unsafe {
                EVP_PKEY_sign_init(ctx.as_mut_ptr())
            } != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }
            let params = [
                unsafe {
                    OSSL_PARAM_construct_utf8_string(
                        OSSL_PKEY_PARAM_PAD_MODE.as_ptr() as *const i8,
                        OSSL_PKEY_RSA_PAD_MODE_PKCSV15.as_ptr() as *mut i8,
                        OSSL_PKEY_RSA_PAD_MODE_PKCSV15.len())
                },
                unsafe { OSSL_PARAM_construct_end() },
            ];
            if unsafe {
                EVP_PKEY_CTX_set_params(
                    ctx.as_mut_ptr(),
                    params.as_ptr())
            } != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }

            self.finalized = true;

            let mut siglen = 0usize;
            let siglen_ptr: *mut usize = &mut siglen;
            if unsafe {
                EVP_PKEY_sign(
                    ctx.as_mut_ptr(),
                    std::ptr::null_mut(),
                    siglen_ptr,
                    data.as_ptr(),
                    data.len())
            } != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }
            if signature.len() != siglen {
                return err_rv!(CKR_GENERAL_ERROR);
            }

            if unsafe {
                EVP_PKEY_sign(
                    ctx.as_mut_ptr(),
                    signature.as_mut_ptr(),
                    siglen_ptr,
                    data.as_ptr(),
                    data.len())
            } != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }

            return Ok(())
        }
        self.sign_update(data)?;
        self.sign_final(rng, signature)
    }

    fn sign_update(&mut self, data: &[u8]) -> KResult<()> {
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if !self.in_use {
            if self.mech == CKM_RSA_PKCS {
                return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
            }
            self.in_use = true;

            let params = [
                unsafe {
                    OSSL_PARAM_construct_utf8_string(
                        OSSL_SIGNATURE_PARAM_PAD_MODE.as_ptr() as *const i8,
                        OSSL_PKEY_RSA_PAD_MODE_PKCSV15.as_ptr() as *mut i8,
                        OSSL_PKEY_RSA_PAD_MODE_PKCSV15.len())
                },
                unsafe { OSSL_PARAM_construct_end() },
            ];
            self.sigctx.as_mut().unwrap().digest_sign_init(
                self.mdname.as_ptr(),
                &self.private_key,
                params.as_ptr())?;
        }

        self.sigctx.as_mut().unwrap().digest_sign_update(data)
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

        self.sigctx.as_mut().unwrap().digest_sign_final(signature)
    }

    fn signature_len(&self) -> KResult<usize> {
        Ok(self.output_len)
    }
}

impl Verify for RsaPKCSOperation {
    fn verify(&mut self, data: &[u8], signature: &[u8]) -> KResult<()> {
        if self.in_use {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if self.mech == CKM_RSA_PKCS {
            self.finalized = true;
            if data.len() > self.max_input {
                return err_rv!(CKR_DATA_LEN_RANGE);
            }
            if signature.len() != self.output_len {
                return err_rv!(CKR_GENERAL_ERROR);
            }
            let mut ctx = EvpPkeyCtx::from_ptr(
                unsafe {
                    EVP_PKEY_CTX_new_from_pkey(
                        get_libctx(),
                        self.public_key.as_mut_ptr(),
                        std::ptr::null_mut())
                }
            )?;
            if unsafe {
                EVP_PKEY_verify_init(ctx.as_mut_ptr())
            } != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }
            let params = [
                unsafe {
                    OSSL_PARAM_construct_utf8_string(
                        OSSL_PKEY_PARAM_PAD_MODE.as_ptr() as *const i8,
                        OSSL_PKEY_RSA_PAD_MODE_PKCSV15.as_ptr() as *mut i8,
                        OSSL_PKEY_RSA_PAD_MODE_PKCSV15.len())
                },
                unsafe { OSSL_PARAM_construct_end() },
            ];
            if unsafe {
                EVP_PKEY_CTX_set_params(
                    ctx.as_mut_ptr(),
                    params.as_ptr())
            } != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }

            self.finalized = true;

            let mut siglen = signature.len();
            let siglen_ptr: *mut usize = &mut siglen;
            if unsafe {
                EVP_PKEY_sign(
                    ctx.as_mut_ptr(),
                    signature.as_ptr() as *mut u8,
                    siglen_ptr,
                    data.as_ptr(),
                    data.len())
            } != 1 {
                return err_rv!(CKR_DEVICE_ERROR);
            }
            return Ok(())
        }
        self.verify_update(data)?;
        self.verify_final(signature)
    }

    fn verify_update(&mut self, data: &[u8]) -> KResult<()> {
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if !self.in_use {
            if self.mech == CKM_RSA_PKCS {
                return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
            }
            self.in_use = true;

            let params = [
                unsafe {
                    OSSL_PARAM_construct_utf8_string(
                        OSSL_SIGNATURE_PARAM_PAD_MODE.as_ptr() as *const i8,
                        OSSL_PKEY_RSA_PAD_MODE_PKCSV15.as_ptr() as *mut i8,
                        OSSL_PKEY_RSA_PAD_MODE_PKCSV15.len())
                },
                unsafe { OSSL_PARAM_construct_end() },
            ];
            self.sigctx.as_mut().unwrap().digest_verify_init(
                self.mdname.as_ptr(),
                &self.public_key,
                params.as_ptr())?;
        }

        self.sigctx.as_mut().unwrap().digest_verify_update(data)
    }

    fn verify_final(&mut self, signature: &[u8]) -> KResult<()> {
        if !self.in_use {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        if self.finalized {
            return err_rv!(CKR_OPERATION_NOT_INITIALIZED);
        }
        self.finalized = true;

        self.sigctx.as_mut().unwrap().digest_verify_final(signature)
    }

    fn signature_len(&self) -> KResult<usize> {
        Ok(self.output_len)
    }
}
