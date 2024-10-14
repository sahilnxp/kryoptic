// Copyright 2024 Simo Sorce
// See LICENSE.txt file for terms

use crate::attribute::{string_to_ck_date, AttrType, Attribute};
use crate::error::{Error, Result};
use crate::interface::*;
use crate::object::Object;
use crate::storage::{memory, Storage};

use data_encoding::BASE64;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_string_pretty, Map, Number, Value};

fn to_json_value(a: &Attribute) -> Value {
    match a.get_attrtype() {
        AttrType::BoolType => match a.to_bool() {
            Ok(b) => Value::Bool(b),
            Err(_) => Value::Null,
        },
        AttrType::NumType => match a.to_ulong() {
            Ok(l) => Value::Number(Number::from(l)),
            Err(_) => Value::Null,
        },
        AttrType::StringType => match a.to_string() {
            Ok(s) => Value::String(s),
            Err(_) => Value::String(BASE64.encode(a.get_value())),
        },
        AttrType::BytesType => Value::String(BASE64.encode(a.get_value())),
        AttrType::DateType => match a.to_date_string() {
            Ok(d) => Value::String(d),
            Err(_) => Value::String(String::new()),
        },
        AttrType::IgnoreType => Value::Null,
        AttrType::DenyType => Value::Null,
    }
}

fn uninit(e: std::io::Error) -> Error {
    if e.kind() == std::io::ErrorKind::NotFound {
        Error::ck_rv_from_error(CKR_CRYPTOKI_NOT_INITIALIZED, e)
    } else {
        Error::other_error(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonObject {
    attributes: Map<String, Value>,
}

impl JsonObject {
    pub fn from_object(o: &Object) -> JsonObject {
        let mut jo = JsonObject {
            attributes: Map::new(),
        };
        for a in o.get_attributes() {
            jo.attributes.insert(a.name(), to_json_value(a));
        }
        jo
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonToken {
    objects: Vec<JsonObject>,
}

impl JsonToken {
    pub fn load(filename: &str) -> Result<JsonToken> {
        match std::fs::File::open(filename) {
            Ok(f) => Ok(from_reader::<std::fs::File, JsonToken>(f)?),
            Err(e) => Err(uninit(e)),
        }
    }

    pub fn prime_cache(&self, cache: &mut Box<dyn Storage>) -> Result<()> {
        for jo in &self.objects {
            let mut obj = Object::new();
            let mut uid: Option<String> = None;
            for (key, val) in &jo.attributes {
                let (id, atype) = AttrType::attr_name_to_id_type(key)?;
                let attr = match atype {
                    AttrType::BoolType => match val.as_bool() {
                        Some(b) => Attribute::from_bool(id, b),
                        None => return Err(CKR_ATTRIBUTE_VALUE_INVALID)?,
                    },
                    AttrType::NumType => match val.as_u64() {
                        Some(n) => Attribute::from_u64(id, n),
                        None => return Err(CKR_ATTRIBUTE_VALUE_INVALID)?,
                    },
                    AttrType::StringType => match val.as_str() {
                        Some(s) => Attribute::from_string(id, s.to_string()),
                        None => return Err(CKR_ATTRIBUTE_VALUE_INVALID)?,
                    },
                    AttrType::BytesType => match val.as_str() {
                        Some(s) => {
                            let len = match BASE64.decode_len(s.len()) {
                                Ok(l) => l,
                                Err(_) => return Err(CKR_GENERAL_ERROR)?,
                            };
                            let mut v = vec![0; len];
                            match BASE64.decode_mut(s.as_bytes(), &mut v) {
                                Ok(l) => {
                                    Attribute::from_bytes(id, v[0..l].to_vec())
                                }
                                Err(_) => return Err(CKR_GENERAL_ERROR)?,
                            }
                        }
                        None => return Err(CKR_ATTRIBUTE_VALUE_INVALID)?,
                    },
                    AttrType::DateType => match val.as_str() {
                        Some(s) => {
                            if s.len() == 0 {
                                /* special case for default empty value */
                                Attribute::from_date_bytes(id, Vec::new())
                            } else {
                                Attribute::from_date(id, string_to_ck_date(&s)?)
                            }
                        }
                        None => return Err(CKR_ATTRIBUTE_VALUE_INVALID)?,
                    },
                    AttrType::DenyType => continue,
                    AttrType::IgnoreType => continue,
                };

                obj.set_attr(attr)?;
                if key == "CKA_UNIQUE_ID" {
                    uid = match val.as_str() {
                        Some(s) => Some(s.to_string()),
                        None => return Err(CKR_DEVICE_ERROR)?,
                    }
                }
            }
            match uid {
                Some(u) => cache.store(&u, obj)?,
                None => return Err(CKR_DEVICE_ERROR)?,
            }
        }
        Ok(())
    }

    pub fn from_cache(cache: &mut Box<dyn Storage>) -> JsonToken {
        let objs = cache.search(&[]).unwrap();
        let mut jt = JsonToken {
            objects: Vec::with_capacity(objs.len()),
        };
        for o in objs {
            if !o.is_token() {
                continue;
            }
            jt.objects.push(JsonObject::from_object(&o));
        }

        jt
    }

    pub fn save(&self, filename: &str) -> Result<()> {
        let jstr = match to_string_pretty(&self) {
            Ok(j) => j,
            Err(e) => return Err(Error::other_error(e)),
        };
        match std::fs::write(filename, jstr) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::other_error(e)),
        }
    }
}

#[derive(Debug)]
pub struct JsonStorage {
    filename: String,
    cache: Box<dyn Storage>,
}

impl Storage for JsonStorage {
    fn open(&mut self, filename: &String) -> Result<()> {
        self.filename = filename.clone();
        let token = JsonToken::load(&self.filename)?;
        token.prime_cache(&mut self.cache)
    }
    fn reinit(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }
    fn flush(&mut self) -> Result<()> {
        let token = JsonToken::from_cache(&mut self.cache);
        token.save(&self.filename)
    }
    fn fetch_by_uid(&self, uid: &String) -> Result<Object> {
        self.cache.fetch_by_uid(uid)
    }
    fn store(&mut self, uid: &String, obj: Object) -> Result<()> {
        self.cache.store(uid, obj)?;
        self.flush()
    }
    fn search(&self, template: &[CK_ATTRIBUTE]) -> Result<Vec<Object>> {
        self.cache.search(template)
    }
    fn remove_by_uid(&mut self, uid: &String) -> Result<()> {
        self.cache.remove_by_uid(uid)?;
        self.flush()
    }
}

pub fn json() -> Box<dyn Storage> {
    Box::new(JsonStorage {
        filename: String::from(""),
        cache: memory::memory(),
    })
}
