// Copyright 2024 Simo Sorce
// See LICENSE.txt file for terms

use super::tests;
use tests::*;

use std::env;

fn test_token(name: &str) {
    let mut testdata = TestData::new(name);
    testdata.setup_db(None);

    let mut plist: *mut CK_FUNCTION_LIST = std::ptr::null_mut();
    let pplist = &mut plist;
    let result = C_GetFunctionList(&mut *pplist);
    assert_eq!(result, 0);
    unsafe {
        let list: CK_FUNCTION_LIST = *plist;
        match list.C_Initialize {
            Some(value) => {
                let mut args = testdata.make_init_args();
                let args_ptr = &mut args as *mut CK_C_INITIALIZE_ARGS;
                let ret = value(args_ptr as *mut std::ffi::c_void);
                assert_eq!(ret, CKR_OK)
            }
            None => todo!(),
        }
    }

    testdata.finalize();
}

fn test_token_env(name: &str) {
    let mut testdata = TestData::new(name);
    testdata.setup_db(None);

    let mut plist: *mut CK_FUNCTION_LIST = std::ptr::null_mut();
    let pplist = &mut plist;
    let result = C_GetFunctionList(&mut *pplist);
    assert_eq!(result, 0);
    unsafe {
        let list: CK_FUNCTION_LIST = *plist;
        match list.C_Initialize {
            Some(init_fn) => {
                let mut args = testdata.make_empty_init_args();
                let args_ptr = &mut args as *mut CK_C_INITIALIZE_ARGS;
                env::set_var("KRYOPTIC_CONF", testdata.make_init_string());
                let ret = init_fn(args_ptr as *mut std::ffi::c_void);
                assert_eq!(ret, CKR_OK)
            }
            None => todo!(),
        }
    }

    testdata.finalize();
}

fn test_token_null_args(name: &str) {
    let mut testdata = TestData::new(name);
    testdata.setup_db(None);

    let mut plist: *mut CK_FUNCTION_LIST = std::ptr::null_mut();
    let pplist = &mut plist;
    let result = C_GetFunctionList(&mut *pplist);
    assert_eq!(result, 0);
    unsafe {
        let list: CK_FUNCTION_LIST = *plist;
        match list.C_Initialize {
            Some(init_fn) => {
                env::set_var("KRYOPTIC_CONF", testdata.make_init_string());
                let ret = init_fn(std::ptr::null_mut());
                assert_eq!(ret, CKR_OK)
            }
            None => todo!(),
        }
    }

    testdata.finalize();
}

#[test]
fn test_token_json() {
    test_token("test_token.json");
    test_token_env("test_token.json");
    test_token_null_args("test_token.json");
}

#[test]
fn test_token_sql() {
    test_token("test_token.sql");
    test_token_env("test_token.sql");
    test_token_null_args("test_token.sql");
}
