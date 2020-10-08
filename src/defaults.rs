use anyhow::{anyhow, Context, Result};
use std::ffi::CString;

// Using core-foundation-rs might also be an option: https://github.com/servo/core-foundation-rs
//
// core-foundation-rs doesn't implement anything related to preferences at this time of writing, but it does have a handy way to create a CFString directly from a Rust string.
//
// I'm sure the technique we are using right now isn't as efficient since it goes through C and then Core Foundation, but it has the virtue of being simpler than core-foundation-rs. We're not worried about performance for our purposes.

mod sys {
    include!(concat!(env!("OUT_DIR"), "/defaults.rs"));
}

pub(crate) struct Application {
    c_id: CString,
}

impl Application {
    pub(crate) fn new<S: AsRef<str>>(id: S) -> Result<Application> {
        to_cstring(id.as_ref()).map(|c_id| Application { c_id: c_id })
    }

    pub(crate) fn bool(&self, key: &str, value: bool) -> Result<&Application> {
        // TODO logging
        let c_key = to_cstring(&key)?;
        unsafe { sys::defaults_set_bool(self.c_id.as_ptr(), c_key.as_ptr(), value) }
        Ok(self)
    }

    // TODO float, string, int

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
