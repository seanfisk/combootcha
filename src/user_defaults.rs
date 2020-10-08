use anyhow::{anyhow, Context, Result};
use log::{debug, info};

use std::ffi::CString;
use std::fmt::{Debug, Display};

// Using core-foundation-rs might also be an option: https://github.com/servo/core-foundation-rs
//
// core-foundation-rs doesn't implement anything related to preferences at this time of writing, but it does have a handy way to create a CFString directly from a Rust string.
//
// I'm sure the technique we are using right now isn't as efficient since it goes through C and then Core Foundation, but it has the virtue of being simpler than core-foundation-rs. We're not worried about performance for our purposes.

mod sys {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/user_defaults.rs"));
}

pub(crate) struct App {
    id: String,
    c_id: CString,
}

impl App {
    pub(crate) fn new<S: AsRef<str>>(id: S) -> Result<App> {
        to_cstring(id.as_ref()).map(|c_id| {
            info!("Setting user defaults for application {:?}", id.as_ref());
            App {
                id: id.as_ref().to_owned(),
                c_id: c_id,
            }
        })
    }

    pub(crate) fn bool(&self, key: &str, value: bool) -> Result<&App> {
        self.log_setting("boolean", key, value);
        let c_key = to_cstring(&key)?;
        unsafe { sys::user_defaults_set_bool(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    pub(crate) fn int(&self, key: &str, value: i64) -> Result<&App> {
        self.log_setting("integer", key, value);
        let c_key = to_cstring(&key)?;
        unsafe { sys::user_defaults_set_i64(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    pub(crate) fn float(&self, key: &str, value: f64) -> Result<&App> {
        self.log_setting("float", key, value);
        let c_key = to_cstring(&key)?;
        unsafe { sys::user_defaults_set_f64(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    pub(crate) fn string(&self, key: &str, value: &str) -> Result<&App> {
        self.log_setting("string", key, value);
        let c_key = to_cstring(&key)?;
        let c_value = to_cstring(&value)?;
        unsafe {
            sys::user_defaults_set_string(self.c_id.as_ptr(), c_key.as_ptr(), c_value.as_ptr())
        }
        Ok(self)
    }

    // Would be nice to do this using Drop but it can fail and we want to propagate those failures
    pub(crate) fn sync(&self) -> Result<()> {
        // TODO logging
        let success = unsafe { sys::user_defaults_sync(self.c_id.as_ptr()) };
        if success {
            Ok(())
        } else {
            Err(anyhow!("Defaults synchronization failed"))
        }
    }

    fn log_setting<V: Debug + Display>(&self, type_: &str, key: &str, value: V) {
        debug!(
            "Setting application with ID {:?} user defaults key {:?} to {} value {:?}",
            self.id, key, type_, value
        );
        info!("{:?} => {:?}", key, value);
    }
}

fn to_cstring(str: &str) -> Result<CString> {
    CString::new(str).with_context(|| format!("Converting string {:?} to CString", str))
}
