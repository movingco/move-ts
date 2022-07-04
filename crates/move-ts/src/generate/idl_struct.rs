use super::{Codegen, CodegenContext};
use crate::format::gen_doc_string;
use anyhow::*;
use move_idl::{IDLField, IDLStruct};

impl Codegen for Vec<IDLField> {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        Ok(self
            .iter()
            .map(|field| {
                let ts = &field.ty.generate_typescript(ctx)?;
                Ok(format!(
                    "{}{}: {};",
                    field
                        .doc
                        .as_ref()
                        .map(|doc| format!("\n{}\n", gen_doc_string(doc)))
                        .unwrap_or_default(),
                    field.name,
                    ts
                ))
            })
            .collect::<Result<Vec<_>>>()?
            .join("\n")
            .trim()
            .to_string())
    }
}

impl Codegen for IDLStruct {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        Ok(format!(
            r#"{}export type {}Data = {{
{}
}};"#,
            self.doc
                .as_ref()
                .map(|d| format!("{}\n", gen_doc_string(d)))
                .unwrap_or_default(),
            self.name,
            ctx.generate(&self.fields)?.indent()
        ))
    }
}
