use crate::format::gen_doc_string_opt;

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

impl Codegen for IDLModule {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
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

        let function_bodies = ctx.try_join(&script_fns)?.indent();

        let struct_types = ctx.try_join(&self.structs)?;

        let idl_json = &serde_json::to_string(self)?;

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

/** Entrypoint builders. */
export const entrypoints = {{
{}
}} as const;

/** The IDL of the module. */
export const idl = {} as const;

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

  ...entrypoints
}} as const;

{}export const moduleDefinition = moduleImpl as p.MoveModuleDefinition<"{}", "{}"> as typeof moduleImpl;
"#,
            gen_doc_string_opt(&self.doc.as_ref().map(|s| format!("{}\n\n@module", s))),
            PRELUDE,
            struct_types,
            function_payloads,
            function_bodies,
            idl_json,
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
