use anyhow::{anyhow, Context, Result};
use log::{debug, info};

use std::collections::HashMap;
use std::ffi::CString;
use std::fmt::Debug;

// Using core-foundation-rs might also be an option: https://github.com/servo/core-foundation-rs
//
// core-foundation-rs doesn't implement anything related to preferences at this time of writing, but it does have a handy way to create a CFString directly from a Rust string.
//
// I'm sure the technique we are using right now isn't as efficient since it goes through C and then Core Foundation, but it has the virtue of being simpler than core-foundation-rs. We're not worried about performance for our purposes.

mod sys {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/user_defaults.rs"));
}

#[derive(Debug)]
pub enum DictValue<'a> {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(&'a str),
}

pub struct App {
    id: String,
    c_id: CString,
}

impl App {
    pub fn new<S: AsRef<str>>(id: S) -> Result<App> {
        to_cstring(id.as_ref()).map(|c_id| {
            info!("Setting user defaults for application {:?}", id.as_ref());
            App {
                id: id.as_ref().to_owned(),
                c_id,
            }
        })
    }

    pub fn bool(&self, key: &str, value: bool) -> Result<&App> {
        self.log_setting("boolean", key, value);
        let c_key = to_cstring(key)?;
        unsafe { sys::user_defaults_set_bool(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    pub fn int(&self, key: &str, value: i64) -> Result<&App> {
        self.log_setting("integer", key, value);
        let c_key = to_cstring(key)?;
        unsafe { sys::user_defaults_set_i64(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    pub fn float(&self, key: &str, value: f64) -> Result<&App> {
        self.log_setting("float", key, value);
        let c_key = to_cstring(key)?;
        unsafe { sys::user_defaults_set_f64(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    pub fn string(&self, key: &str, value: &str) -> Result<&App> {
        self.log_setting("string", key, value);
        let c_key = to_cstring(key)?;
        let c_value = to_cstring(value)?;
        unsafe {
            sys::user_defaults_set_string(self.c_id.as_ptr(), c_key.as_ptr(), c_value.as_ptr())
        }
        Ok(self)
    }

    pub fn data(&self, key: &str, value: &[u8]) -> Result<&App> {
        self.log_setting("data", key, value); // TODO This doesn't look great; consider "buffer with length 123"
        let c_key = to_cstring(key)?;
        let size = i64::try_from(value.len()).context("Could not convert data length into i64")?;
        unsafe {
            sys::user_defaults_set_data(self.c_id.as_ptr(), c_key.as_ptr(), value.as_ptr(), size)
        }
        Ok(self)
    }

    pub fn dict(&self, key: &str, value: &HashMap<&str, DictValue>) -> Result<&App> {
        self.log_setting("dict", key, value); // TODO Audit this
        let c_key = to_cstring(key)?;

        // There are a number of strings we will be passing to C functions that we need to stay alive until the end of this function. This is SEPARATE from the CFStrings we will be allocating and releasing which are entirely confined to the C code.
        let mut keep_alive_cstrings = Vec::new();

        let cf_dict = unsafe { sys::user_defaults_dict_create() };
        for (dict_key, dict_value) in value {
            use DictValue::*;

            let dict_c_key = to_cstring(dict_key)?;
            match dict_value {
                Bool(bool_value) => unsafe {
                    sys::user_defaults_dict_set_bool_value(
                        cf_dict,
                        dict_c_key.as_ptr(),
                        *bool_value,
                    )
                },
                Int(i64_value) => unsafe {
                    sys::user_defaults_dict_set_i64_value(cf_dict, dict_c_key.as_ptr(), *i64_value)
                },
                Float(f64_value) => unsafe {
                    sys::user_defaults_dict_set_f64_value(cf_dict, dict_c_key.as_ptr(), *f64_value)
                },
                String(string_value) => {
                    let dict_c_value = to_cstring(string_value)?;
                    unsafe {
                        sys::user_defaults_dict_set_string_value(
                            cf_dict,
                            dict_c_key.as_ptr(),
                            dict_c_value.as_ptr(),
                        )
                    }
                    keep_alive_cstrings.push(dict_c_value);
                }
            }
            keep_alive_cstrings.push(dict_c_key);
        }
        unsafe {
            sys::user_defaults_set_dict(self.c_id.as_ptr(), c_key.as_ptr(), cf_dict);
            sys::user_defaults_dict_release(cf_dict);
        }
        Ok(self)
    }

    // Would be nice to do this using Drop but it can fail and we want to propagate those failures
    pub fn sync(&self) -> Result<()> {
        // TODO logging
        let success = unsafe { sys::user_defaults_sync(self.c_id.as_ptr()) };
        if success {
            Ok(())
        } else {
            Err(anyhow!("Defaults synchronization failed"))
        }
    }

    fn log_setting<V: Debug>(&self, type_: &str, key: &str, value: V) {
        debug!(
            "Setting application with ID {:?} user defaults key {:?} to {} value {:?}",
            self.id, key, type_, value
        );
        info!("{:?} → {:?}", key, value);
    }
}

fn to_cstring(s: &str) -> Result<CString> {
    CString::new(s).with_context(|| format!("Converting string {s:?} to CString"))
}
