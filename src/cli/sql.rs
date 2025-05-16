use anyhow::anyhow;
use colored::Colorize;
use rayon::prelude::*;
use sqlparser::parser::Parser;
use std::{fs, path::Path};
use walkdir::WalkDir;

pub fn parse_file<P: AsRef<Path>>(file: P) -> anyhow::Result<Vec<sqlparser::ast::Statement>> {
    let sql = std::fs::read_to_string(file)?;

    let dialect = sqlparser::dialect::GenericDialect {};
    let ast = Parser::parse_sql(&dialect, &sql)?;

    Ok(ast)
}

pub fn fmt_recursively<P: AsRef<Path>>(dir: P, minify: bool) -> anyhow::Result<()> {
    if !dir.as_ref().is_dir() {
        return Err(anyhow!("argument 1 must be a directory"));
    }

    let entries: Vec<walkdir::DirEntry> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| {
            if let Some(f) = e.ok() {
                if f.path().extension().is_some_and(|f| f == "sql") {
                    return Some(f);
                } else {
                    return None;
                }
            }

            None
        })
        .collect();

    if entries.is_empty() {
        eprintln!("{}: No target files found.", "Error".red());
        std::process::exit(1);
    }

    entries.par_iter().for_each(|d| {
        if d.file_type().is_file() {
            let ast = match parse_file(d.path()) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("{}: {e} ({:?})", "Error".red(), d.path());
                    std::process::exit(1);
                }
            };

            let mut buffer = String::new();
            for node in &ast {
                let formatted = match minify {
                    false => {
                        let mut node_str = sqlformat::format(
                            &node.to_string(),
                            &sqlformat::QueryParams::None,
                            &sqlformat::FormatOptions::default(),
                        );

                        if !node_str.ends_with(";") {
                            node_str.push(';');
                        }

                        node_str
                    }
                    true => {
                        let mut node_str = node.to_string();
                        node_str.push(';');
                        node_str
                    }
                };

                buffer.push_str(&formatted);
                buffer.push_str("\n\n");
            }

            fs::write(d.path(), buffer).unwrap();
            println!("{:?}", d.path());
        }
    });

    Ok(())
}

pub fn fmt(sql: &str, minify: bool) -> anyhow::Result<String> {
    let dialect = sqlparser::dialect::GenericDialect {};
    let ast = Parser::parse_sql(&dialect, &sql)?;

    let mut buffer = String::new();
    for node in &ast {
        let formatted = match minify {
            false => {
                let mut node_str = sqlformat::format(
                    &node.to_string(),
                    &sqlformat::QueryParams::None,
                    &sqlformat::FormatOptions::default(),
                );

                if !node_str.ends_with(";") {
                    node_str.push(';');
                }

                node_str
            }
            true => {
                let mut node_str = node.to_string();
                node_str.push(';');
                node_str
            }
        };

        buffer.push_str(&formatted);
    }

    Ok(buffer)
}
