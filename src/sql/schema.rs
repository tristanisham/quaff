use std::{fmt::Display, str::FromStr};

use sqlparser::parser::Parser;


pub struct Formatter {
    pub minify: bool,
}

impl Default for Formatter {
    fn default() -> Self {
        Self { minify: false }
    }
}

impl Formatter {
    pub fn new(minify: bool) -> Self {
        Self { minify }
    }

    pub fn run(&self, sql: &str) -> anyhow::Result<String> {
        let dialect = sqlparser::dialect::GenericDialect {};
        let ast = Parser::parse_sql(&dialect, &sql)?;

        let mut buffer = String::new();
        for (i, node) in ast.iter().enumerate() {
            let formatted = match self.minify {
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

            if ast.len() > 1 && i < ast.len() - 1 {
                buffer.push_str("\n\n");
            }
        }

        Ok(buffer)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RowCase {
    CamelCase,
    SnakeCase,
    PascalCase,
}

impl Display for RowCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for RowCase {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "camel" => Ok(Self::CamelCase),
            "snake" => Ok(Self::SnakeCase),
            "pascal" => Ok(Self::PascalCase),
            _ => Err(format!("Unknown case: {s}")),
        }
    }
}