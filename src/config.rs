use std::collections::HashMap;
use std::fs::{metadata, read_to_string};
use std::io;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::rc::Rc;

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

    let question_names_tree = Rc::new(HashMap::new());

    let questions = json
        .questions
        .iter()
        .map(|raw_question| {
            let path = match QuestionPath::parse(raw_question.name) {
                Some(path) => path,
                None => {
                    eprintln!("{} is not a valid question name", raw_question.name);
                    return None;
                }
            };

            if *path.names().first().unwrap() == "__template__" {
                eprintln!(
                    "{} is not a valid question name (__template__)",
                    raw_question.name
                );
                return None;
            }

            {
                let mut question_names_tree = Rc::clone(&question_names_tree);

                if !check_duplicate_name(
                    Rc::get_mut(&mut question_names_tree).unwrap(),
                    &path.names(),
                ) {
                    eprintln!(
                        "{} is not a valid question name (conflict with nested property container)",
                        raw_question.name
                    );
                    return None;
                }
            }

            Some(Question {
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
                            return None;
                        }

                        QuestionSpec::Selection {
                            items,
                            multi_select: raw_question.multi.unwrap_or(false),
                        }
                    }
                },
            })
        })
        .filter_map(|it| it)
        .collect();

    Ok(Config {
        name: json.name.trim(),
        version: json.version.trim(),
        questions,
    })
}

fn check_duplicate_name<'cfg>(
    tree: &'cfg mut HashMap<&'cfg str, ValueMapItem<'cfg>>,
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
            tree.insert(first_name, ValueMapItem::Value(first_name));

            true
        } else {
            {
                tree.insert(first_name, ValueMapItem::Map(HashMap::new()));
            }

            let map = match tree.get_mut(first_name).unwrap() {
                ValueMapItem::Map(map) => map,
                _ => panic!(),
            };

            check_duplicate_name(map, &names[1..])
        }
    }
}

#[derive(Deserialize)]
struct ConfigJson<'cfg> {
    name: &'cfg str,
    version: &'cfg str,
    questions: Vec<RawQuestion<'cfg>>,
}

#[derive(Deserialize)]
struct RawQuestion<'cfg> {
    name: &'cfg str,
    #[serde(rename(deserialize = "type"))]
    question_type: RawQuestionType,
    pretty: Option<&'cfg str>,
    items: Option<Vec<&'cfg str>>,
    multi: Option<bool>,
}

#[derive(Deserialize)]
enum RawQuestionType {
    Identifier,
    Option,
    Selection,
    Text,
}

enum ValueMapItem<'cfg> {
    Map(HashMap<&'cfg str, ValueMapItem<'cfg>>),
    Value(&'cfg str),
}

#[derive(Serialize)]
pub struct Config<'cfg> {
    pub name: &'cfg str,
    pub version: &'cfg str,
    #[serde(skip)]
    pub questions: Vec<Question<'cfg>>,
}

pub struct Question<'cfg> {
    pub path: QuestionPath<'cfg>,
    pub pretty: Option<&'cfg str>,
    pub spec: QuestionSpec<'cfg>,
}

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

    pub fn names(self: &Self) -> &Vec<&'cfg str> {
        &self.names
    }
}

pub enum QuestionSpec<'cfg> {
    Identifier,
    Option,
    Selection {
        items: Vec<&'cfg str>,
        multi_select: bool,
    },
    Text,
}
