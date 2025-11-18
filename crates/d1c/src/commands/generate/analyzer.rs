use anyhow::Result;
use rusqlite::Connection;
use crate::commands::generate::types::{Cardinality, ColumnInfo, Query};

pub fn analyze_query(conn: &Connection, query: &mut Query) -> Result<()> {
    // 1. Use the transformed SQL from the parser (which handles parameter replacement robustly)
    // instead of re-implementing it naively here.
    let final_sql = &query.transformed_sql;

    // 2. Prepare against local SQLite to get return types
    let stmt = conn.prepare(final_sql);
    
    match stmt {
        Ok(stmt) => {
            let column_count = stmt.column_count();
            
            // Strict validation for Scalar queries
            if matches!(query.cardinality, Cardinality::Scalar) {
                if column_count != 1 {
                    anyhow::bail!(
                        "Query '{}' is marked as :scalar but returns {} columns. Scalar queries must return exactly one column.",
                        query.name,
                        column_count
                    );
                }
            }

            let mut columns = Vec::with_capacity(column_count);
            for column in stmt.columns() {
                let decl_type = column.decl_type().unwrap_or("TEXT");
                // SQLite's prepare/stmt.columns() doesn't expose "NOT NULL" constraints directly
                // via the stable public API easily in older rusqlite versions, OR it requires
                // inspecting the table schema if it's a simple select.
                //
                // However, for arbitrary queries (SELECT sum(x)...), nullability is complex.
                // Rusqlite's `Column` doesn't have `not_null()`.
                //
                // But! d1c runs locally against a real SQLite.
                // We can use `PRAGMA table_info(table_name)` IF the column origin is known.
                // `column.table_name()` and `column.origin_name()` exist if `features = ["column_decltype"]` is on (which it is).
                //
                // Actually, rusqlite 0.31 `Column` doesn't expose `not_null`.
                // We have to infer it or default to Option.
                //
                // STRATEGY: 
                // 1. If we can identify the source table and column, check that table's schema.
                // 2. If it's an expression (count(*)), assume NOT NULL for specific aggregates?
                // 
                // Let's try to check origin.
                // Note: origin_name() might be None for expressions.
                
                // TODO: Enable SQLITE_ENABLE_COLUMN_METADATA to use table_name()/origin_name()
                // For now, defaulting to NOT NULL (false) to preserve existing behavior until build is fixed.
                let is_nullable = false; 
                /*
                let is_nullable = if let Some(table) = column.table_name() {
                     if let Some(col_name) = column.origin_name() {
                         is_column_nullable(conn, table, col_name).unwrap_or(true)
                     } else {
                         true // Expression or unknown
                     }
                } else {
                     true // Expression or unknown
                };
                */

                let rust_type = sqlite_type_to_rust(decl_type)
                    .unwrap_or_else(|_| "String"); 
                
                columns.push(ColumnInfo {
                    name: column.name().to_string(),
                    decl: decl_type.to_string(),
                    rust_type: rust_type.to_string(),
                    not_null: !is_nullable,
                });
            }
            query.columns = columns;
        }
        Err(e) => {
            // It's common for prepare to fail if the query relies on a table 
            // that doesn't exist in the local schema yet (if migrations failed).
            eprintln!("⚠️  Warning: Could not analyze query '{}': {}", query.name, e);
        }
    }

    Ok(())
}

fn sqlite_type_to_rust(decl_type: &str) -> Result<&str> {
    let upper = decl_type.to_uppercase();
    // Handle common SQLite type affinities and aliases
    // https://www.sqlite.org/datatype3.html
    if upper.contains("INT") {
        Ok("i64")
    } else if upper.contains("CHAR") || upper.contains("CLOB") || upper.contains("TEXT") {
        Ok("String")
    } else if upper.contains("BLOB") {
        Ok("Vec<u8>")
    } else if upper.contains("REAL") || upper.contains("FLOA") || upper.contains("DOUB") {
        Ok("f64")
    } else if upper.contains("BOOL") {
        Ok("bool") 
    } else if upper == "DATE" || upper == "DATETIME" {
        // D1/SQLite usually stores these as Strings or Numbers. 
        // String is the safer default without more context.
        Ok("String") 
    } else if upper == "NUMERIC" {
        Ok("f64")
    } else {
        // If we really can't match, fallback is handled by caller, 
        // but we can return a generic error here if we want to be strict.
        // For this helper, let's default to String if it looks like text, or error.
        // But given the fallback in caller, we can just error or return String.
        Ok("String")
    }
}

#[allow(dead_code)]
fn is_column_nullable(conn: &Connection, table: &str, col_name: &str) -> Result<bool> {
    // PRAGMA table_info(table_name) returns:
    // cid | name | type | notnull | dflt_value | pk
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        let notnull: i64 = row.get(3)?;
        Ok((name, notnull))
    })?;

    for row in rows {
        let (name, notnull) = row?;
        if name == col_name {
            // notnull is 1 if NOT NULL, 0 otherwise.
            // So nullable is true if notnull == 0
            return Ok(notnull == 0);
        }
    }

    // Column not found? Default to true (nullable)
    Ok(true)
}
