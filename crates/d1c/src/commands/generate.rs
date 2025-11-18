use std::{fs, path::{Path, PathBuf}};

use anyhow::Result;
use rusqlite::Connection;
use crate::{D1CConfig, commands::{dump_schema::dump_schema, generate::{analyzer::analyze_query, parser::process_query_file, renderer::render_module}}, utils::sql::collect_sql_files};

mod analyzer;
mod parser;
mod renderer;
mod types;

pub fn run(conn: &Connection, config: &D1CConfig) -> Result<()> {
    let query_files = collect_sql_files(PathBuf::from(&config.queries_dir))?;
    let mut queries = Vec::new();
    for query_file in query_files {
        let parsed_queries = process_query_file(&query_file)?;
        queries.extend(parsed_queries);
    }

    for mut query in &mut queries {
        analyze_query(conn, &mut query)?;
    }

    let module_tokens = render_module(&queries, config.instrument_by_default);
    let ast = syn::parse2(module_tokens)?;
    let formatted = prettyplease::unparse(&ast);
    
    let output_file = if config.module_name.ends_with(".rs") {
        config.module_name.clone()
    } else {
        format!("{}.rs", config.module_name)
    };
    
    fs::write(Path::new(&config.out_dir).join(output_file), formatted)?;

    // Emit schema.sql if configured
    if config.emit_schema {
        let schema_rows = dump_schema(conn)?;
        let schema_string = schema_rows
            .iter()
            .map(|row| row.sql.clone())
            .collect::<Vec<String>>()
            .join("\n\n");
        // Ensure it ends with a newline for git friendliness
        let final_schema = if schema_string.is_empty() {
            schema_string
        } else {
            format!("{}\n", schema_string)
        };
        
        fs::write(Path::new(&config.queries_dir).join("schema.sql"), final_schema)?;
    }

    Ok(())
}
