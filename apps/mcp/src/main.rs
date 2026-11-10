//! Creative Engine MCP Server
//!
//! Model Context Protocol server for the Creative Engine.

use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _transport = stdio();
    println!("Creative Engine MCP Server - placeholder");
    Ok(())
}