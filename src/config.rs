/*
 * BSD 3-Clause License
 *
 * Copyright (c) ${year}, Alex Katlein
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice, this
 *    list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its
 *    contributors may be used to endorse or promote products derived from
 *    this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

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

    let mut context_tree = HashMap::new();

    let questions = json
        .questions
        .unwrap_or(vec![])
        .iter()
        .filter_map(|raw_question| {
            let path = match QuestionPath::parse(raw_question.name) {
                Some(path) => path,
                None => {
                    eprintln!("\"{}\" is not a valid question name", raw_question.name);
                    return None;
                }
            };

            if *path.names().first().unwrap() == "__template__" {
                eprintln!(
                    "\"{}\" is not a valid question name (__template__)",
                    raw_question.name
                );

                return None;
            }

            if !check_context_tree(&mut context_tree, &path.names()) {
                eprintln!(
                    "\"{}\" is not a valid question name (cannot add children to value)",
                    raw_question.name
                );

                return None;
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
                            multi: raw_question.multi.unwrap_or(false),
                        }
                    }
                },
            })
        })
        .collect();

    Ok(Config {
        name: json.name.trim(),
        version: json.version.trim(),
        questions,
    })
}

fn check_context_tree<'cfg>(
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
                    check_context_tree(map, &names[1..])
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

            check_context_tree(map, &names[1..])
        }
    }
}

#[derive(Deserialize, Serialize)]
struct ConfigJson<'cfg> {
    name: &'cfg str,
    version: &'cfg str,
    questions: Option<Vec<RawQuestion<'cfg>>>,
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
    Selection { items: Vec<&'cfg str>, multi: bool },
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
            questions: Some(vec![
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
            ]),
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
                            multi: true
                        },
                        pretty: None
                    }
                ]
            }
        );

        assert_eq!(
            read_config(r#"{ "name": "Some Template", "version": "" }"#).unwrap(),
            Config {
                name: "Some Template",
                version: "",
                questions: vec![]
            }
        )
    }

    #[test]
    fn test_read_config_failures() {
        let empty_name_error = read_config(r#"{ "name": "  ", "version": "" }"#);

        assert!(empty_name_error.is_err());
        assert_eq!(
            empty_name_error.unwrap_err().to_string(),
            "The template name cannot be empty"
        );

        let malformed_names_json = serde_json::to_string_pretty(&ConfigJson {
            name: "Some Template",
            version: "0.1.0",
            questions: Some(vec![
                RawQuestion {
                    name: "$author",
                    pretty: Some("Who is the author of this project?"),
                    question_type: RawQuestionType::Text,
                    items: None,
                    multi: None,
                },
                RawQuestion {
                    name: "1.debug",
                    question_type: RawQuestionType::Option,
                    pretty: None,
                    items: None,
                    multi: None,
                },
                RawQuestion {
                    name: "main..package",
                    question_type: RawQuestionType::Identifier,
                    pretty: None,
                    items: None,
                    multi: None,
                },
                RawQuestion {
                    name: "",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec!["feature_1", "feature_2", "feature_3"]),
                    multi: Some(true),
                    pretty: None,
                },
                RawQuestion {
                    name: "__template__.something",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec!["feature_1", "feature_2", "feature_3"]),
                    multi: Some(true),
                    pretty: None,
                },
            ]),
        })
        .unwrap();

        assert_eq!(
            read_config(&malformed_names_json).unwrap(),
            Config {
                name: "Some Template",
                version: "0.1.0",
                questions: vec![]
            }
        );

        let malformed_context_tree = serde_json::to_string_pretty(&ConfigJson {
            name: "Some Template",
            version: "0.1.0",
            questions: Some(vec![
                RawQuestion {
                    name: "author",
                    pretty: Some("Who is the author of this project?"),
                    question_type: RawQuestionType::Text,
                    items: None,
                    multi: None,
                },
                RawQuestion {
                    name: "author.email",
                    question_type: RawQuestionType::Identifier,
                    pretty: None,
                    items: None,
                    multi: None,
                },
                RawQuestion {
                    name: "author.email.domain",
                    question_type: RawQuestionType::Identifier,
                    pretty: None,
                    items: None,
                    multi: None,
                },
            ]),
        })
        .unwrap();

        assert_eq!(
            read_config(&malformed_context_tree).unwrap(),
            Config {
                name: "Some Template",
                version: "0.1.0",
                questions: vec![Question {
                    path: QuestionPath {
                        names: vec!["author"]
                    },
                    pretty: Some("Who is the author of this project?"),
                    spec: QuestionSpec::Text
                },]
            }
        );

        let malformed_selection_items = serde_json::to_string_pretty(&ConfigJson {
            name: "Some Template",
            version: "0.1.0",
            questions: Some(vec![
                RawQuestion {
                    name: "features1",
                    question_type: RawQuestionType::Selection,
                    items: None,
                    pretty: None,
                    multi: Some(true),
                },
                RawQuestion {
                    name: "features2",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec!["$feature1", "feature2", "abc.def"]),
                    pretty: None,
                    multi: None,
                },
                RawQuestion {
                    name: "features3",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec![]),
                    pretty: None,
                    multi: None,
                },
            ]),
        })
        .unwrap();

        assert_eq!(
            read_config(&malformed_selection_items).unwrap(),
            Config {
                name: "Some Template",
                version: "0.1.0",
                questions: vec![Question {
                    path: QuestionPath {
                        names: vec!["features2"]
                    },
                    spec: QuestionSpec::Selection {
                        items: vec!["feature2"],
                        multi: false
                    },
                    pretty: None
                }]
            }
        )
    }
}
