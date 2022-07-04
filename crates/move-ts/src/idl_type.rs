use super::{Codegen, CodegenContext};
use crate::{format::gen_doc_string, CodeText};
use anyhow::*;
use move_idl::{IDLField, IDLStructType, IDLType};

fn generate_field_with_type_args(
    ty: &IDLField,
    ctx: &CodegenContext,
    type_args: &[String],
) -> Result<String> {
    Ok(format!(
        "{}{}: {};",
        ty.doc
            .as_ref()
            .map(|doc| format!("\n{}\n", gen_doc_string(doc)))
            .unwrap_or_default(),
        ty.name,
        generate_idl_type_with_type_args(&ty.ty, ctx, type_args)?
    ))
}

fn generate_struct_with_type_args(
    ty: &IDLStructType,
    ctx: &CodegenContext,
    type_args: &[String],
) -> Result<String> {
    let struct_def = ctx
        .pkg
        .structs
        .iter()
        .find(|sd| sd.module_id == ty.module_id && sd.name == ty.name)
        .unwrap();

    let fields_gen: CodeText = struct_def
        .fields
        .iter()
        .map(|v| generate_field_with_type_args(v, ctx, type_args))
        .collect::<Result<Vec<_>>>()?
        .join("\n")
        .into();

    Ok(format!("{{\n{}\n}}", fields_gen.trim().indent()))
}

pub(crate) fn generate_idl_type_with_type_args(
    idl_type: &IDLType,
    ctx: &CodegenContext,
    type_args: &[String],
) -> Result<String> {
    Ok(match idl_type {
        IDLType::Bool => "boolean".to_string(),
        IDLType::U8 => "number".to_string(),
        IDLType::U64 => "p.U64".to_string(),
        IDLType::U128 => "p.U128".to_string(),
        IDLType::Address => "p.HexStringArg".to_string(),
        IDLType::Signer => "p.HexStringArg".to_string(),
        IDLType::Vector(inner) => match *inner.clone() {
            IDLType::U8 => "p.HexStringArg".to_string(),
            inner => format!(
                "ReadonlyArray<{}>",
                generate_idl_type_with_type_args(&inner, ctx, type_args)?
            ),
        },
        IDLType::Struct(inner) => {
            let next_type_args = inner
                .ty_args
                .iter()
                .map(|arg| generate_idl_type_with_type_args(arg, ctx, type_args))
                .collect::<Result<Vec<_>>>()?;
            generate_struct_with_type_args(inner, ctx, &next_type_args)?
        }
        IDLType::TypeParameter(v) => {
            let result = type_args.get(*v as usize);
            match result {
                Some(v) => v.clone(),
                None => "unknown".to_string(),
            }
        }
        IDLType::Tuple(_) => todo!(),
    })
}

impl Codegen for IDLType {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        generate_idl_type_with_type_args(self, ctx, &[])
    }
}
