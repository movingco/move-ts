//! Generates TypeScript based on a Move IDL.

pub mod format;
pub mod generate;

use anyhow::Result;
use generate::package::PackageCodegen;
use move_idl::IDLBuilder;
use std::path::{Path, PathBuf};

pub fn generate_types(package_root: &Path) -> Result<()> {
    std::env::set_current_dir(package_root)?;

    let root = PathBuf::from(".");

    let builder = IDLBuilder::load(&root)?;
    let idl = &builder.gen()?;

    std::fs::create_dir_all(root.join("generated"))?;

    let codegen = PackageCodegen::new(idl);
    codegen.gen_package(&root.join("generated"), true)?;

    std::fs::write(
        root.join("generated/idl.json"),
        serde_json::to_string_pretty(idl)?,
    )?;

    Ok(())
}
