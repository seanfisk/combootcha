use anyhow::{Error, Result};
use std::env::{var, VarError};

pub(crate) fn get(name: &str) -> Result<Option<String>> {
    match var(name) {
        Ok(v) => Ok(Some(v)),
        Err(e) => match e {
            VarError::NotPresent => Ok(None),
            VarError::NotUnicode(_) => Err(Error::new(e)),
        },
    }
}
