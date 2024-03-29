use anyhow::{anyhow, Context, Error, Result};

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> std::result::Result<(), Error> {
    // If we don't set this, bindgen will emit a warning. Things still seem to
    // work, but we may as well tell bindgen where llvm-config is. It doesn't
    // work to just set this var for rustc.
    std::env::set_var("LLVM_CONFIG_PATH", "/usr/local/opt/llvm/bin/llvm-config"); // Homebrew

    let macos_min_version = "10.10";
    let macos_min_version_flag = format!("-mmacosx-version-min={macos_min_version}");
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET={macos_min_version}");

    let ffi_dir = Path::new("src/ffi");
    let out_path = PathBuf::from(
        std::env::var("OUT_DIR").context("Could not get out directory from environment")?,
    );
    let process_input_file = |file_name: &str| -> Result<String> {
        let path = ffi_dir.join(file_name);
        let path_str = path.to_str_safe()?;
        rerun_if_changed(&path)?;
        Ok(path_str.to_owned())
    };

    bindgen::Builder::default()
        .header(process_input_file("user_defaults.h")?)
        .allowlist_function("user_defaults_.+")
        .generate()
        .map_err(|_| anyhow!("Could not generate bindings"))?
        .write_to_file(out_path.join("user_defaults.rs"))?;

    println!("cargo:rustc-link-lib=framework=CoreFoundation");

    cc::Build::new()
        .file(process_input_file("user_defaults.c")?)
        .flag(&macos_min_version_flag)
        .compile("user_defaults");

    {
        let scripts_dir = Path::new("scripts");
        rerun_if_changed(scripts_dir)?;
        let status = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(scripts_dir)
            .status()
            .context("Failed to build scoby scripts")?;

        if !status.success() {
            return Err(anyhow!("Failed to build scoby scripts"));
        }
    }

    Ok(())
}

fn rerun_if_changed(path: &Path) -> Result<()> {
    // Tell cargo to invalidate the built crate whenever the path changes
    println!("cargo:rerun-if-changed={}", path.to_str_safe()?);
    Ok(())
}

pub(crate) trait PathExt {
    fn to_str_safe(&self) -> Result<&str>;
}

impl PathExt for Path {
    fn to_str_safe(&self) -> Result<&str> {
        self.to_str()
            .ok_or_else(|| anyhow!("Error converting path {:?} to a string", self))
    }
}
