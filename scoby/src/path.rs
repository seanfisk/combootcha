use anyhow::{anyhow, Result};
use std::path::Path;

pub trait Ext {
    fn to_str_safe(&self) -> Result<&str>;
}

impl Ext for Path {
    fn to_str_safe(&self) -> Result<&str> {
        self.to_str()
            .ok_or_else(|| anyhow!("Error converting path {:?} to a string", self))
    }
}
