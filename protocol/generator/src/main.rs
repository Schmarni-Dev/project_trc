use std::{
    collections::HashMap,
    fs::{self, ReadDir},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Fmt,
    Generate,
}

pub struct RecursiveDirIter(Vec<ReadDir>);
impl RecursiveDirIter {
    fn new(path: impl AsRef<Path>) -> RecursiveDirIter {
        Self(vec![fs::read_dir(path).unwrap()])
    }
}
impl Iterator for RecursiveDirIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let w = match self.0.last_mut()?.next() {
            Some(f) => f.ok()?,
            None => {
                self.0.pop();
                return self.next();
            }
        };
        if w.metadata().ok()?.is_file() {
            return Some(w.path());
        }
        if w.metadata().ok()?.is_dir() {
            match fs::read_dir(w.path()) {
                Ok(v) => {
                    self.0.push(v);
                }
                Err(err) => {
                    println!("ERROR: {}", err);
                }
            }
            return self.next();
        }
        None
    }
}

fn main() {
    let args = Args::parse();
    match args.cmd {
        Command::Fmt => {
            for path in RecursiveDirIter::new("protocol/defenitions") {
                format_kdl_file(&path).unwrap();
            }
        }
        Command::Generate => {
            for path in RecursiveDirIter::new("protocol/defenitions") {
                let doc = trc_protocol_gen::parse_kdl_file(path).unwrap();
                trc_protocol_gen::rust_generator::generate_rust_file(
                    "protocol/src/generated",
                    &doc,
                    &HashMap::new(),
                )
                .unwrap();

                trc_protocol_gen::lua_generator::generate_rust_file(
                    "protocol/lua/generated",
                    &doc,
                    &HashMap::new(),
                )
                .unwrap()
            }
        }
    }
}

fn format_kdl_file(path: &Path) -> color_eyre::Result<()> {
    let str = fs::read_to_string(path)?;
    let mut document = kdl::KdlDocument::from_str(&str)?;
    document.fmt();
    let fmt_str = document.to_string();
    fs::write(path, fmt_str)?;
    Ok(())
}
