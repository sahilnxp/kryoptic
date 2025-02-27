// Copyright 2024 Simo Sorce
// See LICENSE.txt file for terms

use crate::tests::*;

use serial_test::parallel;

#[test]
#[parallel]
fn test_random() {
    let mut testtokn = TestToken::initialized("test_random.sql", None);
    let session = testtokn.get_session(false);

    let data: &[u8] = &mut [0, 0, 0, 0];
    let ret = fn_generate_random(
        session,
        data.as_ptr() as *mut u8,
        data.len() as CK_ULONG,
    );
    assert_eq!(ret, CKR_OK);
    assert_ne!(data, &[0, 0, 0, 0]);

    testtokn.finalize();
}
