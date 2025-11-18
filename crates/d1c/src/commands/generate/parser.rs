use std::{fs, ops::ControlFlow, path::PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use sqlparser::{
    ast::{visit_expressions_mut, Expr, Value, ValueWithSpan},
    dialect::SQLiteDialect,
    parser::Parser,
};

use crate::commands::generate::types::{Cardinality, ParamSpec, Query};

const QUERY_NAME: &str = "-- name:";
const QUERY_PARAMS: &str = "-- params:";

struct ParamVisitor {
    found_params: Vec<String>,
}

pub fn process_query_file(file: &PathBuf) -> Result<Vec<Query>> {
    let sql_content = fs::read_to_string(file)?;
    let mut lines = sql_content.lines().peekable();
    let mut queries: Vec<Query> = Vec::new();

    while let Some(line) = lines.next() {
        if line.starts_with(QUERY_NAME) {
            let mut query_sql: Vec<String> = Vec::new();
            let query_name = line.to_string();
            let mut explicit_params = Vec::new();

            // 1. Consume lines belonging to this query block
            while let Some(next_line) = lines.peek() {
                if next_line.starts_with(QUERY_NAME) {
                    break; // Start of next query
                }

                if next_line.starts_with(QUERY_PARAMS) {
                    let parsed_params = parse_query_params(next_line)?;
                    explicit_params.extend(parsed_params);
                } else {
                    let trimmed = next_line.trim();
                    // Skip empty lines and comments
                    if !trimmed.is_empty() && !trimmed.starts_with("--") {
                        query_sql.push(trimmed.to_string());
                    }
                }

                lines.next();
            }

            // 2. Parse header
            let (name, cardinality) = parse_query_header(query_name.as_str())?;

            // 3. Parse and Rewrite SQL
            let raw_sql = query_sql.join("\n");
            let (transformed_sql, detected_param_names) = rewrite_and_extract_params(&raw_sql)
                .with_context(|| format!("Failed to rewrite SQL for query: {}", name))?;

            // 4. Merge explicit types (from -- params:) with detected names
            // If a param was detected but not explicitly typed, bailout

            let mut final_params = Vec::new();

            for param_name in detected_param_names {
                let rust_type = explicit_params
                    .iter()
                    .find(|p| p.name == param_name)
                    .map(|p| p.rust_type.clone())
                    .unwrap_or_else(|| "String".to_string());

                final_params.push(ParamSpec {
                    name: param_name,
                    rust_type,
                });
            }

            queries.push(Query {
                name,
                cardinality,
                params: if final_params.is_empty() { None } else { Some(final_params) },
                sql: query_sql,
                transformed_sql,
                returns: None,
                columns: Vec::new(),
            });
        }
    }

    Ok(queries)
}

/// Uses sqlparser to traverse the SQL, extract :params, and rewrite them to ?1, ?2, etc.
fn rewrite_and_extract_params(sql: &str) -> Result<(String, Vec<String>)> {
    let dialect = SQLiteDialect {};
    
    let mut statements = Parser::parse_sql(&dialect, sql)?;

    let mut visitor = ParamVisitor { found_params: Vec::new() };

    let _ = visit_expressions_mut(&mut statements, |expr| {
        if let Expr::Value(ValueWithSpan { value: Value::Placeholder(param_name), .. }) = expr {
            let clean_name = param_name.trim_start_matches(':').to_string();
            let idx = if let Some(i) = visitor.found_params.iter().position(|p| p == &clean_name) {
                i + 1
            } else {
                visitor.found_params.push(clean_name);
                visitor.found_params.len()
            };

            *param_name = format!("?{}", idx);
        }
        ControlFlow::<()>::Continue(())
    });

    // Reconstruct the SQL string from the modified AST
    // This has the side effect of normalizing formatting, which is good
    let transformed_sql = statements
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    Ok((transformed_sql, visitor.found_params))
} 

pub fn parse_query_header(line: &str) -> Result<(String, Cardinality)> {
    let parts = line
        .strip_prefix(QUERY_NAME)
        .ok_or(anyhow::anyhow!("Invalid query header"))?
        .split_whitespace()
        .collect::<Vec<&str>>();

    if parts.len() < 2 {
        anyhow::bail!("Query header must be: -- name: <function_name> :<cardinality>");
    }
    
    let name = parts[0].to_string();
    let cardinality = match parts[1] {
        ":one" => Cardinality::One,
        ":many" => Cardinality::Many,
        ":exec" => Cardinality::Exec,
        ":scalar" => Cardinality::Scalar,
        _ => anyhow::bail!("Invalid cardinality: {}", parts[1]),
    };

    Ok((name, cardinality))
}

pub fn parse_query_params(line: &str) -> Result<Vec<ParamSpec>> {
    let parts = line
        .strip_prefix(QUERY_PARAMS)
        .ok_or(anyhow::anyhow!("Invalid query params"))?
        .split(",")
        .map(|p| p.trim())
        .collect::<Vec<&str>>();

    let mut params = Vec::new();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        let pieces = part.split_whitespace().collect::<Vec<&str>>();
        if pieces.len() != 2 {
            anyhow::bail!("Params must be in the format `name Type`");
        }
        params.push(ParamSpec {
            name: pieces[0].to_string(),
            rust_type: pieces[1].to_string(),
        });
    }

    Ok(params)
}

fn extract_params_from_sql(sql: &str) -> Vec<ParamSpec> {
    let re = Regex::new(r":(\w+)").unwrap();
    let mut params = Vec::new();
    // Use a HashSet to dedupe if needed
    for cap in re.captures_iter(sql) {
        let name = cap[1].to_string();
        // Basic type inference assumption: Everything is String or &str unless configured otherwise
        // For v1, defaulting to String/&str is safe.
        params.push(ParamSpec { 
            name, 
            rust_type: "String".to_string() 
        });
    }
    params
}
