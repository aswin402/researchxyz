// mcp/client.rs - wrapper for rmcp client transport

pub struct McpClientWrapper {
    pub server_alias: String,
}

impl McpClientWrapper {
    pub fn new(server_alias: String) -> Self {
        Self { server_alias }
    }
    
    pub async fn connect(&self) -> Result<(), anyhow::Error> {
        // Stub for rmcp stdio client connection
        Ok(())
    }
}
