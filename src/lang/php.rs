use anyhow::anyhow;
use sqlparser::ast::{ColumnOption, DataType, ExactNumberInfo, Statement};
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

use crate::lang::PascalCase;

#[derive(Debug, Default)]
pub struct Class {
    pub name: String,
    fields: Vec<Field>,
}

impl Class {
    pub fn new(stmts: Vec<Statement>) -> anyhow::Result<Self> {
        let mut class = Self::default();

        for stmt in stmts {
            if let Statement::CreateTable(table) = stmt {
                class.name = table.name.0.get(0).unwrap().to_string();

                let mut fields: Vec<Field> = vec![];

                for column in table.columns {
                    column.options.iter().for_each(|f| match &f.option {
                        ColumnOption::Unique {
                            is_primary,
                            characteristics,
                        } => {
                            _ = characteristics;
                            if *is_primary {
                                fields.push(Field {
                                    visibility: Visibilty::Private,
                                    type_annotation: Kind::from(&column.data_type),
                                    name: column.name.to_string(),
                                    nullable: false,
                                });
                            }
                        }
                        ColumnOption::Null => {
                            fields.push(Field {
                                visibility: Visibilty::Public,
                                type_annotation: Kind::from(&column.data_type),
                                name: column.name.to_string(),
                                nullable: true,
                            });
                        }
                        ColumnOption::NotNull => {
                            fields.push(Field {
                                visibility: Visibilty::Public,
                                type_annotation: Kind::from(&column.data_type),
                                name: column.name.to_string(),
                                nullable: false,
                            });
                        }
                        _ => {} // ColumnOption::Default(expr) => todo!(),
                                // ColumnOption::Materialized(expr) => todo!(),
                                // ColumnOption::Ephemeral(expr) => todo!(),
                                // ColumnOption::Alias(expr) => todo!(),
                                // ColumnOption::ForeignKey {
                                //     foreign_table,
                                //     referred_columns,
                                //     on_delete,
                                //     on_update,
                                //     characteristics,
                                // } => todo!("Hook into other files to build relationships"),
                                // ColumnOption::Check(expr) => todo!(),
                                // ColumnOption::DialectSpecific(tokens) => todo!(),
                                // ColumnOption::CharacterSet(object_name) => todo!(),
                                // ColumnOption::Collation(object_name) => todo!(),
                                // ColumnOption::Comment(_) => todo!(),
                                // ColumnOption::OnUpdate(expr) => todo!(),
                                // ColumnOption::Generated {
                                //     generated_as,
                                //     sequence_options,
                                //     generation_expr,
                                //     generation_expr_mode,
                                //     generated_keyword,
                                // } => todo!(),
                                // ColumnOption::Options(sql_options) => todo!(),
                                // ColumnOption::Identity(identity_property_kind) => todo!(),
                                // ColumnOption::OnConflict(keyword) => todo!(),
                                // ColumnOption::Policy(column_policy) => todo!(),
                                // ColumnOption::Tags(tags_column_option) => todo!(),
                    });
                }

                class.fields = fields;
            } else {
                return Err(anyhow!("SQL must be a create table statment"));
            }
        }

        Ok(class)
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fields = String::new();

        let len = self.fields.len();
        for (i, field) in self.fields.iter().enumerate() {
            if i + 1 == len {
                // no trailing comma
                fields.push_str(&format!("        {field}\n"));
            } else {
                fields.push_str(&format!("        {field},\n"));
            }
        }

        write!(
            f,
            "class {} {{\n    public function __construct(\n{fields}    ) {{ }}\n}}",
            self.name.to_pascal_case()
        )
    }
}

#[derive(Debug)]
pub struct Field {
    pub visibility: Visibilty,
    pub type_annotation: Kind,
    pub name: String,
    pub nullable: bool,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let null = if self.nullable { "?" } else { "" };

        write!(
            f,
            "{} {}{} ${}",
            self.visibility, null, self.type_annotation, self.name
        )
    }
}

#[derive(Debug)]
pub enum Visibilty {
    Public,
    Protected,
    Private,
}

impl Display for Visibilty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Visibilty::Public => "public",
            Visibilty::Protected => "protected",
            Visibilty::Private => "private",
        };

        write!(f, "{s}")
    }
}

impl FromStr for Visibilty {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            "protected" => Ok(Self::Protected),
            _ => Err(anyhow!("invalid visibility keyword: {s}")),
        }
    }
}

#[derive(Debug)]
pub enum Kind {
    Int,
    Float,
    Bool,
    String,
    Array,
    Object,
    Callable,
    Iterable,
    Mixed,
    Null,
    False,
    True,
    DateTime,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Kind::Int => "int",
            Kind::Float => "float",
            Kind::Bool => "bool",
            Kind::String => "string",
            Kind::Array => "array",
            Kind::Object => "object",
            Kind::Callable => "callable",
            Kind::Iterable => "iterable",
            Kind::Mixed => "mixed",
            Kind::Null => "null",
            Kind::False => "false",
            Kind::True => "true",
            Kind::DateTime => "DateTime",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Kind {
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "int" => Ok(Kind::Int),
            "float" => Ok(Kind::Float),
            "bool" => Ok(Kind::Bool),
            "string" => Ok(Kind::String),
            "array" => Ok(Kind::Array),
            "object" => Ok(Kind::Object),
            "callable" => Ok(Kind::Callable),
            "iterable" => Ok(Kind::Iterable),
            "mixed" => Ok(Kind::Mixed),
            "null" => Ok(Kind::Null),
            "false" => Ok(Kind::False),
            "true" => Ok(Kind::True),
            "DateTime" => Ok(Kind::DateTime),
            other => Err(TypeParseError::UnknownType(String::from(other))),
        }
    }
}

impl From<&DataType> for Kind {
    fn from(value: &DataType) -> Self {
        match value {
            DataType::Table(_column_defs) => Kind::Array,
            DataType::Character(_character_length) => Kind::String,
            DataType::Char(_character_length) => Kind::String,
            DataType::CharacterVarying(_character_length) => Kind::String,
            DataType::CharVarying(_character_length) => Kind::String,
            DataType::Varchar(_character_length) => Kind::String,
            DataType::Nvarchar(_character_length) => Kind::String,
            DataType::Uuid => Kind::String,
            DataType::CharacterLargeObject(_) => Kind::String,
            DataType::CharLargeObject(_) => Kind::String,
            DataType::Clob(_) => Kind::String,
            DataType::Binary(_) => Kind::Array,
            DataType::Varbinary(_binary_length) => Kind::Array,
            DataType::Blob(_) => Kind::Array,
            DataType::TinyBlob => Kind::Array,
            DataType::MediumBlob => Kind::Array,
            DataType::LongBlob => Kind::Array,
            DataType::Bytes(_) => Kind::Array,
            DataType::Numeric(exact_number_info) => {
                if let ExactNumberInfo::None = exact_number_info {
                    return Kind::Int;
                } else {
                    return Kind::String;
                }
            }
            DataType::Decimal(exact_number_info) => {
                if let ExactNumberInfo::None = exact_number_info {
                    return Kind::Float;
                } else {
                    return Kind::String;
                }
            }
            DataType::BigNumeric(exact_number_info) => {
                if let ExactNumberInfo::None = exact_number_info {
                    return Kind::Int;
                } else {
                    return Kind::String;
                }
            }
            DataType::BigDecimal(exact_number_info) => {
                if let ExactNumberInfo::None = exact_number_info {
                    return Kind::Float;
                } else {
                    return Kind::String;
                }
            }
            DataType::Dec(exact_number_info) => {
                if let ExactNumberInfo::None = exact_number_info {
                    return Kind::Float;
                } else {
                    return Kind::String;
                }
            }
            DataType::Float(_) => Kind::Float,
            DataType::TinyInt(_) => Kind::Int,
            DataType::TinyIntUnsigned(_) => Kind::Int,
            DataType::UTinyInt => Kind::Int,
            DataType::Int2(_) => Kind::Int,
            DataType::Int2Unsigned(_) => Kind::Int,
            DataType::SmallInt(_) => Kind::Int,
            DataType::SmallIntUnsigned(_) => Kind::Int,
            DataType::USmallInt => Kind::Int,
            DataType::MediumInt(_) => Kind::Int,
            DataType::MediumIntUnsigned(_) => Kind::Int,
            DataType::Int(_) => Kind::Int,
            DataType::Int4(_) => Kind::Int,
            DataType::Int8(_) => Kind::Int,
            DataType::Int16 => Kind::Int,
            DataType::Int32 => Kind::Int,
            DataType::Int64 => Kind::Int,
            DataType::Int128 => Kind::Int,
            DataType::Int256 => Kind::Int,
            DataType::Integer(_) => Kind::Int,
            DataType::IntUnsigned(_) => Kind::Int,
            DataType::Int4Unsigned(_) => Kind::Int,
            DataType::IntegerUnsigned(_) => Kind::Int,
            DataType::HugeInt => Kind::Int,
            DataType::UHugeInt => Kind::Int,
            DataType::UInt8 => Kind::Int,
            DataType::UInt16 => Kind::Int,
            DataType::UInt32 => Kind::Int,
            DataType::UInt64 => Kind::Int,
            DataType::UInt128 => Kind::Int,
            DataType::UInt256 => Kind::Int,
            DataType::BigInt(_) => Kind::Int,
            DataType::BigIntUnsigned(_) => Kind::Int,
            DataType::UBigInt => Kind::Int,
            DataType::Int8Unsigned(_) => Kind::Int,
            DataType::Signed => Kind::Int,
            DataType::SignedInteger => Kind::Int,
            DataType::Unsigned => Kind::Int,
            DataType::UnsignedInteger => Kind::Int,
            DataType::Float4 => Kind::Float,
            DataType::Float32 => Kind::Float,
            DataType::Float64 => Kind::Float,
            DataType::Real => Kind::Float,
            DataType::Float8 => Kind::Float,
            DataType::Double(exact_number_info) => {
                if let ExactNumberInfo::None = exact_number_info {
                    return Kind::Float;
                } else {
                    return Kind::String;
                }
            }
            DataType::DoublePrecision => Kind::Float,
            DataType::Bool => Kind::Bool,
            DataType::Boolean => Kind::Bool,
            DataType::Date => Kind::DateTime,
            DataType::Date32 => Kind::DateTime,
            DataType::Time(_, _timezone_info) => Kind::DateTime,
            DataType::Datetime(_) => Kind::DateTime,
            DataType::Datetime64(_, _) => Kind::DateTime,
            DataType::Timestamp(_, _timezone_info) => Kind::DateTime,
            DataType::TimestampNtz => Kind::DateTime,
            DataType::Interval => Kind::String,
            DataType::JSON => Kind::String,
            DataType::JSONB => Kind::Array,
            DataType::Regclass => Kind::String,
            DataType::Text => Kind::String,
            DataType::TinyText => Kind::String,
            DataType::MediumText => Kind::String,
            DataType::LongText => Kind::String,
            DataType::String(_) => Kind::String,
            DataType::FixedString(_) => Kind::String,
            DataType::Bytea => Kind::String,
            DataType::Bit(_) => Kind::String,
            DataType::BitVarying(_) => Kind::String,
            DataType::VarBit(_) => Kind::String,
            // Need to add better parsing for this at a later date.
            DataType::Custom(_object_name, _items) => Kind::Mixed,
            DataType::Array(_array_elem_type_def) => Kind::Mixed,
            DataType::Map(_data_type, _data_type1) => Kind::Mixed,
            DataType::Tuple(_struct_fields) => Kind::Mixed,
            DataType::Nested(_column_defs) => Kind::Mixed,
            DataType::Enum(_enum_members, _) => Kind::Mixed,
            DataType::Set(_items) => Kind::Mixed,
            DataType::Struct(_struct_fields, _struct_bracket_kind) => Kind::Mixed,
            DataType::Union(_union_fields) => Kind::Mixed,
            DataType::Nullable(_data_type) => Kind::Mixed,
            DataType::LowCardinality(_data_type) => Kind::Mixed,
            DataType::Unspecified => Kind::Mixed,
            DataType::Trigger => Kind::Mixed,
            DataType::AnyType => Kind::Mixed,
            DataType::GeometricType(_geometric_type_kind) => Kind::Mixed,
        }
    }
}

impl From<DataType> for Kind {
    fn from(value: DataType) -> Self {
        Kind::from(&value)
    }
}

#[derive(Debug, Error)]
pub enum TypeParseError {
    #[error("Unknown type: {0}")]
    UnknownType(String),
}
