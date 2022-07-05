//! CLI for parsing an IDL from a Move package.
use std::path::PathBuf;

use anyhow::*;
use json_cli::{CliTool, CliTypedResult};
use move_idl::IDLBuilder;
use move_ts::{generate_index, CodegenContext};

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
        let idl = IDLBuilder::load(&self.root)?.gen()?;

        std::fs::create_dir_all(&self.out_dir)?;

        let relevant_modules = if self.with_dependencies {
            let mut idl_mut = idl.clone();
            let mut modules = idl_mut.modules;
            modules.append(&mut idl_mut.dependencies);
            modules
        } else {
            idl.clone().modules
        };

        let ctx = CodegenContext::new(&idl);
        for (name, module_idl) in relevant_modules.iter() {
            let ts = ctx.generate(module_idl)?;
            std::fs::write(self.out_dir.join(name).with_extension("ts"), ts.to_string())?;
        }

        std::fs::write(
            self.out_dir.join("index").with_extension("ts"),
            generate_index(
                &relevant_modules
                    .into_iter()
                    .map(|(name, _)| name)
                    .collect::<Vec<_>>(),
            )?
            .to_string(),
        )?;

        Ok(())
    }
}
