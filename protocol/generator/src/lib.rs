pub mod rust_generator;
pub mod types;
pub mod generated;
pub mod config;
pub mod lua_generator;

use std::{fs, io::Read, path::Path, str::FromStr};

use eyre::{eyre, Context, ContextCompat, OptionExt};
use types::{Document, Struct};

use crate::types::{Enum, EnumVarient, Field, Type, VarientType};

pub fn parse_kdl_file(path: impl AsRef<Path>) -> eyre::Result<Document> {
    let mut file = fs::File::open(&path)?;
    let name = path
        .as_ref()
        .file_stem()
        .ok_or_eyre("no file name")?
        .to_str()
        .ok_or_eyre("file OsStr is not valid utf8")?;
    let mut file_string = "".to_string();
    file.read_to_string(&mut file_string)?;
    let kdl_document = kdl::KdlDocument::from_str(&file_string).wrap_err(format!(
        "error with file: {}",
        path.as_ref().to_string_lossy()
    ))?;
    let mut document = Document::new(name.to_string());
    for node in kdl_document.nodes().iter() {
        println!("value {}", node.name().value());
        match node.name().value() {
            "struct" => {
                document.structs.push(struct_from_node(node)?);
            }
            "enum" => {
                document.enums.push(enum_from_node(node)?);
            }
            "use" => {
                let uses = node
                    .entries()
                    .iter()
                    .map(|entry| entry.value().to_string())
                    .collect::<Vec<String>>();
                document.uses.extend(uses);
            }
            name => Err(eyre!("unkown ident: {name}"))?,
        }
    }
    Ok(document)
}

fn enum_from_node(node: &kdl::KdlNode) -> eyre::Result<Enum> {
    let name = node
        .entries()
        .first()
        .wrap_err("no name")?
        .value()
        .as_string()
        .wrap_err("name is not a string")?;
    let Some(children) = node.children() else {
        return Ok(Enum {
            name: name.to_string(),
            varients: Vec::new(),
        });
    };
    let mut varients: Vec<EnumVarient> = Vec::new();
    for node in children.nodes().iter() {
        let name = node.name().value();
        let has_fields = node.children().is_some();
        let has_types = !node.entries().is_empty();
        let varient = match (has_types, has_fields) {
            (true, false) => {
                let types = node
                    .entries()
                    .iter()
                    .map(|entry| {
                        match entry
                            .value()
                            .as_string()
                            .context("type is not a tring")
                            .map(Type::from_str)
                        {
                            Ok(val @ Ok(_)) => val,
                            Err(err) => Err(err),
                            Ok(err @ Err(_)) => err,
                        }
                    })
                    .collect::<eyre::Result<Vec<Type>>>()?;
                EnumVarient {
                    name: name.to_string(),
                    varient_type: VarientType::Tuple(types),
                }
            }
            (false, true) => {
                let fields = parse_fields_from_node(node);
                EnumVarient {
                    name: name.to_string(),
                    varient_type: VarientType::Struct(fields?),
                }
            }
            (false, false) => EnumVarient {
                name: name.to_string(),
                varient_type: VarientType::Simple,
            },
            (true, true) => return Err(eyre!("Both Types and fields specified")),
        };
        varients.push(varient);
    }

    Ok(Enum {
        name: name.to_string(),
        varients,
    })
}

fn parse_fields_from_node(node: &kdl::KdlNode) -> eyre::Result<Vec<Field>> {
    let fields = node.children().cloned().map(|doc| {
        doc.into_iter()
            .map(|node| -> eyre::Result<Field> {
                Ok(Field {
                    name: node.name().value().to_string(),
                    type_name: Type::from_str(
                        node.entries()
                            .first()
                            .wrap_err("no type defined")?
                            .value()
                            .as_string()
                            .wrap_err("type name is not a string")?,
                    )?,
                })
            })
            .collect::<eyre::Result<Vec<Field>>>()
    });
    match fields {
        Some(fields) => fields,
        None => Ok(Vec::new()),
    }
}

fn struct_from_node(node: &kdl::KdlNode) -> eyre::Result<Struct> {
    let mut entry_iter = node.entries().iter();
    let name = entry_iter
        .next()
        .wrap_err("no struct name")?
        .value()
        .as_string()
        .wrap_err("name is not a string")?;
    let fields = parse_fields_from_node(node)?;
    let entries = entry_iter.collect::<Vec<_>>();
    if !entries.is_empty() {
        let types = entries
            .into_iter()
            .map(|entry| -> eyre::Result<_> {
                Type::from_str(entry.value().as_string().wrap_err("entry not a type")?)
            })
            .collect::<eyre::Result<_>>()?;
        return Ok(Struct {
            name: name.to_string(),
            struct_type: VarientType::Tuple(types),
        });
    }
    if fields.is_empty() {
        return Ok(Struct {
            name: name.to_string(),
            struct_type: VarientType::Simple,
        });
    }
    Ok(Struct {
        name: name.to_string(),
        struct_type: VarientType::Struct(fields),
    })
}
