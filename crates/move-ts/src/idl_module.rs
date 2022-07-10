use crate::{
    format::{gen_doc_string, gen_doc_string_opt},
    CodeText,
};

use super::{script_function::ScriptFunctionType, Codegen, CodegenContext};
use anyhow::*;
use move_idl::{IDLAbility, IDLError, IDLModule, IDLScriptFunction};
use serde::Serialize;
use std::collections::BTreeMap;

const PRELUDE: &str = "import * as p from \"@movingco/prelude\";\n";

#[derive(Serialize)]
struct ErrorInfo {
    /// Error code.
    code: u64,
    /// Error.
    #[serde(flatten)]
    error: IDLError,
}

pub struct IDLModuleGenerator<'info>(&'info IDLModule);

impl<'info> IDLModuleGenerator<'info> {
    pub fn new(module: &'info IDLModule) -> Self {
        IDLModuleGenerator(module)
    }

    fn generate_module_header(&self) -> String {
        format!("**Module ID:** `{}`", self.0.module_id)
    }

    pub fn generate_module_doc(&self) -> String {
        gen_doc_string(
            &vec![
                self.0.doc.clone().unwrap_or_default(),
                self.generate_module_header(),
                "@module".to_string(),
            ]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
        )
    }

    fn get_script_fns(&self) -> Vec<ScriptFunctionType> {
        self.0
            .functions
            .iter()
            .map(|script_fn| ScriptFunctionType::new(self.0, script_fn))
            .collect::<Vec<_>>()
    }

    pub fn has_entrypoints(&self) -> bool {
        !self.0.functions.is_empty()
    }

    pub fn generate_entrypoint_bodies(&self, ctx: &CodegenContext) -> Result<CodeText> {
        Ok(ctx.try_join(&self.get_script_fns())?.indent())
    }

    pub fn generate_entrypoint_module(&self, ctx: &CodegenContext) -> Result<CodeText> {
        Ok(format!(
            "{}{}\nimport * as mod from './index.js';\n{}",
            gen_doc_string("Entrypoint builders.\n\n@module"),
            PRELUDE,
            self.generate_entrypoint_bodies(ctx)?
        )
        .into())
    }

    pub fn generate_idl_module(&self) -> Result<CodeText> {
        Ok(format!(
            "{}export const idl = {} as const;",
            gen_doc_string("The IDL of the module.\n\n@module"),
            &serde_json::to_string(self.0)?,
        )
        .into())
    }

    pub fn generate_entrypoints(&self, ctx: &CodegenContext) -> Result<String> {
        Ok(format!(
            "{}export const entrypoints = {{\n{}\n}};",
            gen_doc_string("Entrypoint builders."),
            self.generate_entrypoint_bodies(ctx)?
        ))
    }
}

impl Codegen for IDLModule {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        let gen = IDLModuleGenerator(self);
        let name = self.module_id.name();

        let script_fns = self
            .functions
            .iter()
            .map(|script_fn| ScriptFunctionType::new(self, script_fn))
            .collect::<Vec<_>>();

        let function_payloads = ctx.try_join(
            &script_fns
                .iter()
                .filter_map(|f| {
                    if f.should_render_payload_struct() {
                        Some(f.payload())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        )?;

        let struct_types = ctx.try_join(&self.structs)?;

        let mut error_map: BTreeMap<String, ErrorInfo> = BTreeMap::new();
        for (code, error) in self.errors.iter() {
            error_map.insert(
                error.name.clone(),
                ErrorInfo {
                    code: *code,
                    error: error.clone(),
                },
            );
        }

        let mut fn_map: BTreeMap<String, IDLScriptFunction> = BTreeMap::new();
        for script_fn in self.functions.iter() {
            fn_map.insert(script_fn.name.clone(), script_fn.clone());
        }

        let mut resources: BTreeMap<String, String> = BTreeMap::new();
        for struct_info in self
            .structs
            .iter()
            .filter(|s| s.abilities.contains(&IDLAbility::Key))
        {
            resources.insert(
                struct_info.name.name.to_string(),
                struct_info.name.to_string(),
            );
        }

        let mut structs: BTreeMap<String, String> = BTreeMap::new();
        for struct_info in self.structs.iter() {
            structs.insert(
                struct_info.name.name.to_string(),
                struct_info.name.to_string(),
            );
        }

        let ts = format!(
            r#"{}{}

{}

{}

{}

export {{ idl }} from "./idl.js";

/** Module ID information. */
export const id = {{
  /** The address of the module. */
  ADDRESS: "{}",
  /** The full module name. */
  FULL_NAME: "{}",
  /** The name of the module. */
  NAME: "{}"
}} as const;

/** Module errors. */
export const errors = {} as const;

/** Module error codes. */
export const errorCodes = {} as const;

/** All module function IDLs. */
export const functions = {} as const;

/** All struct types with ability `key`. */
export const resources = {} as const;

/** All struct types. */
export const structs = {} as const;

/** Payload generators for module `{}`. */
const moduleImpl = {{
  ...id,
  errors,
  errorCodes,
  functions,
  resources,
  structs,
}} as const;

{}export const moduleDefinition = moduleImpl as p.MoveModuleDefinition<"{}", "{}"> as typeof moduleImpl;
"#,
            gen.generate_module_doc(),
            PRELUDE,
            struct_types,
            function_payloads,
            if gen.has_entrypoints() {
                "export * as entry from \"./entry.js\";"
            } else {
                ""
            },
            self.module_id.address().to_hex_literal(),
            self.module_id.short_str_lossless(),
            self.module_id.name(),
            serde_json::to_string_pretty(&error_map)?,
            serde_json::to_string_pretty(&self.errors)?,
            serde_json::to_string_pretty(&fn_map)?,
            serde_json::to_string_pretty(&resources)?,
            serde_json::to_string_pretty(&structs)?,
            self.module_id.short_str_lossless(),
            gen_doc_string_opt(&self.doc),
            self.module_id.address().to_hex_literal(),
            name,
        );

        Ok(ts)
    }
}
