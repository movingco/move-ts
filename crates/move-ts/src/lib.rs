//! Generates TypeScript code from a Move IDL.

pub mod format;
pub mod idl_module;
pub mod idl_package;
pub mod idl_struct;
pub mod idl_type;
pub mod script_function;

use crate::format::indent;
use anyhow::*;
use format::gen_doc_string;
use idl_module::IDLModuleGenerator;
use move_idl::{IDLModule, IDLPackage};
use serde::Serialize;
use std::fmt::Display;

/// Generate TypeScript code for a value.
pub trait Codegen {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String>;
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CodeText(String);

impl Codegen for CodeText {
    fn generate_typescript(&self, _ctx: &CodegenContext) -> Result<String> {
        Ok(self.to_string())
    }
}

impl AsRef<[u8]> for CodeText {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Display for CodeText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl CodeText {
    pub fn new(s: &str) -> Self {
        CodeText(s.to_string())
    }

    pub fn new_reexport(name: &str, path: &str) -> Self {
        format!("export * as {} from \"{}\";", name, path).into()
    }

    pub fn new_named_reexport(name: &str, path: &str) -> Self {
        format!("export {{ {} }} from \"{}\";", name, path).into()
    }

    pub fn new_fields_export(name: &str, fields: &str) -> Self {
        format!("export type {} = {{\n{} }};", name, fields).into()
    }

    pub fn append_newline(&self) -> Self {
        CodeText(format!("{}\n", self.0))
    }

    /// Creates a `export const {name} = {value} as const` statement.
    pub fn new_const_export<T>(name: &str, value: &T) -> Result<Self>
    where
        T: ?Sized + Serialize,
    {
        Ok(format!(
            "export const {} = {} as const;",
            name,
            serde_json::to_string_pretty(value)?
        )
        .into())
    }

    pub fn try_join_with_separator<'a, I>(values: I, separator: &str) -> Result<CodeText>
    where
        I: IntoIterator<Item = &'a CodeText>,
    {
        Ok(values
            .into_iter()
            .map(|v| v.to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(separator)
            .into())
    }

    pub fn docs(&self, docs: &str) -> CodeText {
        format!("{}{}", gen_doc_string(docs), self).into()
    }

    pub fn module_docs(&self, docs: &str) -> CodeText {
        format!(
            "{}\n{}",
            gen_doc_string(&format!("{}\n\n@module", docs)),
            self
        )
        .into()
    }
}

pub struct CodegenContext<'info> {
    pkg: &'info IDLPackage,
}

impl<'info> CodegenContext<'info> {
    pub fn new(pkg: &'info IDLPackage) -> Self {
        CodegenContext { pkg }
    }

    pub fn get_module_generator(&self, value: &'info IDLModule) -> IDLModuleGenerator<'info> {
        IDLModuleGenerator::new(value)
    }

    pub fn generate<T: Codegen>(&self, value: &T) -> Result<CodeText> {
        Ok(CodeText(value.generate_typescript(self)?))
    }

    pub fn try_join_with_separator<'a, I, T>(&self, values: I, separator: &str) -> Result<CodeText>
    where
        I: IntoIterator<Item = &'a T>,
        T: Codegen + 'a,
    {
        CodeText::try_join_with_separator(
            &values
                .into_iter()
                .map(|v| self.generate(v))
                .collect::<Result<Vec<_>>>()?,
            separator,
        )
    }

    pub fn try_join<'a, I, T>(&self, values: I) -> Result<CodeText>
    where
        I: IntoIterator<Item = &'a T>,
        T: Codegen + 'a,
    {
        self.try_join_with_separator(values, "\n\n")
    }
}

impl From<&str> for CodeText {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl From<String> for CodeText {
    fn from(s: String) -> Self {
        CodeText(s)
    }
}

impl From<CodeText> for String {
    fn from(s: CodeText) -> Self {
        s.0
    }
}

impl CodeText {
    pub fn indent(&self) -> CodeText {
        indent(&self.0).into()
    }

    pub fn trim(&self) -> CodeText {
        self.0.trim().into()
    }
}
