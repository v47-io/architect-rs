/*
 * BSD 3-Clause License
 *
 * Copyright (c) 2021, Alex Katlein
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

use globset::GlobMatcher;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils::{glob, is_identifier, ToolConfig};

pub fn load_config_file(base_path: &Path, tool_config: &ToolConfig) -> io::Result<Option<String>> {
    let config_file_path = base_path.join(".architect.json");

    if tool_config.verbose {
        println!(
            "Loading config file from path {}",
            config_file_path.display()
        )
    }

    if metadata(&config_file_path).is_ok() {
        Ok(Some(read_to_string(config_file_path)?))
    } else {
        Ok(None)
    }
}

pub fn read_config<'cfg>(input: &'cfg str, tool_config: &ToolConfig) -> io::Result<Config<'cfg>> {
    let json: ConfigJson = serde_json::from_str(input)?;

    let mut context_tree = HashMap::new();

    let questions = json
        .questions
        .unwrap_or_default()
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

            if !check_context_tree(&mut context_tree, path.names()) {
                eprintln!(
                    "\"{}\" is not a valid question name (cannot add children to value)",
                    raw_question.name
                );

                return None;
            }

            let default_value = match read_default_value(
                raw_question,
                matches!(raw_question.question_type, RawQuestionType::Identifier | RawQuestionType::Selection),
            ) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("{}", err);

                    if tool_config.ignore_checks {
                        None
                    } else {
                        return None;
                    }
                }
            };

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
                    RawQuestionType::Identifier => QuestionSpec::Identifier {
                        default: get_default_str(default_value),
                    },
                    RawQuestionType::Option => QuestionSpec::Option {
                        default: get_default_bool(default_value),
                    },
                    RawQuestionType::Text => QuestionSpec::Text {
                        default: get_default_str(default_value),
                    },
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

                        let default = get_default_str_list(default_value);
                        let mut default = if default.iter().any(|item| !items.contains(&&**item)) {
                            eprintln!(
                                "Default value for question {} contains unknown items",
                                raw_question.name
                            );

                            if tool_config.ignore_checks {
                                vec![]
                            } else {
                                return None;
                            }
                        } else {
                            default
                        };

                        let multi = raw_question.multi.unwrap_or(false);

                        let default = if !multi && default.len() > 1 {
                            eprintln!("Multiple default values specified, but selection doesn't allow multiple selection");

                            vec![default.remove(0)]
                        } else {
                            default
                        };

                        QuestionSpec::Selection {
                            items,
                            multi,
                            default,
                        }
                    }
                },
            })
        })
        .collect();

    Ok(Config {
        name: json.name.map(|it| it.trim()),
        version: json.version.map(|it| it.trim()),
        questions,
        filters: json
            .filters
            .map(read_filters)
            .unwrap_or_else(Filters::empty),
    })
}

fn read_filters(raw_filters: RawFilters) -> Filters {
    let cond_files_specs = raw_filters
        .conditional_files
        .unwrap_or_default()
        .iter()
        .filter_map(|raw_cond_templates| {
            if raw_cond_templates.matcher.trim().is_empty() {
                eprintln!(
                    "Matcher for condition {} is blank",
                    raw_cond_templates.condition
                );
                return None;
            }

            if raw_cond_templates.condition.trim().is_empty() {
                eprintln!(
                    "Condition for matcher {} is blank",
                    raw_cond_templates.matcher
                );
                return None;
            }

            match glob(raw_cond_templates.matcher) {
                Ok(matcher) => Some(ConditionalFilesSpec {
                    condition: raw_cond_templates.condition.trim(),
                    matcher,
                }),
                Err(e) => {
                    eprintln!(
                        "Failed to parse glob expression {} ({})",
                        raw_cond_templates.matcher, e
                    );
                    None
                }
            }
        })
        .collect();

    Filters {
        conditional_files: cond_files_specs,
        include_hidden: map_glob_matchers(raw_filters.include_hidden.as_ref()).unwrap_or_default(),
        exclude: map_glob_matchers(raw_filters.exclude.as_ref()).unwrap_or_default(),
        templates: map_glob_matchers(raw_filters.templates.as_ref()),
        non_templates: map_glob_matchers(raw_filters.non_templates.as_ref()),
    }
}

fn map_glob_matchers(raw: Option<&Vec<&str>>) -> Option<Vec<GlobMatcher>> {
    let result = raw
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|&raw_glob| match glob(raw_glob) {
            Ok(matcher) => Some(matcher),
            Err(e) => {
                eprintln!("Failed to parse glob expression {} ({})", raw_glob, e);
                None
            }
        })
        .collect::<Vec<_>>();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
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

fn read_default_value(
    question: &RawQuestion,
    must_be_identifier: bool,
) -> io::Result<Option<Value>> {
    match question.default.as_ref() {
        Some(value) => match question.question_type {
            RawQuestionType::Option => match value {
                it @ Value::Bool(_) => Ok(Some(it.clone())),
                _ => Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid default value for 'Option': {}", value),
                )),
            },
            RawQuestionType::Selection => match value {
                it @ Value::String(value) => {
                    if must_be_identifier && !is_identifier(value) {
                        Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Default value is not an identifier: {}", value),
                        ))
                    } else {
                        Ok(Some(it.clone()))
                    }
                }
                it @ Value::Array(list) => {
                    if list.iter().any(|item| {
                        if let Value::String(value) = item {
                            must_be_identifier && !is_identifier(value)
                        } else {
                            true
                        }
                    }) {
                        Err(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "Invalid default value, contains non-{}: {:?}",
                                if must_be_identifier {
                                    "identifiers"
                                } else {
                                    "strings"
                                },
                                list
                            ),
                        ))
                    } else {
                        Ok(Some(it.clone()))
                    }
                }
                _ => Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid default value for 'Selection': {}", value),
                )),
            },
            _ => match value {
                it @ Value::String(value) => {
                    if must_be_identifier && !is_identifier(value) {
                        Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Default value is not an identifier: {}", value),
                        ))
                    } else {
                        Ok(Some(it.clone()))
                    }
                }
                _ => Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Invalid default value for '{:?}': {}",
                        question.question_type, value
                    ),
                )),
            },
        },
        None => Ok(None),
    }
}

fn get_default_str(value: Option<Value>) -> Option<String> {
    value.map(|it| {
        if let Value::String(str) = it {
            str
        } else {
            panic!()
        }
    })
}

fn get_default_str_list(value: Option<Value>) -> Vec<String> {
    value
        .map(|it| match it {
            Value::String(str) => vec![str],
            Value::Array(list) => list
                .into_iter()
                .map(|item| match item {
                    Value::String(str) => str,
                    _ => panic!(),
                })
                .collect(),
            _ => panic!(),
        })
        .unwrap_or_default()
}

fn get_default_bool(value: Option<Value>) -> Option<bool> {
    value.map(|it| {
        if let Value::Bool(value) = it {
            value
        } else {
            panic!()
        }
    })
}

#[derive(Deserialize, Serialize)]
struct ConfigJson<'cfg> {
    name: Option<&'cfg str>,
    version: Option<&'cfg str>,
    questions: Option<Vec<RawQuestion<'cfg>>>,
    filters: Option<RawFilters<'cfg>>,
}

#[derive(Deserialize, Serialize)]
struct RawQuestion<'cfg> {
    name: &'cfg str,
    #[serde(rename(deserialize = "type", serialize = "type"))]
    question_type: RawQuestionType,
    pretty: Option<&'cfg str>,
    items: Option<Vec<&'cfg str>>,
    multi: Option<bool>,
    default: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
enum RawQuestionType {
    Identifier,
    Option,
    Selection,
    Text,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(deserialize = "'de: 'cfg"))]
struct RawFilters<'cfg> {
    #[serde(rename(
        deserialize = "conditionalTemplates",
        serialize = "conditionalTemplates"
    ))]
    conditional_files: Option<Vec<RawConditionalFiles<'cfg>>>,
    #[serde(rename(deserialize = "includeHidden", serialize = "includeHidden"))]
    include_hidden: Option<Vec<&'cfg str>>,
    exclude: Option<Vec<&'cfg str>>,
    templates: Option<Vec<&'cfg str>>,
    #[serde(rename(deserialize = "nonTemplates", serialize = "nonTemplates"))]
    non_templates: Option<Vec<&'cfg str>>,
}

impl Default for RawFilters<'_> {
    fn default() -> Self {
        RawFilters {
            conditional_files: None,
            include_hidden: None,
            exclude: None,
            templates: None,
            non_templates: None,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct RawConditionalFiles<'cfg> {
    condition: &'cfg str,
    matcher: &'cfg str,
}

enum ValueMapItem {
    Map(HashMap<String, ValueMapItem>),
    Value(String),
}

#[derive(Debug, Serialize)]
pub struct Config<'cfg> {
    pub name: Option<&'cfg str>,
    pub version: Option<&'cfg str>,
    #[serde(skip)]
    pub questions: Vec<Question<'cfg>>,
    #[serde(skip)]
    pub filters: Filters<'cfg>,
}

impl<'cfg> Config<'cfg> {
    pub(crate) fn empty() -> Self {
        Config {
            name: None,
            version: None,
            questions: vec![],
            filters: Filters::empty(),
        }
    }
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
    Identifier {
        default: Option<String>,
    },
    Option {
        default: Option<bool>,
    },
    Selection {
        items: Vec<&'cfg str>,
        multi: bool,
        default: Vec<String>,
    },
    Text {
        default: Option<String>,
    },
}

#[derive(Debug)]
pub struct Filters<'cfg> {
    pub conditional_files: Vec<ConditionalFilesSpec<'cfg>>,
    pub include_hidden: Vec<GlobMatcher>,
    pub exclude: Vec<GlobMatcher>,
    pub templates: Option<Vec<GlobMatcher>>,
    pub non_templates: Option<Vec<GlobMatcher>>,
}

impl<'cfg> Filters<'cfg> {
    pub(crate) fn empty() -> Self {
        Filters {
            conditional_files: vec![],
            include_hidden: vec![],
            exclude: vec![],
            templates: None,
            non_templates: None,
        }
    }
}

#[derive(Debug)]
pub struct ConditionalFilesSpec<'cfg> {
    pub condition: &'cfg str,
    pub matcher: GlobMatcher,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Number;
    use tempfile::tempdir;

    use super::*;

    // todo: add conditional file specs to tests
    // todo: add include hidden globs to tests
    // todo: add excluded globs to tests

    const CONFIG_CONTENT: &str = r#"{
    "name": "Some Template",
    "version": "1.0.0"
}
"#;

    const TOOL_CONFIG: ToolConfig<'_> = ToolConfig {
        verbose: true,
        no_history: false,
        no_init: false,
        ignore_checks: false,
        template: None,
    };

    #[test]
    fn test_load_config_file() {
        let working_dir = tempdir().unwrap();

        assert_eq!(
            load_config_file(
                working_dir.path(),
                &ToolConfig {
                    template: None,
                    no_history: false,
                    no_init: false,
                    ignore_checks: false,
                    verbose: true,
                },
            )
            .unwrap(),
            None
        );

        fs::write(working_dir.path().join(".architect.json"), CONFIG_CONTENT).unwrap();

        assert_eq!(
            load_config_file(
                working_dir.path(),
                &ToolConfig {
                    template: None,
                    no_history: false,
                    no_init: false,
                    ignore_checks: false,
                    verbose: true,
                },
            )
            .unwrap(),
            Some(CONFIG_CONTENT.to_string())
        );
    }

    #[test]
    fn test_read_config() {
        let config_json = serde_json::to_string_pretty(&ConfigJson {
            name: Some("Some Template"),
            version: Some("0.1.0"),
            questions: Some(vec![
                RawQuestion {
                    name: "author",
                    pretty: Some("Who is the author of this project?"),
                    question_type: RawQuestionType::Text,
                    items: None,
                    multi: None,
                    default: None,
                },
                RawQuestion {
                    name: "debug",
                    question_type: RawQuestionType::Option,
                    pretty: None,
                    items: None,
                    multi: None,
                    default: Some(Value::Bool(true)),
                },
                RawQuestion {
                    name: "main.package",
                    question_type: RawQuestionType::Identifier,
                    pretty: None,
                    items: None,
                    multi: None,
                    default: None,
                },
                RawQuestion {
                    name: "main.features",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec!["feature_1", "feature_2", "feature_3"]),
                    multi: Some(true),
                    pretty: None,
                    default: Some(Value::Array(vec![
                        Value::String("feature_2".into()),
                        Value::String("feature_3".into()),
                    ])),
                },
            ]),
            filters: None,
        })
        .unwrap();

        let config = read_config(&config_json, &TOOL_CONFIG).unwrap();

        assert_eq!(
            config,
            Config {
                name: Some("Some Template"),
                version: Some("0.1.0"),
                questions: vec![
                    Question {
                        path: QuestionPath {
                            names: vec!["author"]
                        },
                        pretty: Some("Who is the author of this project?"),
                        spec: QuestionSpec::Text { default: None },
                    },
                    Question {
                        path: QuestionPath {
                            names: vec!["debug"]
                        },
                        spec: QuestionSpec::Option {
                            default: Some(true)
                        },
                        pretty: None,
                    },
                    Question {
                        path: QuestionPath {
                            names: vec!["main", "package"]
                        },
                        spec: QuestionSpec::Identifier { default: None },
                        pretty: None,
                    },
                    Question {
                        path: QuestionPath {
                            names: vec!["main", "features"]
                        },
                        spec: QuestionSpec::Selection {
                            items: vec!["feature_1", "feature_2", "feature_3"],
                            multi: true,
                            default: vec!["feature_2".into(), "feature_3".into()],
                        },
                        pretty: None,
                    },
                ],
                filters: Filters::empty(),
            }
        );

        assert_eq!(
            read_config(
                r#"{ "name": "Some Template", "version": null }"#,
                &TOOL_CONFIG,
            )
            .unwrap(),
            Config {
                name: Some("Some Template"),
                version: None,
                questions: vec![],
                filters: Filters::empty(),
            }
        )
    }

    #[test]
    fn test_read_config_failures() {
        let malformed_names_json = serde_json::to_string_pretty(&ConfigJson {
            name: Some("Some Template"),
            version: Some("0.1.0"),
            questions: Some(vec![
                RawQuestion {
                    name: "&author",
                    pretty: Some("Who is the author of this project?"),
                    question_type: RawQuestionType::Text,
                    items: None,
                    multi: None,
                    default: None,
                },
                RawQuestion {
                    name: "1.debug",
                    question_type: RawQuestionType::Option,
                    pretty: None,
                    items: None,
                    multi: None,
                    default: None,
                },
                RawQuestion {
                    name: "main..package",
                    question_type: RawQuestionType::Identifier,
                    pretty: None,
                    items: None,
                    multi: None,
                    default: None,
                },
                RawQuestion {
                    name: "",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec!["feature_1", "feature_2", "feature_3"]),
                    multi: Some(true),
                    pretty: None,
                    default: None,
                },
                RawQuestion {
                    name: "__template__.something",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec!["feature_1", "feature_2", "feature_3"]),
                    multi: Some(true),
                    pretty: None,
                    default: None,
                },
            ]),
            filters: None,
        })
        .unwrap();

        assert_eq!(
            read_config(&malformed_names_json, &TOOL_CONFIG).unwrap(),
            Config {
                name: Some("Some Template"),
                version: Some("0.1.0"),
                questions: vec![],
                filters: Filters::empty(),
            }
        );

        let malformed_context_tree = serde_json::to_string_pretty(&ConfigJson {
            name: Some("Some Template"),
            version: Some("0.1.0"),
            questions: Some(vec![
                RawQuestion {
                    name: "author",
                    pretty: Some("Who is the author of this project?"),
                    question_type: RawQuestionType::Text,
                    items: None,
                    multi: None,
                    default: Some(Value::String("You".into())),
                },
                RawQuestion {
                    name: "author.email",
                    question_type: RawQuestionType::Identifier,
                    pretty: None,
                    items: None,
                    multi: None,
                    default: None,
                },
                RawQuestion {
                    name: "author.email.domain",
                    question_type: RawQuestionType::Identifier,
                    pretty: None,
                    items: None,
                    multi: None,
                    default: None,
                },
            ]),
            filters: None,
        })
        .unwrap();

        assert_eq!(
            read_config(&malformed_context_tree, &TOOL_CONFIG).unwrap(),
            Config {
                name: Some("Some Template"),
                version: Some("0.1.0"),
                questions: vec![Question {
                    path: QuestionPath {
                        names: vec!["author"]
                    },
                    pretty: Some("Who is the author of this project?"),
                    spec: QuestionSpec::Text {
                        default: Some("You".into())
                    },
                },],
                filters: Filters::empty(),
            }
        );

        let malformed_selection_items = serde_json::to_string_pretty(&ConfigJson {
            name: None,
            version: None,
            questions: Some(vec![
                RawQuestion {
                    name: "features1",
                    question_type: RawQuestionType::Selection,
                    items: None,
                    pretty: None,
                    multi: Some(true),
                    default: None,
                },
                RawQuestion {
                    name: "features2",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec!["#feature1", "feature2", "abc.def"]),
                    pretty: None,
                    multi: None,
                    default: Some(Value::Array(vec!["feature2".into()])),
                },
                RawQuestion {
                    name: "features3",
                    question_type: RawQuestionType::Selection,
                    items: Some(vec![]),
                    pretty: None,
                    multi: None,
                    default: None,
                },
            ]),
            filters: None,
        })
        .unwrap();

        assert_eq!(
            read_config(&malformed_selection_items, &TOOL_CONFIG).unwrap(),
            Config {
                name: None,
                version: None,
                questions: vec![Question {
                    path: QuestionPath {
                        names: vec!["features2"]
                    },
                    spec: QuestionSpec::Selection {
                        items: vec!["feature2"],
                        multi: false,
                        default: vec!["feature2".into()],
                    },
                    pretty: None,
                }],
                filters: Filters::empty(),
            }
        )
    }

    #[test]
    fn test_read_default_value() {
        let no_default = RawQuestion {
            name: "an_option",
            question_type: RawQuestionType::Option,
            default: None,
            pretty: None,
            items: None,
            multi: None,
        };

        let no_default_result = read_default_value(&no_default, false);
        assert!(no_default_result.is_ok());
        assert_eq!(None, no_default_result.unwrap());

        let valid_option = RawQuestion {
            name: "an_option",
            question_type: RawQuestionType::Option,
            default: Some(Value::Bool(true)),
            pretty: None,
            items: None,
            multi: None,
        };

        let valid_option_default = read_default_value(&valid_option, false);

        assert!(valid_option_default.is_ok());
        assert_eq!(valid_option_default.unwrap(), Some(Value::Bool(true)));

        let invalid_option = RawQuestion {
            name: "an_option",
            question_type: RawQuestionType::Option,
            default: Some(Value::Number(Number::from(0))),
            pretty: None,
            items: None,
            multi: None,
        };

        let invalid_option_default = read_default_value(&invalid_option, false);
        assert!(invalid_option_default.is_err());

        let valid_selection = RawQuestion {
            name: "a_selection",
            question_type: RawQuestionType::Selection,
            items: Some(vec!["item1", "item2"]),
            default: Some(Value::String("item1".into())),
            pretty: None,
            multi: None,
        };

        let valid_selection_default = read_default_value(&valid_selection, false);

        assert!(valid_selection_default.is_ok());
        assert_eq!(
            valid_selection_default.unwrap(),
            Some(Value::String("item1".into()))
        );

        let invalid_selection = RawQuestion {
            name: "a_selection",
            question_type: RawQuestionType::Selection,
            items: Some(vec!["item1", "item2"]),
            default: Some(Value::String("item-1".into())),
            pretty: None,
            multi: None,
        };

        let invalid_selection_default = read_default_value(&invalid_selection, true);
        assert!(invalid_selection_default.is_err());

        let another_invalid_selection = RawQuestion {
            name: "a_selection",
            question_type: RawQuestionType::Selection,
            items: Some(vec!["item1", "item2"]),
            default: Some(Value::Bool(true)),
            pretty: None,
            multi: None,
        };

        let another_invalid_selection_default =
            read_default_value(&another_invalid_selection, true);

        assert!(another_invalid_selection_default.is_err());

        let valid_selection_list = RawQuestion {
            name: "a_selection",
            question_type: RawQuestionType::Selection,
            items: Some(vec!["item1", "item2"]),
            default: Some(Value::Array(vec!["item1".into()])),
            pretty: None,
            multi: None,
        };

        let valid_selection_list_default = read_default_value(&valid_selection_list, true);

        assert!(valid_selection_list_default.is_ok());
        assert_eq!(
            Some(Value::Array(vec!["item1".into()])),
            valid_selection_list_default.unwrap()
        );

        let invalid_selection_list = RawQuestion {
            name: "a_selection",
            question_type: RawQuestionType::Selection,
            items: Some(vec!["item1", "item2"]),
            default: Some(Value::Array(vec!["item-1".into()])),
            pretty: None,
            multi: None,
        };

        let invalid_selection_list_default = read_default_value(&invalid_selection_list, true);
        assert!(invalid_selection_list_default.is_err());

        let another_invalid_sel_list = RawQuestion {
            name: "a_selection",
            question_type: RawQuestionType::Selection,
            items: Some(vec!["item1", "item2"]),
            default: Some(Value::Bool(true)),
            pretty: None,
            multi: None,
        };

        let another_invalid_sel_list_default = read_default_value(&another_invalid_sel_list, true);
        assert!(another_invalid_sel_list_default.is_err());

        let other_question = RawQuestion {
            name: "a_text",
            question_type: RawQuestionType::Text,
            default: Some(Value::String("the content".into())),
            items: None,
            pretty: None,
            multi: None,
        };

        let other_question_default = read_default_value(&other_question, false);

        assert!(other_question_default.is_ok());
        assert_eq!(
            Some(Value::String("the content".into())),
            other_question_default.unwrap()
        );

        let inv_id_question = RawQuestion {
            name: "a_text",
            question_type: RawQuestionType::Identifier,
            default: Some(Value::String("the content".into())),
            items: None,
            pretty: None,
            multi: None,
        };

        let inv_id_question_default = read_default_value(&inv_id_question, true);
        assert!(inv_id_question_default.is_err());
    }

    impl<'cfg> PartialEq for Config<'cfg> {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
                && self.version == other.version
                && self.questions == other.questions
                && self.filters == other.filters
        }
    }

    impl<'cfg> PartialEq for Filters<'cfg> {
        fn eq(&self, other: &Self) -> bool {
            self.conditional_files == other.conditional_files
                && matchers_eq(&self.include_hidden, &other.include_hidden)
                && matchers_eq(&self.exclude, &other.exclude)
                && opt_matchers_eq(&self.templates, &other.templates)
                && opt_matchers_eq(&self.non_templates, &other.non_templates)
        }
    }

    fn matchers_eq(matchers_a: &[GlobMatcher], matchers_b: &[GlobMatcher]) -> bool {
        let mut matchers_a_iter = matchers_a.iter();
        let mut matchers_b_iter = matchers_b.iter();

        loop {
            let a = match matchers_a_iter.next() {
                Some(a) => a,
                None => return matchers_b_iter.next().is_none(),
            };

            let b = match matchers_b_iter.next() {
                Some(b) => b,
                None => return false,
            };

            if !glob_matcher_eq(a, b) {
                return false;
            }
        }
    }

    fn opt_matchers_eq(
        matchers_a: &Option<Vec<GlobMatcher>>,
        matchers_b: &Option<Vec<GlobMatcher>>,
    ) -> bool {
        if matchers_a.is_none() {
            return matchers_b.is_none();
        } else if matchers_b.is_none() {
            return false;
        }

        matchers_eq(matchers_a.as_ref().unwrap(), matchers_b.as_ref().unwrap())
    }

    #[inline]
    fn glob_matcher_eq(matcher_a: &GlobMatcher, matcher_b: &GlobMatcher) -> bool {
        matcher_a.glob() == matcher_b.glob()
    }

    impl<'cfg> PartialEq for ConditionalFilesSpec<'cfg> {
        fn eq(&self, other: &Self) -> bool {
            self.condition == other.condition && self.matcher.glob() == other.matcher.glob()
        }
    }
}
