// Copyright 2023 Simo Sorce
// See LICENSE.txt file for terms

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_snake_case)]
include!("nettle_bindings.rs");

use core::fmt::Error;
use std::fmt::Debug;
use std::fmt::Formatter;
use zeroize::Zeroize;

unsafe impl Send for rsa_public_key {}
unsafe impl Sync for rsa_public_key {}
unsafe impl Send for rsa_private_key {}
unsafe impl Sync for rsa_private_key {}

impl Debug for rsa_public_key {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("rsa_public_key")
            .field("size", &self.size)
            .field("e", &"e")
            .field("n", &"n")
            .finish()
    }
}

impl Debug for rsa_private_key {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("rsa_private_key")
            .field("size", &self.size)
            .field("d", &"d")
            .field("p", &"p")
            .field("q", &"q")
            .field("a", &"a")
            .field("b", &"b")
            .field("c", &"c")
            .finish()
    }
}

macro_rules! zero_mpz_struct {
    ($field:expr) => {
        let z: &mut [::std::os::raw::c_ulong] = unsafe {
            std::slice::from_raw_parts_mut(
                $field._mp_d,
                $field._mp_alloc as usize,
            )
        };
        z.zeroize();
    };
}

impl Drop for rsa_public_key {
    fn drop(&mut self) {
        unsafe { nettle_rsa_public_key_clear(self) };
    }
}

impl Drop for rsa_private_key {
    fn drop(&mut self) {
        zero_mpz_struct!(self.d[0]);
        zero_mpz_struct!(self.p[0]);
        zero_mpz_struct!(self.q[0]);
        zero_mpz_struct!(self.a[0]);
        zero_mpz_struct!(self.b[0]);
        zero_mpz_struct!(self.c[0]);
        unsafe { nettle_rsa_private_key_clear(self) };
    }
}

pub struct mpz_struct_wrapper {
    mpz: __mpz_struct,
}

impl mpz_struct_wrapper {
    pub fn new() -> mpz_struct_wrapper {
        let mut x = mpz_struct_wrapper {
            mpz: __mpz_struct::default(),
        };
        unsafe { __gmpz_init(&mut x.mpz) };
        x
    }
    pub fn as_mut_ptr(&mut self) -> &mut __mpz_struct {
        &mut self.mpz
    }
}

impl Drop for mpz_struct_wrapper {
    fn drop(&mut self) {
        zero_mpz_struct!(self.mpz);
        unsafe { __gmpz_clear(&mut self.mpz) };
    }
}

include!("hacl_bindings.rs");

#[derive(Debug)]
pub struct SHA1state {
    s: *mut Hacl_Streaming_SHA1_state,
}

impl SHA1state {
    pub fn new() -> SHA1state {
        SHA1state {
            s: std::ptr::null_mut(),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.s = Hacl_Streaming_SHA1_legacy_create_in();
        }
    }

    pub fn get_state(&mut self) -> *mut Hacl_Streaming_SHA1_state {
        if self.s.is_null() {
            self.init();
        }
        self.s
    }
}

impl Drop for SHA1state {
    fn drop(&mut self) {
        if !self.s.is_null() {
            unsafe {
                Hacl_Streaming_SHA1_legacy_free(self.s);
            }
            self.s = std::ptr::null_mut();
        }
    }
}

unsafe impl Send for SHA1state {}
unsafe impl Sync for SHA1state {}

#[derive(Debug)]
pub struct SHA256state {
    s: *mut Hacl_Streaming_SHA2_state_sha2_256,
}

impl SHA256state {
    pub fn new() -> SHA256state {
        SHA256state {
            s: std::ptr::null_mut(),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.s = Hacl_Streaming_SHA2_create_in_256();
        }
    }

    pub fn get_state(&mut self) -> *mut Hacl_Streaming_SHA2_state_sha2_256 {
        if self.s.is_null() {
            self.init();
        }
        self.s
    }
}

impl Drop for SHA256state {
    fn drop(&mut self) {
        if !self.s.is_null() {
            unsafe {
                Hacl_Streaming_SHA2_free_256(self.s);
            }
            self.s = std::ptr::null_mut();
        }
    }
}

unsafe impl Send for SHA256state {}
unsafe impl Sync for SHA256state {}

#[derive(Debug)]
pub struct SHA384state {
    s: *mut Hacl_Streaming_SHA2_state_sha2_384,
}

impl SHA384state {
    pub fn new() -> SHA384state {
        SHA384state {
            s: std::ptr::null_mut(),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.s = Hacl_Streaming_SHA2_create_in_384();
        }
    }

    pub fn get_state(&mut self) -> *mut Hacl_Streaming_SHA2_state_sha2_384 {
        if self.s.is_null() {
            self.init();
        }
        self.s
    }
}

impl Drop for SHA384state {
    fn drop(&mut self) {
        if !self.s.is_null() {
            unsafe {
                Hacl_Streaming_SHA2_free_384(self.s);
            }
            self.s = std::ptr::null_mut();
        }
    }
}

unsafe impl Send for SHA384state {}
unsafe impl Sync for SHA384state {}

#[derive(Debug)]
pub struct SHA512state {
    s: *mut Hacl_Streaming_SHA2_state_sha2_512,
}

impl SHA512state {
    pub fn new() -> SHA512state {
        SHA512state {
            s: std::ptr::null_mut(),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.s = Hacl_Streaming_SHA2_create_in_512();
        }
    }

    pub fn get_state(&mut self) -> *mut Hacl_Streaming_SHA2_state_sha2_512 {
        if self.s.is_null() {
            self.init();
        }
        self.s
    }
}

impl Drop for SHA512state {
    fn drop(&mut self) {
        if !self.s.is_null() {
            unsafe {
                Hacl_Streaming_SHA2_free_512(self.s);
            }
            self.s = std::ptr::null_mut();
        }
    }
}

unsafe impl Send for SHA512state {}
unsafe impl Sync for SHA512state {}
