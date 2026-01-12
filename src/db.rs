use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio_postgres::{Client, NoTls, Row};
use tracing::{error, info};

pub struct DatabaseClient {
    client: Client,
    allow_write_ops: bool,
}

impl DatabaseClient {
    pub async fn connect(database_url: &str, allow_write_ops: bool) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(database_url, NoTls)
            .await
            .context("Failed to connect to PostgreSQL")?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("PostgreSQL connection error: {}", e);
            }
        });

        info!("Connected to PostgreSQL database");

        Ok(Self {
            client,
            allow_write_ops,
        })
    }

    pub async fn query(&self, sql: &str) -> Result<Vec<Row>> {
        if !self.allow_write_ops && Self::is_write_query(sql) {
            anyhow::bail!(
                "Write operations are not allowed. Set DANGEROUSLY_ALLOW_WRITE_OPS=true to enable."
            );
        }

        self.client
            .query(sql, &[])
            .await
            .context("Failed to execute query")
    }

    pub async fn list_tables(&self) -> Result<Vec<String>> {
        let query =
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'";
        let rows = self.client.query(query, &[]).await?;

        Ok(rows
            .iter()
            .filter_map(|row| row.get::<_, Option<String>>(0))
            .collect())
    }

    pub async fn read_table(&self, table_name: &str, limit: usize) -> Result<Vec<Row>> {
        let query = format!("SELECT * FROM {} LIMIT {}", table_name, limit);
        self.client
            .query(&query, &[])
            .await
            .context("Failed to read table")
    }

    pub fn is_write_query(sql: &str) -> bool {
        let sql_upper = sql.trim().to_uppercase();
        sql_upper.starts_with("INSERT")
            || sql_upper.starts_with("UPDATE")
            || sql_upper.starts_with("DELETE")
            || sql_upper.starts_with("DROP")
            || sql_upper.starts_with("CREATE")
            || sql_upper.starts_with("ALTER")
            || sql_upper.starts_with("TRUNCATE")
    }

    pub fn row_to_json(row: &Row) -> Value {
        let mut row_map = HashMap::new();

        for (idx, column) in row.columns().iter().enumerate() {
            let value: Value = match column.type_().name() {
                "int2" => row
                    .get::<_, Option<i16>>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(Value::Null),
                "int4" => row
                    .get::<_, Option<i32>>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(Value::Null),
                "int8" => row
                    .get::<_, Option<i64>>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(Value::Null),
                "text" | "varchar" | "bpchar" | "name" => row
                    .get::<_, Option<String>>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(Value::Null),
                "bool" => row
                    .get::<_, Option<bool>>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(Value::Null),
                "float4" => row
                    .get::<_, Option<f32>>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(Value::Null),
                "float8" => row
                    .get::<_, Option<f64>>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(Value::Null),
                "numeric" => row
                    .get::<_, Option<String>>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(Value::Null),
                "timestamp" | "timestamptz" => row
                    .get::<_, Option<NaiveDateTime>>(idx)
                    .map(|v| json!(v.to_string()))
                    .unwrap_or(Value::Null),
                "date" => row
                    .get::<_, Option<NaiveDate>>(idx)
                    .map(|v| json!(v.to_string()))
                    .unwrap_or(Value::Null),
                "time" | "timetz" => row
                    .get::<_, Option<NaiveTime>>(idx)
                    .map(|v| json!(v.to_string()))
                    .unwrap_or(Value::Null),
                "uuid" => row
                    .get::<_, Option<String>>(idx)
                    .map(|v| json!(v))
                    .unwrap_or(Value::Null),
                "json" | "jsonb" => row
                    .get::<_, Option<String>>(idx)
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or(Value::Null),
                _ => Value::Null,
            };
            row_map.insert(column.name().to_string(), value);
        }

        json!(row_map)
    }

    pub fn rows_to_json(rows: &[Row]) -> Vec<Value> {
        rows.iter().map(Self::row_to_json).collect()
    }
}
