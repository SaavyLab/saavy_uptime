use anyhow::Result;
use rusqlite::Connection;
use crate::commands::generate::types::{ColumnInfo, ParamSpec, Query};

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

pub fn analyze_query(conn: &Connection, query: &mut Query) -> Result<()> {
    // 1. Combine lines
    let raw_sql = query.sql_text();
    let cleaned = strip_comments(&raw_sql);

    // 2. Detect parameters (:id) and replace with ?1, ?2
    // Note: This assumes your parser.rs populated `query.params` from the comments.
    // Ideally, we would detect them here from the SQL itself. 
    
    let mut final_sql = cleaned.clone();
    
    if let Some(params) = &query.params {
        // Replace :param_name with ?
        // We iterate in reverse order of length to avoid replacing :user_id with ?_id when :user exists
        let mut sorted_params = params.clone();
        sorted_params.sort_by(|a, b| b.name.len().cmp(&a.name.len()));

        for (idx, param) in sorted_params.iter().enumerate() {
            // D1 uses ?1, ?2. 
            let placeholder = format!("?{}", idx + 1); 
            // Simple replace. In a real parser we'd check we aren't inside a string literal
            final_sql = final_sql.replace(&format!(":{}", param.name), &placeholder);
        }
    }

    query.transformed_sql = final_sql.clone();

    // 3. Prepare against local SQLite to get return types
    // We use the Transformed SQL (with ?s)
    let stmt = conn.prepare(&final_sql);
    
    match stmt {
        Ok(stmt) => {
            let column_count = stmt.column_count();
            let mut columns = Vec::with_capacity(column_count);
            for column in stmt.columns() {
                let decl_type = column.decl_type().unwrap_or("TEXT").to_string(); // Default to TEXT if unknown
                let rust_type = sqlite_type_to_rust(&decl_type)?.to_string();
                columns.push(ColumnInfo {
                    name: column.name().to_string(),
                    decl: decl_type,
                    rust_type,
                });
            }
            query.columns = columns;
        }
        Err(e) => {
            // It's common for prepare to fail if the query relies on a table 
            // that doesn't exist in the local schema yet (if migrations failed).
            // We should warn but maybe not crash?
            eprintln!("⚠️  Warning: Could not analyze query '{}': {}", query.name, e);
        }
    }

    Ok(())
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
