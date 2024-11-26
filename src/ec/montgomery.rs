// Copyright 2024 Jakub Jelen
// See LICENSE.txt file for terms

use std::fmt::Debug;

use crate::attr_element;
use crate::attribute::Attribute;
use crate::ec::montgomery::montgomery::ECMontgomeryOperation;
use crate::ec::*;
use crate::error::{general_error, Error, Result};
use crate::kasn1::oid;
use crate::mechanism::*;
use crate::object::*;
use crate::ossl::montgomery;

use once_cell::sync::Lazy;

pub const MIN_EC_MONTGOMERY_SIZE_BITS: usize = BITS_X25519;
pub const MAX_EC_MONTGOMERY_SIZE_BITS: usize = BITS_X448;

#[derive(Debug)]
pub struct ECMontgomeryPubFactory {
    attributes: Vec<ObjectAttr>,
}

impl ECMontgomeryPubFactory {
    pub fn new() -> ECMontgomeryPubFactory {
        let mut data: ECMontgomeryPubFactory = ECMontgomeryPubFactory {
            attributes: Vec::new(),
        };
        data.attributes.append(&mut data.init_common_object_attrs());
        data.attributes
            .append(&mut data.init_common_storage_attrs());
        data.attributes.append(&mut data.init_common_key_attrs());
        data.attributes
            .append(&mut data.init_common_public_key_attrs());
        data.attributes.push(attr_element!(
            CKA_EC_PARAMS; OAFlags::AlwaysRequired | OAFlags::Unchangeable;
            Attribute::from_bytes; val Vec::new()));
        data.attributes.push(attr_element!(
            CKA_EC_POINT; OAFlags::RequiredOnCreate
            | OAFlags::SettableOnlyOnCreate | OAFlags::Unchangeable;
            Attribute::from_bytes; val Vec::new()));
        data
    }
}

impl ObjectFactory for ECMontgomeryPubFactory {
    fn create(&self, template: &[CK_ATTRIBUTE]) -> Result<Object> {
        let obj = self.default_object_create(template)?;

        /* According to PKCS#11 v3.1 6.3.7:
         * CKA_EC_PARAMS, Byte array,
         * DER-encoding of a Parameters value as defined above (6.3.3?) */
        let oid = get_oid_from_obj(&obj).map_err(|e| {
            if e.attr_not_found() {
                Error::ck_rv_from_error(CKR_TEMPLATE_INCOMPLETE, e)
            } else if e.rv() != CKR_ATTRIBUTE_VALUE_INVALID {
                Error::ck_rv_from_error(CKR_ATTRIBUTE_VALUE_INVALID, e)
            } else {
                general_error(e)
            }
        })?;
        match oid {
            oid::X25519_OID | oid::X448_OID => (),
            _ => return Err(CKR_ATTRIBUTE_VALUE_INVALID)?,
        }

        /* According to PKCS#11 v3.1 6.3.7:
         * CKA_EC_POINT, Byte array,
         * Public key bytes in little endian order as defined in RFC 7748 */
        let point = get_ec_point_from_obj(&obj).map_err(|e| {
            if e.attr_not_found() {
                Error::ck_rv_from_error(CKR_TEMPLATE_INCOMPLETE, e)
            } else if e.rv() != CKR_ATTRIBUTE_VALUE_INVALID {
                Error::ck_rv_from_error(CKR_ATTRIBUTE_VALUE_INVALID, e)
            } else {
                general_error(e)
            }
        })?;
        if point.len() != ec_point_size(&oid)? {
            return Err(CKR_ATTRIBUTE_VALUE_INVALID)?;
        }

        Ok(obj)
    }

    fn get_attributes(&self) -> &Vec<ObjectAttr> {
        &self.attributes
    }
}

impl CommonKeyFactory for ECMontgomeryPubFactory {}

impl PubKeyFactory for ECMontgomeryPubFactory {}

#[derive(Debug)]
pub struct ECMontgomeryPrivFactory {
    attributes: Vec<ObjectAttr>,
}

impl ECMontgomeryPrivFactory {
    pub fn new() -> ECMontgomeryPrivFactory {
        let mut data: ECMontgomeryPrivFactory = ECMontgomeryPrivFactory {
            attributes: Vec::new(),
        };
        data.attributes.append(&mut data.init_common_object_attrs());
        data.attributes
            .append(&mut data.init_common_storage_attrs());
        data.attributes.append(&mut data.init_common_key_attrs());
        data.attributes
            .append(&mut data.init_common_private_key_attrs());
        data.attributes.push(attr_element!(
            CKA_EC_PARAMS; OAFlags::RequiredOnCreate | OAFlags::Unchangeable;
            Attribute::from_bytes; val Vec::new()));
        data.attributes.push(attr_element!(
            CKA_VALUE; OAFlags::Sensitive | OAFlags::RequiredOnCreate
            | OAFlags::SettableOnlyOnCreate | OAFlags::Unchangeable;
            Attribute::from_bytes; val Vec::new()));

        /* default to private */
        let private = attr_element!(
            CKA_PRIVATE; OAFlags::Defval | OAFlags::ChangeOnCopy;
            Attribute::from_bool; val true);
        match data
            .attributes
            .iter()
            .position(|x| x.get_type() == CKA_PRIVATE)
        {
            Some(idx) => data.attributes[idx] = private,
            None => data.attributes.push(private),
        }

        data
    }
}

impl ObjectFactory for ECMontgomeryPrivFactory {
    fn create(&self, template: &[CK_ATTRIBUTE]) -> Result<Object> {
        let mut obj = self.default_object_create(template)?;

        ec_key_check_import(&mut obj)?;

        Ok(obj)
    }

    fn get_attributes(&self) -> &Vec<ObjectAttr> {
        &self.attributes
    }

    fn export_for_wrapping(&self, key: &Object) -> Result<Vec<u8>> {
        PrivKeyFactory::export_for_wrapping(self, key)
    }

    fn import_from_wrapped(
        &self,
        data: Vec<u8>,
        template: &[CK_ATTRIBUTE],
    ) -> Result<Object> {
        PrivKeyFactory::import_from_wrapped(self, data, template)
    }
}

impl CommonKeyFactory for ECMontgomeryPrivFactory {}

impl PrivKeyFactory for ECMontgomeryPrivFactory {}

static PUBLIC_KEY_FACTORY: Lazy<Box<dyn ObjectFactory>> =
    Lazy::new(|| Box::new(ECMontgomeryPubFactory::new()));

static PRIVATE_KEY_FACTORY: Lazy<Box<dyn ObjectFactory>> =
    Lazy::new(|| Box::new(ECMontgomeryPrivFactory::new()));

#[derive(Debug)]
struct ECMontgomeryMechanism {
    info: CK_MECHANISM_INFO,
}

impl ECMontgomeryMechanism {
    fn register_mechanisms(mechs: &mut Mechanisms) {
        /* TODO PKCS #11 defines also CKM_XEDDSA for signatures, but it is not implemented by
         * OpenSSL */
        mechs.add_mechanism(
            CKM_EC_MONTGOMERY_KEY_PAIR_GEN,
            Box::new(ECMontgomeryMechanism {
                info: CK_MECHANISM_INFO {
                    ulMinKeySize: CK_ULONG::try_from(
                        MIN_EC_MONTGOMERY_SIZE_BITS,
                    )
                    .unwrap(),
                    ulMaxKeySize: CK_ULONG::try_from(
                        MAX_EC_MONTGOMERY_SIZE_BITS,
                    )
                    .unwrap(),
                    flags: CKF_GENERATE_KEY_PAIR,
                },
            }),
        );
    }
}

impl Mechanism for ECMontgomeryMechanism {
    fn info(&self) -> &CK_MECHANISM_INFO {
        &self.info
    }

    fn generate_keypair(
        &self,
        mech: &CK_MECHANISM,
        pubkey_template: &[CK_ATTRIBUTE],
        prikey_template: &[CK_ATTRIBUTE],
    ) -> Result<(Object, Object)> {
        let mut pubkey =
            PUBLIC_KEY_FACTORY.default_object_generate(pubkey_template)?;
        if !pubkey.check_or_set_attr(Attribute::from_ulong(
            CKA_CLASS,
            CKO_PUBLIC_KEY,
        ))? {
            return Err(CKR_TEMPLATE_INCONSISTENT)?;
        }
        if !pubkey.check_or_set_attr(Attribute::from_ulong(
            CKA_KEY_TYPE,
            CKK_EC_EDWARDS,
        ))? {
            return Err(CKR_TEMPLATE_INCONSISTENT)?;
        }

        let mut privkey =
            PRIVATE_KEY_FACTORY.default_object_generate(prikey_template)?;
        if !privkey.check_or_set_attr(Attribute::from_ulong(
            CKA_CLASS,
            CKO_PRIVATE_KEY,
        ))? {
            return Err(CKR_TEMPLATE_INCONSISTENT)?;
        }
        if !privkey.check_or_set_attr(Attribute::from_ulong(
            CKA_KEY_TYPE,
            CKK_EC_EDWARDS,
        ))? {
            return Err(CKR_TEMPLATE_INCONSISTENT)?;
        }

        let ec_params = match pubkey.get_attr_as_bytes(CKA_EC_PARAMS) {
            Ok(a) => a.clone(),
            Err(_) => {
                return Err(CKR_ATTRIBUTE_VALUE_INVALID)?;
            }
        };
        if !privkey.check_or_set_attr(Attribute::from_bytes(
            CKA_EC_PARAMS,
            ec_params,
        ))? {
            return Err(CKR_TEMPLATE_INCONSISTENT)?;
        }

        ECMontgomeryOperation::generate_keypair(&mut pubkey, &mut privkey)?;
        default_key_attributes(&mut privkey, mech.mechanism)?;
        default_key_attributes(&mut pubkey, mech.mechanism)?;

        Ok((pubkey, privkey))
    }
}

pub fn register(mechs: &mut Mechanisms, ot: &mut ObjectFactories) {
    ECMontgomeryMechanism::register_mechanisms(mechs);

    ot.add_factory(
        ObjectType::new(CKO_PUBLIC_KEY, CKK_EC_MONTGOMERY),
        &PUBLIC_KEY_FACTORY,
    );
    ot.add_factory(
        ObjectType::new(CKO_PRIVATE_KEY, CKK_EC_MONTGOMERY),
        &PRIVATE_KEY_FACTORY,
    );
}
