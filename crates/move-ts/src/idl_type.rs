use super::{Codegen, CodegenContext};
use crate::{format::gen_doc_string, CodeText};
use anyhow::*;
use move_idl::{IDLField, IDLStructType, IDLType};

pub fn serialize_arg(arg: &str, ty: &IDLType, ctx: &CodegenContext) -> Result<String> {
    let ts_type = &ctx.generate(ty)?.to_string();
    if ts_type.starts_with("ReadonlyArray") {
        if let IDLType::Vector(inner) = ty {
            let inner_arg = format!("inner_{}", arg.replace('.', "__"));
            let inner_serialized = serialize_arg(&inner_arg, inner, ctx)?;
            Ok(format!(
                "{}.map(({}) => {})",
                arg, inner_arg, inner_serialized
            ))
        } else {
            anyhow::bail!("Expected vector type for {}", arg)
        }
    } else {
        let serializer = if ts_type == "p.U64" {
            "p.serializers.u64"
        } else if ts_type == "p.U128" {
            "p.serializers.u128"
        } else if ts_type == "p.HexStringArg"
            || ts_type == "p.RawAddress"
            || ts_type == "p.RawSigner"
            || ts_type == "p.ByteString"
        {
            "p.serializers.hexString"
        } else {
            return Ok(arg.to_string());
        };
        Ok(format!("{}({})", serializer, &arg))
    }
}

fn generate_field_with_type_args(
    ty: &IDLField,
    ctx: &CodegenContext,
    type_args: &[String],
    parse_args: bool,
) -> Result<String> {
    Ok(format!(
        "{}{}: {};",
        ty.doc
            .as_ref()
            .map(|doc| format!("\n{}", gen_doc_string(doc)))
            .unwrap_or_default(),
        ty.name,
        generate_idl_type_with_type_args(&ty.ty, ctx, type_args, parse_args)?
    ))
}

fn generate_struct_with_type_args(
    ty: &IDLStructType,
    ctx: &CodegenContext,
    type_args: &[String],
    parse_args: bool,
) -> Result<String> {
    let struct_def = ctx
        .pkg
        .structs
        .iter()
        .find(|sd| sd.name == ty.name)
        .unwrap();

    let fields_gen: CodeText = struct_def
        .fields
        .iter()
        .map(|v| generate_field_with_type_args(v, ctx, type_args, parse_args))
        .collect::<Result<Vec<_>>>()?
        .join("\n")
        .into();

    Ok(format!("{{\n{}\n}}", fields_gen.trim().indent()))
}

pub(crate) fn generate_idl_type_with_type_args(
    idl_type: &IDLType,
    ctx: &CodegenContext,
    type_args: &[String],
    parse_args: bool,
) -> Result<String> {
    let result = match idl_type {
        IDLType::Bool => "boolean".to_string(),
        IDLType::U8 => "number".to_string(),
        IDLType::U64 => "p.U64".to_string(),
        IDLType::U128 => "p.U128".to_string(),
        IDLType::Address => "p.RawAddress".to_string(),
        IDLType::Signer => "p.RawSigner".to_string(),
        IDLType::Vector(inner) => match *inner.clone() {
            IDLType::U8 => "p.ByteString".to_string(),
            inner => format!(
                "ReadonlyArray<{}>",
                generate_idl_type_with_type_args(&inner, ctx, type_args, parse_args)?
            ),
        },
        IDLType::Struct(inner) => {
            if inner.name.to_string() == *"0x1::ASCII::String" {
                "string".to_string()
            } else {
                let next_type_args = inner
                    .ty_args
                    .iter()
                    .map(|arg| generate_idl_type_with_type_args(arg, ctx, type_args, parse_args))
                    .collect::<Result<Vec<_>>>()?;
                generate_struct_with_type_args(inner, ctx, &next_type_args, parse_args)?
            }
        }
        IDLType::TypeParam(v) => {
            let result = type_args.get(*v as usize);
            match result {
                Some(v) => v.clone(),
                None => "unknown".to_string(),
            }
        }
        IDLType::Tuple(_) => todo!(),
    };
    if !parse_args && result.starts_with("p.") {
        Ok("string".to_string())
    } else {
        Ok(result)
    }
}

impl Codegen for IDLType {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String> {
        generate_idl_type_with_type_args(self, ctx, &[], true)
    }
}
