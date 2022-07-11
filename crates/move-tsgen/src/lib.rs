//! CLI for parsing an IDL from a Move package.
use std::{collections::BTreeMap, path::PathBuf};

use anyhow::*;
use json_cli::{CliTool, CliTypedResult};
use move_idl::IDLBuilder;
use move_package::BuildConfig;
use move_ts::{idl_package::IDLPackageGenerator, Codegen};

/// Parses a Move workspace into a set of IDLs.
#[derive(clap::Parser)]
#[clap(name = "move-tsgen", author, version)]
pub struct MoveTSGenTool {
    /// Path to the root of the Move workspace.
    #[clap(default_value = ".")]
    pub root: PathBuf,
    /// Output directory for the generated files.
    #[clap(short, long, default_value = "./build/ts/")]
    pub out_dir: PathBuf,

    /// Whether to generate module TypeScript files for dependencies.
    #[clap(short, long)]
    pub with_dependencies: bool,
}

#[async_trait::async_trait]
impl CliTool<()> for MoveTSGenTool {
    async fn execute(self) -> CliTypedResult<()> {
        let mut additional_named_addresses = BTreeMap::new();
        additional_named_addresses
            .insert("Std".to_string(), static_address::static_address!("0x1"));
        let build_config_std = BuildConfig {
            generate_docs: true,
            generate_abis: true,
            additional_named_addresses,
            ..Default::default()
        };
        let idl = IDLBuilder::load_with_config(&self.root, build_config_std)?.gen()?;

        std::fs::create_dir_all(&self.out_dir)?;

        let package_gen = IDLPackageGenerator::new(&idl, self.with_dependencies);
        for gen in package_gen.module_generators() {
            let module_dir = &self.out_dir.join(gen.0.module_id.name());
            std::fs::create_dir_all(module_dir)?;

            if gen.has_entrypoints() {
                std::fs::write(
                    module_dir.join("entry").with_extension("ts"),
                    gen.generate_entrypoint_module(&package_gen.ctx)?,
                )?;
            }

            std::fs::write(
                module_dir.join("idl").with_extension("ts"),
                gen.generate_idl_module()?,
            )?;

            if let Some(errors_module) = gen.generate_errors_module()? {
                std::fs::write(
                    module_dir.join("errors").with_extension("ts"),
                    errors_module,
                )?;
            }

            let ts = gen.0.generate_typescript(&package_gen.ctx)?;
            std::fs::write(module_dir.join("index").with_extension("ts"), ts)?;
        }

        std::fs::write(
            self.out_dir.join("errmap").with_extension("ts"),
            &package_gen.generate_errmap_module()?,
        )?;

        std::fs::write(
            self.out_dir.join("index").with_extension("ts"),
            package_gen.generate_index()?,
        )?;

        Ok(())
    }
}
