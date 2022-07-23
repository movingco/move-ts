use anyhow::*;
use heck::ToSnakeCase;
use move_idl::{IDLModule, IDLPackage};

use crate::{idl_module::IDLModuleGenerator, CodeText, CodegenContext};

/// Generates the module re-exports for the given module names.
pub fn generate_module_reexports<'a, I>(prefix: &str, module_names: I) -> Result<CodeText>
where
    I: IntoIterator<Item = &'a String>,
{
    Ok(module_names
        .into_iter()
        .map(|name| {
            format!(
                "export * as {}_{} from \"./{}/index.js\";",
                prefix, name, name
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        .into())
}

pub struct IDLPackageGenerator<'info> {
    idl: &'info IDLPackage,
    pub modules_to_generate: Vec<IDLModule>,
    pub ctx: CodegenContext<'info>,
}

fn get_modules_to_generate(idl: &IDLPackage, with_dependencies: bool) -> Vec<IDLModule> {
    let mut modules = idl.modules.values().cloned().collect::<Vec<_>>();
    if with_dependencies {
        modules.append(&mut idl.dependencies.values().cloned().collect::<Vec<_>>());
    }
    modules
}

impl<'info> IDLPackageGenerator<'info> {
    pub fn new(idl: &'info IDLPackage, with_dependencies: bool) -> Self {
        IDLPackageGenerator {
            idl,
            modules_to_generate: get_modules_to_generate(idl, with_dependencies),
            ctx: CodegenContext::new(idl),
        }
    }

    pub fn generate_index(&self) -> Result<CodeText> {
        let prefix = &self.idl.name.to_snake_case();
        let index: CodeText = format!(
            "{}\n{}",
            generate_module_reexports(
                prefix,
                &self
                    .modules_to_generate
                    .iter()
                    .map(|m| m.module_id.name().to_string())
                    .collect::<Vec<_>>(),
            )?,
            CodeText::new_named_reexport(&format!("errmap as {}_errmap", prefix), "./errmap.js")
        )
        .into();

        Ok(index.module_docs(&format!(
            "This module contains generated types and helper functions for the package `{}`.",
            self.idl.name
        )))
    }

    pub fn generate_errmap_module(&self) -> Result<CodeText> {
        Ok(CodeText::new_const_export("errmap", &self.idl.errors)?
            .docs("All errors in this package.")
            .module_docs("Module containing all errors in this package."))
    }

    pub fn module_generators(&'info self) -> Vec<IDLModuleGenerator<'info>> {
        self.modules_to_generate
            .iter()
            .map(|m| self.ctx.get_module_generator(m))
            .collect::<Vec<_>>()
    }
}
