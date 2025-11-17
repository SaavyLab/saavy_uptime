use anyhow::Result;
use rusqlite::Connection;
use crate::commands::generate::types::{QueryInfo, ColumnInfo};

pub fn strip_comments(query: &str) -> String {
    let mut result = Vec::new();
    let mut in_block_comment = false;

    for line in query.lines() {
        if line.starts_with("--") {
            continue;
        }
        
        if line.starts_with("/*") && !in_block_comment {
            in_block_comment = true;
            continue;
        }

        if in_block_comment && line.ends_with("*/") {
            in_block_comment = false;
            continue;
        }

        if in_block_comment {
            continue;
        }
        result.push(line);
    }
    result.join("\n")
}

pub fn analyze_query(conn: &Connection, query: &str) -> Result<QueryInfo> {
    let cleaned = strip_comments(query);
    let stmt = conn.prepare(&cleaned)?;
    let column_count = stmt.column_count();

    let mut columns = Vec::with_capacity(column_count);
    for column in stmt.columns() {
        let decl_type = column.decl_type().unwrap_or_default().to_string();
        let rust_type = sqlite_type_to_rust(&decl_type)?.to_string();
        columns.push(ColumnInfo {
            name: column.name().to_string(),
            decl: decl_type.clone(),
            rust_type,
        });
    }

    let mut params = Vec::new();
    for i in 1..stmt.parameter_count() {
        if let Some(param_name) = stmt.parameter_name(i) {
            params.push(param_name.trim_start_matches([':', '@', '?', '$']).to_string());
        }
    }

    Ok(QueryInfo { columns, params })
}

fn sqlite_type_to_rust(decl_type: &str) -> Result<&str> {
    match decl_type {
        "INTEGER" => Ok("i64"),
        "REAL" => Ok("f64"),
        "TEXT" => Ok("String"),
        "BLOB" => Ok("Vec<u8>"),
        "NUMERIC" => Ok("f64"),
        _ => anyhow::bail!("Unsupported SQLite type: {}", decl_type),
    }
}