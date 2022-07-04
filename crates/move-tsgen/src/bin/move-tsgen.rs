use anyhow::*;
use json_cli::CliTool;
use move_tsgen::MoveTSGenTool;

#[tokio::main]
async fn main() -> Result<()> {
    MoveTSGenTool::execute_main().await
}
