use std::{fs::File, io::Write, path::{Path, PathBuf}};

use anyhow::Result;
use rusqlite::Connection;
use crate::{D1CConfig, commands::generate::{analyzer::analyze_query, parser::process_query_file, renderer::{generate_queries_file, generate_query_function}}, utils::sql::collect_sql_files};

mod analyzer;
mod parser;
mod renderer;
mod types;

pub fn run(conn: &Connection, config: &D1CConfig) -> Result<()> {
    let query_files = collect_sql_files(PathBuf::from(&config.queries_dir))?;
    let mut functions = Vec::new();
    for query_file in query_files {
        let queries = process_query_file(&query_file)?;

        for query in queries {
            let query_info = analyze_query(&conn, &query.sql.join("\n"))?;
            let function = generate_query_function(&query, &query_info)?;
            functions.push(function);
        }
    }

    let out_file_contents = generate_queries_file(&functions)?;

    let mut out_file = File::create(Path::new(&config.out_dir).join("queries.rs"))?;
    Write::write_all(&mut out_file, out_file_contents.as_bytes())?;

    Ok(())
}

