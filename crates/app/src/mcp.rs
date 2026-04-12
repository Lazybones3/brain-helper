use brain_api::{Settings, response::{datafield::{self, DataField}, operator::Operator}};
use rmcp::{
    ServerHandler, ServiceExt, handler::server::{router::tool::ToolRouter, wrapper::Parameters}, model::{Content, IntoContents, ServerCapabilities, ServerInfo}, schemars, serde_json, tool, tool_handler, tool_router
};
use rmcp::ErrorData as McpError;
use serde::Deserialize;

use crate::app::BrainApp;

mod app;

#[derive(Deserialize, schemars::JsonSchema)]
struct MCPDataFieldRequest {
    dataset_name: String,
}

struct McpVecResponse<T: serde::Serialize> {
    data: Vec<T>
}

impl<T: serde::Serialize> IntoContents for McpVecResponse<T> {
    fn into_contents(self) -> Vec<Content> {
        self.data.iter().filter_map(|item| {
            if let Ok(text) = serde_json::to_string(item) {
                Some(Content::text(text))
            } else {
                None
            }
        }).collect()
    }
}

pub struct WorldQuantBrain {
    tool_router: ToolRouter<WorldQuantBrain>,
    app: BrainApp
}

#[tool_router]
impl WorldQuantBrain {
    async fn new() -> anyhow::Result<Self> {
        let settings = Settings::default();
        let app = BrainApp::new(settings).await?;
        Ok(Self {
            tool_router: Self::tool_router(),
            app,
        })
    }

    #[tool(description = "Get data fields by dataset name.")]
    async fn get_datafields(
        &self,
        Parameters(MCPDataFieldRequest { dataset_name }): Parameters<MCPDataFieldRequest>,
    ) -> Result<McpVecResponse<DataField>, McpError> {
        match self.app.get_fields_by_dataset(&dataset_name, None).await {
            Ok(data) => Ok(McpVecResponse { data }),
            Err(e) => Err(McpError::internal_error(e.to_string(), None))
        }
    }

    #[tool(description = "Get all available operators.")]
    async fn get_operators() -> Result<McpVecResponse<Operator>, McpError> {
        match brain_api::get_operators().await {
            Ok(data) => Ok(McpVecResponse { data }),
            Err(e) => Err(McpError::internal_error(e.to_string(), None))
        }
    }
}

#[tool_handler]
impl ServerHandler for WorldQuantBrain {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("This is a world quant tool.")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let service = WorldQuantBrain::new().await?.serve(transport).await?;
    service.waiting().await?;
    Ok(())
}
