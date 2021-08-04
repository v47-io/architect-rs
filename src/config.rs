use std::collections::HashMap;
use std::fs::{metadata, read_to_string};
use std::io;
use std::io::{Error, ErrorKind};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::utils::is_identifier;

pub fn load_config_file(base_path: &Path, verbose: bool) -> io::Result<Option<String>> {
    let config_file_path = base_path.join(".architect.json");

    if verbose {
        println!(
            "Loading config file from path {}",
            config_file_path.display()
        )
    }

    if let Ok(_) = metadata(&config_file_path) {
        Ok(Some(read_to_string(config_file_path)?))
    } else {
        Ok(None)
    }
}

pub fn read_config(input: &str) -> io::Result<Config> {
    let json: ConfigJson = serde_json::from_str(input)?;

    if json.name.trim().is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "The template name cannot be empty",
        ));
    }

    let mut question_names_tree = HashMap::new();
    let mut questions = Vec::new();

    for raw_question in json.questions.iter() {
        let path = match QuestionPath::parse(raw_question.name) {
            Some(path) => path,
            None => {
                eprintln!("{} is not a valid question name", raw_question.name);
                continue;
            }
        };

        if *path.names().first().unwrap() == "__template__" {
            eprintln!(
                "{} is not a valid question name (__template__)",
                raw_question.name
            );

            continue;
        }

        if !check_duplicate_name(&mut question_names_tree, &path.names()) {
            eprintln!(
                "{} is not a valid question name (conflict with nested property container)",
                raw_question.name
            );

            continue;
        }

        questions.push(Question {
            path,
            pretty: match raw_question.pretty {
                Some(pretty) => {
                    if pretty.trim().is_empty() {
                        None
                    } else {
                        Some(pretty.trim())
                    }
                }
                None => None,
            },
            spec: match raw_question.question_type {
                RawQuestionType::Identifier => QuestionSpec::Identifier,
                RawQuestionType::Option => QuestionSpec::Option,
                RawQuestionType::Text => QuestionSpec::Text,
                RawQuestionType::Selection => {
                    let items = if let Some(raw_items) = &raw_question.items {
                        raw_items
                            .iter()
                            .map(|&item| item.trim())
                            .filter(|&it| is_identifier(it))
                            .collect()
                    } else {
                        vec![]
                    };

                    if items.is_empty() {
                        eprintln!("Question {} doesn't specify any items", raw_question.name);
                        continue;
                    }

                    QuestionSpec::Selection {
                        items,
                        multi_select: raw_question.multi.unwrap_or(false),
                    }
                }
            },
        });
    }

    Ok(Config {
        name: json.name.trim(),
        version: json.version.trim(),
        questions,
    })
}

fn check_duplicate_name<'cfg>(
    tree: &'cfg mut HashMap<String, ValueMapItem>,
    names: &'cfg [&'cfg str],
) -> bool {
    let first_name = *names.first().unwrap();

    if tree.contains_key(first_name) {
        // Name found in tree
        match tree.get_mut(first_name).unwrap() {
            ValueMapItem::Map(map) => {
                if names.len() == 1 {
                    // Can't proceed, conflict with Map
                    false
                } else {
                    // Name and tree refer to map, we can continue
                    check_duplicate_name(map, &names[1..])
                }
            }
            ValueMapItem::Value(_) => {
                // Duplicate item, can't proceed
                false
            }
        }
    } else {
        // Name not found in tree, we populate, my brothers
        if names.len() == 1 {
            tree.insert(
                first_name.to_string(),
                ValueMapItem::Value(first_name.to_string()),
            );

            true
        } else {
            {
                tree.insert(first_name.to_string(), ValueMapItem::Map(HashMap::new()));
            }

            let map = match tree.get_mut(first_name).unwrap() {
                ValueMapItem::Map(map) => map,
                _ => panic!(),
            };

            check_duplicate_name(map, &names[1..])
        }
    }
}

#[derive(Deserialize, Serialize)]
struct ConfigJson<'cfg> {
    name: &'cfg str,
    version: &'cfg str,
    questions: Vec<RawQuestion<'cfg>>,
}

#[derive(Deserialize, Serialize)]
struct RawQuestion<'cfg> {
    name: &'cfg str,
    #[serde(rename(deserialize = "type", serialize = "type"))]
    question_type: RawQuestionType,
    pretty: Option<&'cfg str>,
    items: Option<Vec<&'cfg str>>,
    multi: Option<bool>,
}

#[derive(Deserialize, Serialize)]
enum RawQuestionType {
    Identifier,
    Option,
    Selection,
    Text,
}

enum ValueMapItem {
    Map(HashMap<String, ValueMapItem>),
    Value(String),
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Config<'cfg> {
    pub name: &'cfg str,
    pub version: &'cfg str,
    #[serde(skip)]
    pub questions: Vec<Question<'cfg>>,
}

#[derive(Debug, PartialEq)]
pub struct Question<'cfg> {
    pub path: QuestionPath<'cfg>,
    pub pretty: Option<&'cfg str>,
    pub spec: QuestionSpec<'cfg>,
}

#[derive(Debug, PartialEq)]
pub struct QuestionPath<'cfg> {
    names: Vec<&'cfg str>,
}

impl<'cfg> QuestionPath<'cfg> {
    pub fn parse(name: &'cfg str) -> Option<Self> {
        let names: Vec<&'cfg str> = name.trim().split('.').collect();

        if names.is_empty() || names.iter().any(|&it| !is_identifier(it)) {
            None
        } else {
            Some(QuestionPath { names })
        }
    }

    pub fn names(&self) -> &Vec<&'cfg str> {
        &self.names
    }
}

#[derive(Debug, PartialEq)]
pub enum QuestionSpec<'cfg> {
    Identifier,
    Option,
    Selection {
        items: Vec<&'cfg str>,
        multi_select: bool,
    },
    Text,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    const CONFIG_CONTENT: &str = r#"{
    "name": "Some Template",
    "version": "1.0.0"
}
"#;

    #[test]
    fn test_load_config_file() {
        let working_dir = tempdir().unwrap();

        assert_eq!(load_config_file(working_dir.path(), true).unwrap(), None);

        fs::write(working_dir.path().join(".architect.json"), CONFIG_CONTENT).unwrap();

        assert_eq!(
            load_config_file(working_dir.path(), true).unwrap(),
            Some(CONFIG_CONTENT.to_string())
        );
    }

    #[test]
    fn test_read_config() {
        let config_json = serde_json::to_string_pretty(&ConfigJson {
            name: "Some Template",
            version: "0.1.0",
            questions: vec![
                RawQuestion {
                    name: "author",
                    pretty: Some("Who is the author of this project?"),
                    question_type: RawQuestionType::Text,
                    items: None,
                    multi: None,
                },
                RawQuestion {
                    name: "debug",
                    question_type: RawQuestionType::Option,
                    pretty: None,
                    items: None,
                    multi: None,
                },
                RawQuestion {
                    name: "main.package",
                    question_type: RawQuestionType::Identifier,
                    pretty: None,
                    items: None,
                    multi: None,
                },
                RawQuestion {
                    name: "main.features",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec!["feature_1", "feature_2", "feature_3"]),
                    multi: Some(true),
                    pretty: None,
                },
            ],
        })
        .unwrap();

        let config = read_config(&config_json).unwrap();

        assert_eq!(
            config,
            Config {
                name: "Some Template",
                version: "0.1.0",
                questions: vec![
                    Question {
                        path: QuestionPath {
                            names: vec!["author"]
                        },
                        pretty: Some("Who is the author of this project?"),
                        spec: QuestionSpec::Text
                    },
                    Question {
                        path: QuestionPath {
                            names: vec!["debug"]
                        },
                        spec: QuestionSpec::Option,
                        pretty: None
                    },
                    Question {
                        path: QuestionPath {
                            names: vec!["main", "package"]
                        },
                        spec: QuestionSpec::Identifier,
                        pretty: None
                    },
                    Question {
                        path: QuestionPath {
                            names: vec!["main", "features"]
                        },
                        spec: QuestionSpec::Selection {
                            items: vec!["feature_1", "feature_2", "feature_3"],
                            multi_select: true
                        },
                        pretty: None
                    }
                ]
            }
        )
    }
}
