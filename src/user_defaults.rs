use anyhow::{anyhow, Context, Result};
use log::info;

use std::ffi::CString;
use std::fmt::Debug;

// Using core-foundation-rs might also be an option: https://github.com/servo/core-foundation-rs
//
// core-foundation-rs doesn't implement anything related to preferences at this time of writing, but it does have a handy way to create a CFString directly from a Rust string.
//
// I'm sure the technique we are using right now isn't as efficient since it goes through C and then Core Foundation, but it has the virtue of being simpler than core-foundation-rs. We're not worried about performance for our purposes.

mod sys {
    include!(concat!(env!("OUT_DIR"), "/defaults.rs"));
}

pub(crate) struct App {
    c_id: CString,
}

impl App {
    pub(crate) fn new<S: AsRef<str>>(id: S) -> Result<App> {
        to_cstring(id.as_ref()).map(|c_id| App { c_id: c_id })
    }

    pub(crate) fn bool(&self, key: &str, value: bool) -> Result<&App> {
        log_setting("boolean", key, value);
        let c_key = to_cstring(&key)?;
        unsafe { sys::defaults_set_bool(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    pub(crate) fn int(&self, key: &str, value: i64) -> Result<&App> {
        log_setting("integer", key, value);
        let c_key = to_cstring(&key)?;
        unsafe { sys::defaults_set_i64(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    pub(crate) fn float(&self, key: &str, value: f64) -> Result<&App> {
        log_setting("float", key, value);
        let c_key = to_cstring(&key)?;
        unsafe { sys::defaults_set_f64(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    pub(crate) fn string(&self, key: &str, value: &str) -> Result<&App> {
        log_setting("string", key, value);
        let c_key = to_cstring(&key)?;
        let c_value = to_cstring(&key)?;
        unsafe { sys::defaults_set_string(self.c_id.as_ptr(), c_key.as_ptr(), c_value.as_ptr()) }
        Ok(self)
    }

    // Would be nice to do this using Drop but it can fail and we want to propagate those failures
    pub(crate) fn sync(&self) -> Result<()> {
        // TODO logging
        let success = unsafe { sys::defaults_sync(self.c_id.as_ptr()) };
        if success {
            Ok(())
        } else {
            Err(anyhow!("Defaults synchronization failed"))
        }
    }
}

fn to_cstring(str: &str) -> Result<CString> {
    CString::new(str).with_context(|| format!("Converting string {:?} to CString", str))
}

fn log_setting<V: Debug>(type_: &str, key: &str, value: V) {
    info!(
        "Setting user defaults key {:?} to {} value {:?}",
        key, type_, value
    );
}
