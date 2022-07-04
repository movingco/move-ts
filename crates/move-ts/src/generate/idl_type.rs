use super::{Codegen, CodegenContext};
use crate::format::gen_doc_string;
use anyhow::*;
use move_idl::{IDLField, IDLStructType, IDLType};

impl Codegen for IDLField {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        Ok(format!(
            "{}{}: {};",
            self.doc
                .as_ref()
                .map(|doc| format!("\n{}\n", gen_doc_string(doc)))
                .unwrap_or_default(),
            self.name,
            ctx.generate(&self.ty)?
        ))
    }
}

impl Codegen for IDLStructType {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        let struct_def = ctx
            .pkg
            .structs
            .iter()
            .find(|sd| sd.module_id == self.module_id && sd.name == self.name)
            .unwrap();

        Ok(format!(
            "{{\n{}\n}}",
            ctx.try_join_with_separator(&struct_def.fields, "\n")?
                .trim()
                .indent()
        ))
    }
}

impl Codegen for IDLType {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        Ok(match self {
            IDLType::Bool => "boolean".to_string(),
            IDLType::U8 => "number".to_string(),
            IDLType::U64 => "p.U64".to_string(),
            IDLType::U128 => "p.U128".to_string(),
            IDLType::Address => "p.HexStringArg".to_string(),
            IDLType::Signer => "p.HexStringArg".to_string(),
            IDLType::Vector(inner) => match *inner.clone() {
                IDLType::U8 => "p.HexStringArg".to_string(),
                inner => format!("ReadonlyArray<{}>", ctx.generate(&inner)?),
            },
            IDLType::Struct(inner) => ctx.generate(inner)?.into(),
            IDLType::TypeParameter(_) => "unknown".to_string(),
            IDLType::Tuple(_) => todo!(),
        })
    }
}
