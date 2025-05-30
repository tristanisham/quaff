use anyhow::anyhow;
use clap::Parser;
use cli::Command;
use cli::config;
use cli::config::LangOption;
use colored::Colorize;
use lang::php;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sql::schema::Formatter;
use std::io::{Read, Write};
use std::str::FromStr;
use std::{fs, io};

mod cli;
mod lang;
mod sql;

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    let cwd = std::env::current_dir().unwrap_or(".".into());
    let default_config_path = cwd.join("quaff.toml");
    let config_file = args.config.unwrap_or(default_config_path);

    match &args.command {
        Some(Command::Init) => {
            let default_config = config::Config::default();
            let config_serialized = toml::to_string_pretty(&default_config)?;
            fs::write(&config_file, config_serialized)?;
        }
        Some(Command::Fmt { minify, dirs }) => {
            let formatter = Formatter::new(*minify);

            if dirs.is_empty() {
                eprintln!("Reading from stdin");
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;

                let output = formatter.run(&buffer)?;
                print!("{output}");
                io::stdout().flush()?;

                return Ok(());
            }

            dirs.par_iter().for_each(|d| {
                if d.is_dir() {
                    if let Err(e) = sql::fmt_recursively(d, *minify) {
                        eprintln!("{}: {e}", "Error".red());
                    }
                } else if d.is_file() {
                    let f = fs::read_to_string(d).unwrap();

                    let formatted = match formatter.run(&f) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!(
                                "{}: {e} ({:?})",
                                "Format Error".red(),
                                std::path::absolute(d).unwrap_or(d.to_path_buf())
                            );
                            std::process::exit(1);
                        }
                    };
                    fs::write(d, formatted).unwrap();
                    println!("{:?}", std::path::absolute(d).unwrap_or(d.to_path_buf()));
                }
            });
        }
        None => {
            if !config_file.exists() {
                return Err(anyhow!(
                    "no config file found at specified path: {:?}. Run {}.",
                    config_file,
                    "quaff init".yellow(),
                ));
            }

            let config_data = fs::read_to_string(&config_file)?;
            let config: config::Config = toml::from_str(&config_data)?;

            let dir_entries = fs::read_dir(cwd.join("sql"))?;
            let models_dir = cwd.join("models");
            fs::create_dir_all(&models_dir)?;

            for entry in dir_entries {
                let entry = entry?;
                if let Some(ext) = entry.path().extension() {
                    if ext != "sql" {
                        return Err(anyhow!(
                            "file {} does not have a valid extension",
                            entry.path().display()
                        ));
                    }
                } else {
                    return Err(anyhow!(
                        "file {} does not have a valid extension",
                        entry.path().display()
                    ));
                }

                let stmts = sql::parse_file(entry.path())?;

                let class = match config::LangOption::from_str(&config.lang) {
                    Ok(LangOption::PHP) => php::Class::new(stmts)?,
                    Err(e) => return Err(e),
                };

                let file_php = format!(
                    "{}.php",
                    entry
                        .file_name()
                        .into_string()
                        .unwrap()
                        .strip_suffix(".sql")
                        .unwrap_or("")
                );
                fs::write(
                    models_dir.join(&file_php),
                    format!("<?php\nnamespace Quaff;\nuse DateTime;\n\n{class}"),
                )?;
                println!(
                    "{}: {}",
                    "Wrote".green(),
                    models_dir.join(&file_php).display()
                );
            }
        }
    }

    Ok(())
}
