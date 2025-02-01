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

    // Since macOS 10.14, headers are no longer included in
    // /System/Library/Frameworks, which is where libclang (and therefore
    // bindgen) will look for headers. Determine the correct SDK path and pass
    // it to bindgen.
    // Credit: https://zameermanji.com/blog/2021/7/13/using-bindgen-with-system-frameworks-on-macos/
    // My investigation: https://github.com/seanfisk/combootcha/issues/55
    // Also see: https://github.com/rust-lang/rust-bindgen/issues/1226
    let macos_sdk_path = {
        let context = "Failed to determine macOS SDK path";
        let output = Command::new("/usr/bin/xcrun")
            .args(["--sdk", "macosx", "--show-sdk-path"])
            .output()
            .context(context)?;
        if !output.status.success() {
            return Err(anyhow!(context));
        }
        std::str::from_utf8(&output.stdout)
            .context("Could not parse macOS SDK path as UTF-8")?
            .trim_end()
            .to_owned()
    };

    bindgen::Builder::default()
        .header(process_input_file("user_defaults.h")?)
        .allowlist_function("user_defaults_.+")
        .clang_arg("-isysroot")
        .clang_arg(macos_sdk_path)
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
