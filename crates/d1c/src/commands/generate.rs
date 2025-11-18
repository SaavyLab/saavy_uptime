use std::{fs, path::{Path, PathBuf}};

use anyhow::Result;
use rusqlite::Connection;
use crate::{D1CConfig, commands::generate::{analyzer::analyze_query, parser::process_query_file, renderer::render_module}, utils::sql::collect_sql_files};

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
    fs::write(Path::new(&config.out_dir).join("queries.rs"), formatted)?;
    Ok(())
}
