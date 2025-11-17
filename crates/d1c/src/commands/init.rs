use anyhow::{Error, Result};
use rusqlite::Connection;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use clap::Args;
use toml::Table;

use crate::{commands::dump_schema::dump_schema, D1CConfig};

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Dry run - don't write to files
    #[arg(long, default_value = "false")]
    pub dry_run: bool,
}

pub fn run(conn: &Connection, args: &InitArgs) -> Result<(), Error> {
    let wrangler_toml = walk_directory_for_wrangler_toml()?;
    let mut migrations_dir: Option<String> = None;
    let mut use_detected_migrations_dir = false;

    if let Some(content) = wrangler_toml {
        println!("✨ Found wrangler.toml");

        let parsed_migrations_dir = try_parse_migrations_dir(&content)?;

        if let Some(dir) = parsed_migrations_dir {
            println!("Detected D1 migrations directory: {}", dir);
            use_detected_migrations_dir =
                inquire::Confirm::new("Use this directory for migrations?")
                    .with_default(true)
                    .prompt()?;

            if use_detected_migrations_dir {
                migrations_dir = Some(dir);
            }
        } else {
            println!("⚠️  Found wrangler.toml but no migrations_dir configured");
            println!("   Add a D1 database to wrangler.toml:");
            println!();
            println!("   [[d1_databases]]");
            println!("   binding = \"DB\"");
            println!("   database_name = \"my-db\"");
            println!("   migrations_dir = \"db/migrations\"");
            println!();
        }
    } else {
        println!("💡 No wrangler.toml found - using manual configuration");
    }

    if migrations_dir.is_none() {
        let manual_migrations_dir = inquire::Text::new("Where do your migrations live?")
            .with_default("migrations")
            .prompt()?;
        migrations_dir = Some(manual_migrations_dir);
    }

    let queries_default = if use_detected_migrations_dir {
        migrations_dir
            .as_ref()
            .unwrap()
            .replace("migrations", "queries")
            .to_string()
    } else {
        "db/queries".to_string()
    };

    let queries_dir = inquire::Text::new("Where do you want the queries to live?")
        .with_default(queries_default.as_str())
        .prompt()?;

    let out_dir = inquire::Text::new("Where do you want to write the generated code?")
        .with_default("src/db")
        .prompt()?;

    let module_name = inquire::Text::new("What is the name of the module for the generated code?")
        .with_default("queries")
        .prompt()?;

    let emit_schema = inquire::Confirm::new("Do you want to emit the schema.sql file?")
        .with_help_message(
            "Not required by d1c, but useful for viewing your schema. \
         D1 doesn't provide an easy way to inspect schema otherwise.",
        )
        .with_default(true)
        .prompt()?;

    let config = D1CConfig {
        migrations_dir: migrations_dir.unwrap(),
        queries_dir: queries_dir,
        out_dir: out_dir,
        module_name: module_name,
        emit_schema: emit_schema,
    };

    let config_string = toml::to_string_pretty(&config)?;

    if !args.dry_run {
        let mut config_file = File::create("d1c.toml")?;
        config_file.write_all(config_string.as_bytes())?;

        fs::create_dir_all(&config.queries_dir)?;
        fs::create_dir_all(&config.out_dir)?;

        if config.emit_schema {
            let schema_rows = dump_schema(&conn)?;
            let schema_string = schema_rows
                .iter()
                .map(|row| row.sql.clone())
                .collect::<Vec<String>>()
                .join("\n\n");
            let mut schema_file = File::create(Path::new(&config.out_dir).join("schema.sql"))?;
            schema_file.write_all(schema_string.as_bytes())?;
        }

        println!("✅ Created {}/", config.queries_dir);
        println!();
        println!("✅ Created {}/", config.out_dir);
        println!();
        println!("✅ Created {}/schema.sql", config.out_dir);
        println!();
        println!("✅ Created d1c.toml");
        println!();
        println!("Next steps:");
        println!(
            "  - Create your first query in {}/example.sql",
            &config.queries_dir
        );
        println!("  - Run `d1c generate` to generate typed Rust bindings");
        println!("  - Import them in your Worker:");
        println!();
        println!("      use crate::db::{}::*;", &config.module_name);
        println!();
        println!("Happy querying!");
    } else {
        println!("================================================");
        println!("DRY RUN: Would have created d1c.toml with the following contents:");
        println!("================================================");

        println!("{}", config_string);
    }

    Ok(())
}

const MAX_DEPTH: usize = 5;
fn walk_directory_for_wrangler_toml() -> Result<Option<String>> {
    let mut depth = 0;
    let mut current_dir = std::env::current_dir()?;

    while depth < MAX_DEPTH {
        let candidate = current_dir.join("wrangler.toml");
        if candidate.is_file() {
            let content = fs::read_to_string(candidate)?;
            return Ok(Some(content));
        }

        if !current_dir.pop() {
            break;
        }
        depth += 1;
    }

    Ok(None)
}

fn try_parse_migrations_dir(wrangler_toml: &String) -> Result<Option<String>> {
    let parsed = wrangler_toml.parse::<Table>().unwrap();

    if let Some(toml::Value::Array(dbs)) = parsed.get("d1_databases") {
        for db in dbs {
            if let Some(table) = db.as_table() {
                if let Some(dir) = table.get("migrations_dir").and_then(|v| v.as_str()) {
                    return Ok(Some(dir.to_string()));
                }
            }
        }
    }

    Ok(None)
}
