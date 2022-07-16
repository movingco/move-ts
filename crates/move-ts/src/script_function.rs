use anyhow::*;
use heck::ToPascalCase;
use move_idl::{IDLArgument, IDLModule, IDLScriptFunction};

use crate::{
    format::{gen_doc_string, indent},
    idl_type::{generate_idl_type_with_type_args, serialize_arg},
};

use super::{CodeText, Codegen, CodegenContext};

pub struct ScriptFunctionPayloadStruct<'info>(&'info ScriptFunctionType<'info>);

impl<'info> ScriptFunctionPayloadStruct<'info> {
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
        Ok(CodeText::new_fields_export(
            &self.0.payload_args_type_name(),
            &format!(
                "{}{}",
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
            ),
        )
        .docs(&format!("Payload arguments for {}.", self.0.doc_link()))
        .into())
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
            "{}{}: {};",
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

    pub fn doc_link(&self) -> String {
        format!("{{@link entry.{}}}", self.script.name)
    }

    pub fn payload(&'info self) -> ScriptFunctionPayloadStruct<'info> {
        ScriptFunctionPayloadStruct(self)
    }

    pub fn generate_entry_payload_struct(&self, ctx: &CodegenContext) -> Result<CodeText> {
        let arguments = format!(
            "[{}]",
            self.script
                .args
                .iter()
                .map(|a| {
                    let ts_type = &generate_idl_type_with_type_args(&a.ty, ctx, &[], false)?;
                    Ok(format!("{}: {}", a.name, &ts_type))
                })
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        );
        let type_arguments = format!(
            "[{}]",
            self.script
                .ty_args
                .iter()
                .map(|a| format!("{}: string", a))
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(CodeText::new_fields_export(
            &self.type_name,
            &CodeText::try_join_with_separator(
                &[
                    CodeText::new("readonly type: \"script_function_payload\";"),
                    CodeText::new(&format!("readonly function: \"{}\";", self.full_name())),
                    CodeText::new(&format!("readonly arguments: {};", &arguments)),
                    CodeText::new(&format!("readonly type_arguments: {};", &type_arguments)),
                ],
                "\n",
            )?
            .indent()
            .append_newline()
            .to_string(),
        )
        .docs(&format!(
            "Script function payload for `{}`.{}",
            self.full_name(),
            self.script
                .doc
                .as_ref()
                .map(|s| format!("\n\n{}", s))
                .unwrap_or_default()
        )))
    }

    pub fn full_name(&self) -> String {
        format!("{}::{}", self.module.module_id, self.script.name)
    }

    pub fn payload_args_type_name(&'info self) -> String {
        format!("{}Args", self.type_name)
    }

    pub fn should_render_payload_struct(&'info self) -> bool {
        !(self.script.args.is_empty() && self.script.ty_args.is_empty())
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
                    serialize_arg(&inner, &a.ty, ctx)
                })
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        );

        Ok(format!(
            r#"{}export const {} = ({}): payloads.{} => ({{
  type: "script_function_payload",
  function: "{}",
  type_arguments: {},
  arguments: {},
}});"#,
            self.script
                .doc
                .as_ref()
                .map(|doc| gen_doc_string(doc))
                .unwrap_or_default(),
            self.script.name,
            if self.should_render_payload_struct() {
                format!(
                    "{{ {} }}: mod.{}",
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
                    self.payload_args_type_name()
                )
            } else {
                "".to_string()
            },
            &self.type_name,
            &function,
            &type_arguments,
            &arguments
        ))
    }
}
