pub mod idl_module;
pub mod idl_struct;
pub mod idl_type;
pub mod package;
pub mod script_function;
use crate::format::indent;
use anyhow::*;
use move_idl::IDLPackage;
use std::fmt::Display;

/// Generate TypeScript code for a value.
pub trait Codegen {
    fn generate_typescript(&self, ctx: &CodegenContext) -> Result<String>;
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CodeText(String);

impl Display for CodeText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct CodegenContext<'info> {
    pkg: &'info IDLPackage,
}

impl<'info> CodegenContext<'info> {
    pub fn new(pkg: &'info IDLPackage) -> Self {
        CodegenContext { pkg }
    }

    pub fn generate<T: Codegen>(&self, value: &T) -> Result<CodeText> {
        Ok(CodeText(value.generate_typescript(self)?))
    }

    pub fn try_join_with_separator<'a, I, T>(&self, values: I, separator: &str) -> Result<CodeText>
    where
        I: IntoIterator<Item = &'a T>,
        T: Codegen + 'a,
    {
        Ok(values
            .into_iter()
            .map(|v| self.generate(v))
            .collect::<Result<Vec<_>>>()?
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(separator)
            .into())
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
