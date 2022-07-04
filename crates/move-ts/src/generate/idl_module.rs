use super::{script_function::ScriptFunctionType, Codegen, CodegenContext};
use anyhow::*;
use move_idl::{IDLAbility, IDLModule, IDLScriptFunction};
use std::collections::BTreeMap;

const PRELUDE: &str = "import * as p from \"@movingco/prelude\";\n";

impl Codegen for IDLModule {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        let name = self.module_id.name();

        let script_fns = self
            .functions
            .iter()
            .map(|script_fn| ScriptFunctionType::new(self, script_fn))
            .collect::<Vec<_>>();

        let function_payloads =
            ctx.try_join(&script_fns.iter().map(|f| f.payload()).collect::<Vec<_>>())?;

        let function_bodies = ctx.try_join(&script_fns)?.indent();

        let struct_types = ctx.try_join(&self.structs)?;

        let idl_json = &serde_json::to_string(self)?;

        let mut fn_map: BTreeMap<String, IDLScriptFunction> = BTreeMap::new();
        for script_fn in self.functions.iter() {
            fn_map.insert(script_fn.name.clone(), script_fn.clone());
        }

        let mut resources: BTreeMap<String, String> = BTreeMap::new();
        for struct_info in ctx
            .pkg
            .structs
            .iter()
            .filter(|s| s.abilities.contains(&IDLAbility::Key))
        {
            resources.insert(
                struct_info.name.clone(),
                format!(
                    "{}::{}",
                    self.module_id.short_str_lossless(),
                    struct_info.name.clone()
                ),
            );
        }

        let mut structs: BTreeMap<String, String> = BTreeMap::new();
        for struct_info in ctx.pkg.structs.iter() {
            structs.insert(
                struct_info.name.clone(),
                format!(
                    "{}::{}",
                    self.module_id.short_str_lossless(),
                    struct_info.name.clone()
                ),
            );
        }

        let ts = format!(
            r#"
{}

{}

{}

/** Function builders. */
const builders = {{
{}
}} as const;

/** Payload generators for module `{}`. */
const moduleImpl = {{
  /** The address of the module. */
  ADDRESS: "{}",
  /** The full module name. */
  FULL_NAME: "{}",
  /** The name of the module. */
  NAME: "{}",
  /** The IDL of the module. */
  IDL: {},
  /** All module function IDLs. */
  functions: {},
  /** All struct types with ability `key`. */
  resources: {},
  /** All struct types. */
  structs: {},

  ...builders
}} as const;

export const {}Module = moduleImpl as p.MoveModuleDefinition<"{}", "{}"> as typeof moduleImpl;
"#,
            PRELUDE,
            struct_types,
            function_payloads,
            function_bodies,
            self.module_id.short_str_lossless(),
            self.module_id.address().to_hex_literal(),
            self.module_id.short_str_lossless(),
            self.module_id.name(),
            idl_json,
            serde_json::to_string(&fn_map)?,
            serde_json::to_string(&resources)?,
            serde_json::to_string(&structs)?,
            name,
            self.module_id.address().to_hex_literal(),
            self.module_id.name(),
        );

        Ok(ts)
    }
}
