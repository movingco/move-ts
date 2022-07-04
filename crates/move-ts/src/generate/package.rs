use super::CodegenContext;
use anyhow::Result;
use move_idl::IDLPackage;
use std::path::Path;

pub struct PackageCodegen<'info> {
    pkg: &'info IDLPackage,
}

impl<'info> PackageCodegen<'info> {
    pub fn new(pkg: &'info IDLPackage) -> Self {
        Self { pkg }
    }

    pub fn gen_package_deps(&self, out: &Path) -> Result<()> {
        let ctx = CodegenContext::new(self.pkg);
        for module_idl in self.pkg.dependencies.values() {
            let ts = ctx.generate(module_idl)?;
            std::fs::write(
                out.join(format!("{}.ts", module_idl.module_id.name())),
                ts.to_string(),
            )?;
        }
        Ok(())
    }

    pub fn gen_package_modules(&self, out: &Path) -> Result<()> {
        let ctx = CodegenContext::new(self.pkg);
        for module_idl in self.pkg.modules.values() {
            let ts = ctx.generate(module_idl)?;
            std::fs::write(
                out.join(format!("{}.ts", module_idl.module_id.name())),
                ts.to_string(),
            )?;
        }
        Ok(())
    }

    pub fn gen_package(&self, root: &Path, include_deps: bool) -> Result<()> {
        let modules_out = root.join("modules");
        std::fs::create_dir_all(&modules_out)?;
        self.gen_package_modules(&modules_out)?;

        if include_deps {
            let deps_out = root.join("dependencies");
            std::fs::create_dir_all(&deps_out)?;
            self.gen_package_deps(&deps_out)?;
        }
        Ok(())
    }
}
