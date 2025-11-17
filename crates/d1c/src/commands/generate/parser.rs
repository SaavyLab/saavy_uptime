use std::{fs, path::PathBuf};

use anyhow::Result;

use crate::commands::generate::types::{Cardinality, ParsedParamInfo, ParsedQuery};

const QUERY_NAME: &str = "-- name:";
const QUERY_PARAMS: &str = "-- params:";

pub fn process_query_file(file: &PathBuf) -> Result<Vec<ParsedQuery>> {
    let sql = fs::read_to_string(file)?;
    let mut lines = sql.lines().peekable();
    let mut queries: Vec<ParsedQuery> = Vec::new();

    while let Some(line) = lines.next() {
        if line.starts_with(QUERY_NAME) {
            let mut query_sql: Vec<String> = Vec::new();
            let query_name = line.to_string();
            let mut params = Vec::new();
            while let Some(next_line) = lines.peek() {
                if next_line.starts_with(QUERY_NAME) {
                    break;
                }
                if next_line.starts_with(QUERY_PARAMS) {
                    let parsed_params = parse_query_params(next_line)?;
                    params.extend(parsed_params);
                }

                let trimmed = next_line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    query_sql.push(next_line.to_string());
                }

                lines.next();
            }

            let (name, cardinality) = parse_query_header(query_name.as_str())?;
            queries.push(ParsedQuery {
                name,
                cardinality,
                params: if params.is_empty() { None } else { Some(params) },
                sql: query_sql,
                returns: None,
            });
        }
    }

    Ok(queries)
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

pub fn parse_query_params(line: &str) -> Result<Vec<ParsedParamInfo>> {
    let parts = line
        .strip_prefix(QUERY_PARAMS)
        .ok_or(anyhow::anyhow!("Invalid query params"))?
        .split(",")
        .map(|p| p.trim())
        .collect::<Vec<&str>>();
    
    let mut params = Vec::new();
    for part in parts {
        let (name, rust_type) = part.split_once(" ").ok_or(anyhow::anyhow!("Invalid query param: {}", part))?;
        params.push(ParsedParamInfo {
            name: name.to_string(),
            rust_type: rust_type.to_string(),
        });
    }

    Ok(params)
}