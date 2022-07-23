use super::{Codegen, CodegenContext};
use crate::{format::gen_doc_string, idl_type::generate_idl_type_with_type_args, CodeText};
use anyhow::*;
use move_idl::{IDLStruct, IDLType};

fn generate_struct_fields(s: &IDLStruct, ctx: &CodegenContext) -> Result<CodeText> {
    Ok(s.fields
        .iter()
        .map(|field| {
            let ts = &generate_idl_type_with_type_args(
                &field.ty,
                ctx,
                &s.type_params
                    .iter()
                    .filter(|t| !t.is_phantom)
                    .map(|t| format!("_{}", t.name))
                    .collect::<Vec<_>>(),
                true,
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
        if self.fields.len() == 1
            && self.fields[0].name == "dummy_field"
            && self.fields[0].ty == IDLType::Bool
        {
            return Ok("".to_string());
        }

        let generics = if !self.type_params.iter().any(|p| !p.is_phantom) {
            "".to_string()
        } else {
            format!(
                "<{}>",
                self.type_params
                    .iter()
                    .filter(|p| !p.is_phantom)
                    .map(|p| format!("_{} = unknown", p.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        Ok(CodeText::new(&format!(
            r#"export interface I{}{} {{
{}
}};"#,
            self.name.name,
            generics,
            generate_struct_fields(self, ctx)?.indent()
        ))
        .docs(
            &self
                .doc
                .as_ref()
                .map(|d| gen_doc_string(d))
                .unwrap_or_default(),
        )
        .into())
    }
}
