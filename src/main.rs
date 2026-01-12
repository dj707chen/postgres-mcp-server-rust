use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, Write};
use tokio_postgres::{Client, NoTls};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

struct McpServer {
    database_url: String,
    allow_write_ops: bool,
    client: Option<Client>,
}

impl McpServer {
    fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable not set")?;

        let allow_write_ops = env::var("DANGEROUSLY_ALLOW_WRITE_OPS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        Ok(Self {
            database_url,
            allow_write_ops,
            client: None,
        })
    }

    async fn connect(&mut self) -> Result<()> {
        let (client, connection) = tokio_postgres::connect(&self.database_url, NoTls)
            .await
            .context("Failed to connect to PostgreSQL")?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Connection error: {}", e);
            }
        });

        self.client = Some(client);
        info!("Connected to PostgreSQL");
        Ok(())
    }

    async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_tools_list(request.id),
            "tools/call" => self.handle_tools_call(request.id, request.params).await,
            "resources/list" => self.handle_resources_list(request.id).await,
            "resources/read" => self.handle_resources_read(request.id, request.params).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            },
        }
    }

    fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {}
                },
                "serverInfo": {
                    "name": "postgres-mcp-server",
                    "version": "0.1.0"
                }
            })),
            error: None,
        }
    }

    fn handle_tools_list(&self, id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "query",
                        "description": "Execute a SQL query against the PostgreSQL database",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "sql": {
                                    "type": "string",
                                    "description": "SQL query to execute"
                                }
                            },
                            "required": ["sql"]
                        }
                    }
                ]
            })),
            error: None,
        }
    }

    async fn handle_tools_call(
        &mut self,
        id: Option<Value>,
        params: Option<Value>,
    ) -> JsonRpcResponse {
        if self.client.is_none() {
            if let Err(e) = self.connect().await {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: format!("Failed to connect to database: {}", e),
                        data: None,
                    }),
                };
            }
        }

        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing parameters".to_string(),
                        data: None,
                    }),
                }
            }
        };

        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        match tool_name {
            "query" => self.execute_query(id, arguments).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("Unknown tool: {}", tool_name),
                    data: None,
                }),
            },
        }
    }

    async fn execute_query(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        let sql = match arguments.get("sql").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing 'sql' parameter".to_string(),
                        data: None,
                    }),
                }
            }
        };

        if !self.allow_write_ops && is_write_query(sql) {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Write operations are not allowed. Set DANGEROUSLY_ALLOW_WRITE_OPS=true to enable.".to_string(),
                    data: None,
                }),
            };
        }

        let client = self.client.as_ref().unwrap();

        match client.query(sql, &[]).await {
            Ok(rows) => {
                let mut results = Vec::new();
                for row in rows {
                    let mut row_map = HashMap::new();
                    for (idx, column) in row.columns().iter().enumerate() {
                        let value: Value = match column.type_().name() {
                            "int4" => row.get::<_, Option<i32>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "int8" => row.get::<_, Option<i64>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "text" | "varchar" => row.get::<_, Option<String>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "bool" => row.get::<_, Option<bool>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "float4" => row.get::<_, Option<f32>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "float8" => row.get::<_, Option<f64>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            _ => Value::Null,
                        };
                        row_map.insert(column.name().to_string(), value);
                    }
                    results.push(row_map);
                }

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".to_string())
                            }
                        ]
                    })),
                    error: None,
                }
            }
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: format!("Query execution error: {}", e),
                    data: None,
                }),
            },
        }
    }

    async fn handle_resources_list(&self, id: Option<Value>) -> JsonRpcResponse {
        if self.client.is_none() {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({"resources": []})),
                error: None,
            };
        }

        let client = self.client.as_ref().unwrap();

        let query = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'";

        match client.query(query, &[]).await {
            Ok(rows) => {
                let resources: Vec<Value> = rows
                    .iter()
                    .filter_map(|row| {
                        row.get::<_, Option<String>>(0).map(|table_name| {
                            json!({
                                "uri": format!("postgres:///{}", table_name),
                                "name": table_name,
                                "description": format!("PostgreSQL table: {}", table_name),
                                "mimeType": "application/json"
                            })
                        })
                    })
                    .collect();

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({"resources": resources})),
                    error: None,
                }
            }
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: format!("Failed to list tables: {}", e),
                    data: None,
                }),
            },
        }
    }

    async fn handle_resources_read(
        &self,
        id: Option<Value>,
        params: Option<Value>,
    ) -> JsonRpcResponse {
        let uri = match &params {
            Some(p) => match p.get("uri").and_then(|u| u.as_str()) {
                Some(u) => u.to_string(),
                None => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Missing 'uri' parameter".to_string(),
                            data: None,
                        }),
                    }
                }
            },
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing parameters".to_string(),
                        data: None,
                    }),
                }
            }
        };

        let table_name = uri.trim_start_matches("postgres:///");

        if self.client.is_none() {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Not connected to database".to_string(),
                    data: None,
                }),
            };
        }

        let client = self.client.as_ref().unwrap();
        let query = format!("SELECT * FROM {} LIMIT 100", table_name);

        match client.query(&query, &[]).await {
            Ok(rows) => {
                let mut results = Vec::new();
                for row in rows {
                    let mut row_map = HashMap::new();
                    for (idx, column) in row.columns().iter().enumerate() {
                        let value: Value = match column.type_().name() {
                            "int4" => row.get::<_, Option<i32>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "int8" => row.get::<_, Option<i64>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "text" | "varchar" => row.get::<_, Option<String>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "bool" => row.get::<_, Option<bool>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "float4" => row.get::<_, Option<f32>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            "float8" => row.get::<_, Option<f64>>(idx).map(|v| json!(v)).unwrap_or(Value::Null),
                            _ => Value::Null,
                        };
                        row_map.insert(column.name().to_string(), value);
                    }
                    results.push(row_map);
                }

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "contents": [
                            {
                                "uri": uri,
                                "mimeType": "application/json",
                                "text": serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".to_string())
                            }
                        ]
                    })),
                    error: None,
                }
            }
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: format!("Failed to read table: {}", e),
                    data: None,
                }),
            },
        }
    }
}

fn is_write_query(sql: &str) -> bool {
    let sql_upper = sql.trim().to_uppercase();
    sql_upper.starts_with("INSERT")
        || sql_upper.starts_with("UPDATE")
        || sql_upper.starts_with("DELETE")
        || sql_upper.starts_with("DROP")
        || sql_upper.starts_with("CREATE")
        || sql_upper.starts_with("ALTER")
        || sql_upper.starts_with("TRUNCATE")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting PostgreSQL MCP Server");

    let mut server = McpServer::new()?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse request: {}", e);
                continue;
            }
        };

        let response = server.handle_request(request).await;
        let response_str = serde_json::to_string(&response)?;

        writeln!(stdout, "{}", response_str)?;
        stdout.flush()?;
    }

    Ok(())
}
