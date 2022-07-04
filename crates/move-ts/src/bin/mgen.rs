use std::path::PathBuf;

use anyhow::Result;
use move_ts::*;

fn main() -> Result<()> {
    generate_types(&PathBuf::from("."))?;
    Ok(())
}
