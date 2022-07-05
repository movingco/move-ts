use super::{Codegen, CodegenContext};
use crate::{format::gen_doc_string, idl_type::generate_idl_type_with_type_args, CodeText};
use anyhow::*;
use move_idl::IDLStruct;

fn generate_struct_fields(s: &IDLStruct, ctx: &CodegenContext) -> Result<CodeText> {
    Ok(s.fields
        .iter()
        .map(|field| {
            let ts = &generate_idl_type_with_type_args(
                &field.ty,
                ctx,
                &s.type_params
                    .iter()
                    .map(|t| format!("_{}", t))
                    .collect::<Vec<_>>(),
            )?;
            Ok(format!(
                "{}{}: {};",
                field
                    .doc
                    .as_ref()
                    .map(|doc| format!("\n{}", gen_doc_string(doc)))
                    .unwrap_or_default(),
                field.name,
                ts
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join("\n")
        .trim()
        .to_string()
        .into())
}

impl Codegen for IDLStruct {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        let generics = if self.type_params.is_empty() {
            "".to_string()
        } else {
            format!(
                "<{}>",
                self.type_params
                    .iter()
                    .map(|p| format!("_{} = unknown", p))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        Ok(format!(
            r#"{}export type {}Data{} = {{
{}
}};"#,
            self.doc
                .as_ref()
                .map(|d| gen_doc_string(d))
                .unwrap_or_default(),
            self.name,
            generics,
            generate_struct_fields(self, ctx)?.indent()
        ))
    }
}
