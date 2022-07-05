use anyhow::*;
use heck::ToPascalCase;
use move_idl::{IDLArgument, IDLModule, IDLScriptFunction};

use crate::format::{gen_doc_string, indent};

use super::{CodeText, Codegen, CodegenContext};

pub struct ScriptFunctionPayloadStruct<'info>(&'info ScriptFunctionType<'info>);

impl<'info> ScriptFunctionPayloadStruct<'info> {
    fn doc_link(&self) -> String {
        format!(
            "{{@link {}Module.{}}}",
            self.0.module.module_id.name(),
            self.0.script.name
        )
    }

    fn args_inline(&self, ctx: &CodegenContext) -> Result<CodeText> {
        Ok(ctx
            .try_join_with_separator(&self.0.script.args, "\n")?
            .indent())
    }

    fn type_args_inline(&self) -> CodeText {
        script_fn_type_args(&self.0.script.ty_args).indent()
    }
}

impl<'info> Codegen for ScriptFunctionPayloadStruct<'info> {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        Ok(format!(
            "/**\n * Payload arguments for {}.\n */\nexport type {}Payload = {{\n{}{}}};",
            self.doc_link(),
            self.0.type_name,
            if self.0.script.args.is_empty() {
                "".to_string()
            } else {
                format!(
                    "{}\n",
                    indent(&format!("args: {{\n{}\n}};\n", self.args_inline(ctx)?))
                )
            },
            if self.0.script.ty_args.is_empty() {
                "".to_string()
            } else {
                format!(
                    "{}\n",
                    indent(&format!("typeArgs: {{\n{}\n}};\n", self.type_args_inline()))
                )
            },
        ))
    }
}

pub struct ScriptFunctionType<'info> {
    type_name: String,
    module: &'info IDLModule,
    script: &'info IDLScriptFunction,
}

fn script_fn_type_args(args: &[String]) -> CodeText {
    args.iter()
        .map(|arg| format!("{}: string;", arg))
        .collect::<Vec<_>>()
        .join("\n")
        .into()
}

impl Codegen for IDLArgument {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        let doc = gen_doc_string(&format!("IDL type: `{:?}`", &self.ty));
        Ok(format!(
            "{}\n{}: {};",
            doc,
            self.name,
            &self.ty.generate_typescript(ctx)?
        ))
    }
}

impl<'info> ScriptFunctionType<'info> {
    pub fn new(module: &'info IDLModule, script: &'info IDLScriptFunction) -> Self {
        let type_name = script.name.to_pascal_case();
        Self {
            type_name,
            module,
            script,
        }
    }

    pub fn payload(&'info self) -> ScriptFunctionPayloadStruct<'info> {
        ScriptFunctionPayloadStruct(self)
    }
}

impl<'info> Codegen for ScriptFunctionType<'info> {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        let function = format!(
            "{}::{}",
            &self.module.module_id.short_str_lossless(),
            &self.script.name
        );
        let type_arguments = format!(
            "[{}]",
            self.script
                .ty_args
                .iter()
                .map(|a| format!("typeArgs.{}", a))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let arguments = format!(
            "[{}]",
            self.script
                .args
                .iter()
                .map(|a| {
                    let inner = format!("args.{}", a.name);
                    let ts_type = &ctx.generate(&a.ty)?.to_string();
                    let serializer = if ts_type == "p.U64" {
                        "p.serializers.u64"
                    } else if ts_type == "p.U128" {
                        "p.serializers.u128"
                    } else if ts_type == "p.HexStringArg" {
                        "p.serializers.hexString"
                    } else {
                        return Ok(inner);
                    };
                    Ok(format!("{}({})", serializer, &inner))
                })
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        );

        Ok(format!(
            r#"{}{}: ({{ {} }}: {}Payload): p.ScriptFunctionPayload => ({{
  type: "script_function_payload",
  function: "{}",
  type_arguments: {},
  arguments: {},
}}),"#,
            self.script
                .doc
                .as_ref()
                .map(|doc| gen_doc_string(doc))
                .unwrap_or_default(),
            self.script.name,
            vec![
                if self.script.args.is_empty() {
                    None
                } else {
                    Some("args")
                },
                if self.script.ty_args.is_empty() {
                    None
                } else {
                    Some("typeArgs")
                },
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(", "),
            self.type_name,
            &function,
            &type_arguments,
            &arguments
        ))
    }
}
