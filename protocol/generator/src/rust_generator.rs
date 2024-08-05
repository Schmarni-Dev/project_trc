use std::{borrow::Cow, collections::HashMap, fs, io::Write, path::Path};

use eyre::ContextCompat;

use crate::types::{Document, Type, VarientType};

pub fn generate_rust_file(
    output_dir: impl AsRef<Path>,
    document: &Document,
    use_map: &HashMap<String, String>,
) -> eyre::Result<()> {
    fs::create_dir_all(&output_dir)?;
    let mut path = output_dir.as_ref().to_path_buf();
    path.push(format!("{}.rs", document.name));
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    for u in document.uses.iter() {
        let u_path = use_map.get(u).context("use not in use_map")?;
        file.write_all(format!("use {};\n", u_path).as_bytes())?;
    }

    for struct_data in document.structs.iter() {
        match &struct_data.struct_type {
            VarientType::Struct(fields) => {
                let mut string =
                    "\n#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]\n".to_string();
                string += &format!("pub struct {} {{\n", struct_data.name);
                for f in fields.iter() {
                    string += &format!("\tpub {}: {},\n", f.name, type_to_rust_type(&f.type_name));
                }
                string += "}\n";
                file.write_all(string.as_bytes())?;
            }
            VarientType::Tuple(types) => {
                let mut string =
                    "\n#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]\n".to_string();
                string += &format!("pub struct {}(\n", struct_data.name);
                for t in types.iter() {
                    string += "\tpub ";
                    string += &type_to_rust_type(t);
                    string += ",\n"
                }
                string += ");\n";
                file.write_all(string.as_bytes())?;
            }
            VarientType::Simple => {
                file.write_all(format!("\n#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]\npub struct {};\n", struct_data.name).as_bytes())?;
            }
        }
    }
    for enum_data in document.enums.iter() {
        let mut string = format!("\npub enum {} {{\n", enum_data.name);
        for varient in enum_data.varients.iter() {
            match &varient.varient_type {
                VarientType::Struct(fields) => {
                    string += &format!("\t{} {{\n", varient.name);
                    for f in fields.iter() {
                        string +=
                            &format!("\t\t{}: {},\n", f.name, type_to_rust_type(&f.type_name));
                    }
                    string += "\t},\n";
                }
                VarientType::Tuple(types) => {
                    string += &format!("\t{}(\n", varient.name);
                    for t in types.iter() {
                        string += "\t\t";
                        string += &type_to_rust_type(t);
                        string += ",\n"
                    }
                    string += "\t),\n";
                }
                VarientType::Simple => {
                    string += &format!("\t{},\n", varient.name);
                }
            }
        }
        string += "}\n";
        file.write_all(string.as_bytes())?;
    }
    Ok(())
}

fn type_to_rust_type(type_data: &Type) -> Cow<'static, str> {
    match type_data {
        Type::F32 => "f32".into(),
        Type::F64 => "f64".into(),
        Type::U(s) => format!("u{}", s).into(),
        Type::I(s) => format!("i{}", s).into(),
        Type::String => "String".into(),
        Type::Bool => "bool".into(),
        Type::Custom(v) => v.to_string().into(),
        Type::Optional(v) => format!("Option<{}>", type_to_rust_type(v)).into(),
        Type::List(v) => format!("Vec<{}>", type_to_rust_type(v)).into(),
    }
}
