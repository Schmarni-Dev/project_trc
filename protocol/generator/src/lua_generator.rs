use std::{borrow::Cow, collections::HashMap, fs, io::Write, path::Path};

use crate::types::{Document, Type, VarientType};

pub fn generate_rust_file(
    output_dir: impl AsRef<Path>,
    document: &Document,
    _use_map: &HashMap<String, String>,
) -> eyre::Result<()> {
    fs::create_dir_all(&output_dir)?;
    let mut path = output_dir.as_ref().to_path_buf();
    path.push(format!("{}.lua", document.name));
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all("---@meta\n".as_bytes())?;
    for struct_data in document.structs.iter() {
        match &struct_data.struct_type {
            VarientType::Struct(fields) => {
                let mut str = String::new();
                str += &format!("\n---@class {}", struct_data.name);
                str += "\n";
                for f in fields.iter() {
                    str += &format!("---@field {} {}", f.name, type_to_lua_type(&f.type_name));
                    str += "\n";
                }
                file.write_all(str.as_bytes())?;
            }
            VarientType::Tuple(types) => {
                let mut str = String::new();
                if types.len() != 1 {
                    str += &format!("\n---@alias {} [", struct_data.name);
                } else {
                    str += &format!("\n---@alias {} ", struct_data.name);
                }
                for f in types.iter() {
                    str += &format!("{},", type_to_lua_type(f));
                }
                if types.len() != 1 {
                    str += "]\n";
                } else {
                    str += "\n";
                }
                file.write_all(str.as_bytes())?;
            }
            VarientType::Simple => {
                file.write_all(
                    format!("---@alias {} \"{}\"", struct_data.name, struct_data.name).as_bytes(),
                )?;
            }
        }
    }
    for enum_data in document.enums.iter() {
        let mut str = format!("\n---@alias {}\n", enum_data.name);
        for varient in enum_data.varients.iter() {
            match &varient.varient_type {
                VarientType::Struct(fields) => {
                    str += &format!("---| {{ {}: {{", varient.name);
                    for f in fields.iter() {
                        str += &format!("{}: {}, ", f.name, type_to_lua_type(&f.type_name));
                    }
                    str += "} }";
                    str += "\n";
                }
                VarientType::Tuple(types) => {
                    str += "---| [";
                    for f in types.iter() {
                        str += &format!("{},", type_to_lua_type(f));
                    }
                    str += "]\n";
                }
                VarientType::Simple => {
                    str += &format!("---| \"{}\"", varient.name);
                }
            }
        }
        file.write_all(str.as_bytes())?;
    }
    Ok(())
}

fn type_to_lua_type(type_data: &Type) -> Cow<'static, str> {
    match type_data {
        Type::F32 => "number".into(),
        Type::F64 => "number".into(),
        Type::U(_) => "integer".into(),
        Type::I(_) => "integer".into(),
        Type::String => "string".into(),
        Type::Bool => "boolean".into(),
        Type::Custom(v) => v.to_string().into(),
        Type::Optional(v) => format!("{} | nil", type_to_lua_type(v)).into(),
    }
}
