use crate::db::DatabaseClient;
use anyhow::Result;
use serde_json::{json, Value};

const DEFAULT_ROW_LIMIT: usize = 100;

pub async fn list_table_resources(db: &DatabaseClient) -> Result<Vec<Value>> {
    let tables = db.list_tables().await?;

    let resources: Vec<Value> = tables
        .iter()
        .map(|table_name| {
            json!({
                "uri": format!("postgres:///{}", table_name),
                "name": table_name.clone(),
                "description": format!("PostgreSQL table: {}", table_name),
                "mimeType": "application/json"
            })
        })
        .collect();

    Ok(resources)
}

pub async fn read_table_resource(db: &DatabaseClient, uri: &str) -> Result<Value> {
    let table_name = parse_table_uri(uri)
        .ok_or_else(|| anyhow::anyhow!("Invalid table URI: {}", uri))?;

    let rows = db.read_table(&table_name, DEFAULT_ROW_LIMIT).await?;
    let results = DatabaseClient::rows_to_json(&rows);

    Ok(json!({
        "contents": [
            {
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(&results)?
            }
        ]
    }))
}

fn parse_table_uri(uri: &str) -> Option<String> {
    uri.strip_prefix("postgres:///")
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_table_uri() {
        assert_eq!(
            parse_table_uri("postgres:///users"),
            Some("users".to_string())
        );
        assert_eq!(
            parse_table_uri("postgres:///products"),
            Some("products".to_string())
        );
        assert_eq!(parse_table_uri("invalid://uri"), None);
        assert_eq!(parse_table_uri("postgres://"), None);
    }
}
