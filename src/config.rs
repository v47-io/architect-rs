use std::io;
use std::io::{Error, ErrorKind};

use serde::Deserialize;

pub fn read_config(input: &str) -> io::Result<Config> {
    let json: ConfigJson = serde_json::from_str(input)?;

    if json.name.trim().is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "The template name cannot be empty",
        ));
    }

    let questions = json
        .questions
        .iter()
        .map(|raw_question| {
            if !raw_question.name.trim().is_ascii() {
                eprintln!("{} is not a valid question name", raw_question.name);
                return None;
            }

            Some(Question {
                name: raw_question.name.trim(),
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
                                .map(|item| item.trim())
                                .filter(|it| !it.is_empty())
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
                            multi_select: raw_question.multi_select.unwrap_or(false),
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
    multi_select: Option<bool>,
}

#[derive(Deserialize)]
enum RawQuestionType {
    Identifier,
    Option,
    Selection,
    Text,
}

pub struct Config<'cfg> {
    pub name: &'cfg str,
    pub version: &'cfg str,
    pub questions: Vec<Question<'cfg>>,
}

pub struct Question<'cfg> {
    pub name: &'cfg str,
    pub pretty: Option<&'cfg str>,
    pub spec: QuestionSpec<'cfg>,
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
