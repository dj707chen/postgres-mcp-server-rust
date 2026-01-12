use crate::db::DatabaseClient;
use anyhow::Result;
use serde_json::{json, Value};

pub fn get_query_tool_schema() -> Value {
    json!({
        "name": "query",
        "description": "Execute a SQL query against the PostgreSQL database. Read-only by default unless DANGEROUSLY_ALLOW_WRITE_OPS is enabled.",
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
    })
}

pub async fn execute_query_tool(
    db: &DatabaseClient,
    arguments: &Value,
) -> Result<Value> {
    let sql = arguments
        .get("sql")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'sql' parameter"))?;

    let rows = db.query(sql).await?;
    let results = DatabaseClient::rows_to_json(&rows);

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&results)?
            }
        ]
    }))
}
