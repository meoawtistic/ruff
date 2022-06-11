use std::path::Path;
use std::{fs, path};

use crate::error::Error;
use crate::parser::remove_comments;

#[derive(Clone)]
pub struct File {
    pub path: String,
    pub content: String,
}

pub struct ReadFile {
    pub raw: String,
    pub files: Vec<File>,
}

const IMPORT_KEYWORD: &str = "include";
const FILE_EXTENSION: &str = ".ruff";

pub fn read_file(path: &str) -> Result<ReadFile, Error> {
    fn recurse(path: &Path) -> Result<Vec<File>, Error> {
        if !path.is_file() {
            return Err(Error::new("path not file"));
        }

        let mut content = remove_comments(&fs::read_to_string(path)?)?;
        let path_name = path.to_str().unwrap_or("").to_string();

        let mut v: Vec<File> = vec![];
        while content.contains(&format!("{} ", IMPORT_KEYWORD)) {
            let idx_im = content.find(&format!("{} ", IMPORT_KEYWORD)).unwrap();

            let idx_nl_1 = if idx_im == 0 {
                0
            } else {
                idx_im
                    - content[..idx_im]
                        .chars()
                        .rev()
                        .collect::<String>()
                        .find("\n")
                        .unwrap_or(idx_im)
            };

            let idx_nl_2 = idx_im + content[idx_im..].find("\n").unwrap_or(content.len());

            let import_statement = content[idx_nl_1..idx_nl_2].to_string();

            content = if idx_nl_1 == 0 {
                content[idx_nl_2 + 1..].to_string()
            } else {
                content[..idx_nl_1 - 1].to_string() + &content[idx_nl_2 + 1..]
            };

            let mut import_statement = import_statement.trim().split_whitespace();

            let next_file = match import_statement.next() {
                Some(s) => {
                    if s == IMPORT_KEYWORD {
                        if let Some(i) = import_statement.next() {
                            Some(i)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                None => None,
            };

            if next_file.is_none() {
                continue;
            }

            let next_file = match next_file {
                None => continue,
                Some(f) => {
                    f.to_string()
                        + if f.ends_with(FILE_EXTENSION) {
                            ""
                        } else {
                            FILE_EXTENSION
                        }
                }
            };

            let parent = path.parent().unwrap();
            let next_path = parent.join(next_file);

            if next_path != path
                && v.iter()
                    .find(|x| x.path == next_path.to_str().unwrap_or(""))
                    .is_none()
            {
                v.extend(recurse(next_path.as_ref())?);
            }
        }

        v.push(File {
            path: path_name,
            content,
        });
        Ok(v)
    }

    let files = recurse(Path::new(path))?;

    Ok(ReadFile {
        raw: files
            .iter()
            .map(|f| f.content.to_string())
            .collect::<Vec<String>>()
            .join(""),
        files,
    })
}

pub fn save_json(path: &str, content: &str) -> Result<(), Error> {
    let mut path = path::PathBuf::from(path);
    if path.extension().is_none() || path.extension().unwrap().to_str() != Some("json") {
        path.set_extension("json");
    }
    let content = format!(
        "{{
\t\"runtime_bytecode\": \"{}\"
}}",
        content
    );

    fs::write(&path, content)?;
    println!("output saved to: {}", path.to_str().unwrap());

    Ok(())
}
