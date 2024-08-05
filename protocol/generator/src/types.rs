use std::{fmt::Display, str::FromStr};

use eyre::{eyre, Context};

#[derive(Clone, Debug)]
pub struct Struct {
    pub name: String,
    pub struct_type: VarientType,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub type_name: Type,
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub name: String,
    pub varients: Vec<EnumVarient>,
}

#[derive(Clone, Debug)]
pub struct EnumVarient {
    pub name: String,
    pub varient_type: VarientType,
}

#[derive(Clone, Debug)]
pub enum VarientType {
    Struct(Vec<Field>),
    Tuple(Vec<Type>),
    Simple,
}

#[derive(Clone, Debug)]
pub enum Type {
    F32,
    F64,
    U(Size),
    I(Size),
    String,
    Bool,
    Custom(String),
    List(Box<Type>),
    Optional(Box<Type>),
}

#[derive(Clone, Debug)]
pub enum Size {
    Size,
    B64,
    B32,
    B16,
    B8,
}

pub struct Document {
    pub name: String,
    pub uses: Vec<String>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
}

impl Document {
    pub fn new(name: String) -> Document {
        Document {
            name,
            uses: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
        }
    }
}

impl FromStr for Type {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = Size::from_str(s) {
            return Ok(match s.split_at(1).0 {
                "u" => Self::U(v),
                "i" => Self::I(v),
                _ => unreachable!(),
            });
        }
        if let Some(s) = s.strip_suffix("?[]") {
            return Ok(Self::List(Box::new(Self::Optional(Box::new(
                Self::from_str(s)?,
            )))));
        };
        if let Some(s) = s.strip_suffix("[]?") {
            return Ok(Self::Optional(Box::new(Self::List(Box::new(
                Self::from_str(s)?,
            )))));
        };
        if let Some(s) = s.strip_suffix('?') {
            return Ok(Self::Optional(Box::new(Self::from_str(s)?)));
        };
        if let Some(s) = s.strip_suffix("[]") {
            return Ok(Self::List(Box::new(Self::from_str(s)?)));
        };
        Ok(match s {
            "bool" => Self::Bool,
            "f32" => Self::F32,
            "f64" => Self::F64,
            "String" => Self::String,
            val => Self::Custom(val.to_string()),
        })
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Size::Size => "size",
            Size::B64 => "64",
            Size::B32 => "32",
            Size::B16 => "16",
            Size::B8 => "8",
        };
        f.write_str(v)
    }
}

impl FromStr for Size {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ("i" | "u", size) = s.split_at(1) else {
            return Err(eyre!("unsupported type"));
        };
        if size == "size" {
            return Ok(Size::Size);
        }
        Ok(match size.parse::<u8>().context("unsupported type")? {
            8 => Size::B8,
            16 => Size::B16,
            32 => Size::B32,
            64 => Size::B64,
            size => return Err(eyre!("unsupported size: {}", size)),
        })
    }
}
