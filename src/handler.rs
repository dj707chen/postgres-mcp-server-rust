use crate::db::DatabaseClient;
use crate::resources;
use crate::tools;
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct PostgresHandler {
    db: Arc<Mutex<Option<DatabaseClient>>>,
    database_url: String,
    allow_write_ops: bool,
}

impl PostgresHandler {
    pub fn new(database_url: String, allow_write_ops: bool) -> Self {
        Self {
            db: Arc::new(Mutex::new(None)),
            database_url,
            allow_write_ops,
        }
    }

    async fn ensure_connected(&self) -> Result<()> {
        let mut db_lock = self.db.lock().await;
        if db_lock.is_none() {
            info!("Establishing database connection...");
            let client = DatabaseClient::connect(&self.database_url, self.allow_write_ops).await?;
            *db_lock = Some(client);
            info!("Database connection established");
        }
        Ok(())
    }

    pub async fn handle_initialize(&self) -> Result<Value> {
        Ok(json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "postgres-mcp-server",
                "version": "0.2.0"
            }
        }))
    }

    pub async fn handle_list_tools(&self) -> Result<Value> {
        Ok(json!({
            "tools": [tools::get_query_tool_schema()]
        }))
    }

    pub async fn handle_call_tool(&self, name: &str, arguments: &Value) -> Result<Value> {
        self.ensure_connected().await?;

        let db_lock = self.db.lock().await;
        let db = db_lock
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not connected"))?;

        match name {
            "query" => tools::execute_query_tool(db, arguments).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
        }
    }

    pub async fn handle_list_resources(&self) -> Result<Value> {
        self.ensure_connected().await?;

        let db_lock = self.db.lock().await;
        let db = db_lock
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not connected"))?;

        let resources = resources::list_table_resources(db).await?;

        Ok(json!({
            "resources": resources
        }))
    }

    pub async fn handle_read_resource(&self, uri: &str) -> Result<Value> {
        self.ensure_connected().await?;

        let db_lock = self.db.lock().await;
        let db = db_lock
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not connected"))?;

        resources::read_table_resource(db, uri).await
    }

    pub async fn handle_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        match method {
            "initialize" => self.handle_initialize().await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => {
                let params = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
                let empty_args = json!({});
                let arguments = params.get("arguments").unwrap_or(&empty_args);
                self.handle_call_tool(name, arguments).await
            }
            "resources/list" => self.handle_list_resources().await,
            "resources/read" => {
                let params = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
                let uri = params
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing URI"))?;
                self.handle_read_resource(uri).await
            }
            _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
        }
    }
}
