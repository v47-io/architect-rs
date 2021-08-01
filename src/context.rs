use std::io;
use std::mem::transmute;

use dialoguer::{Confirm, Input, MultiSelect, Select};
use handlebars::Context;
use serde_json::{to_value, Map, Value};

use crate::config::{Config, Question, QuestionSpec};
use crate::utils::is_identifier;

struct UnsafeContext {
    _data: Value,
}

impl UnsafeContext {
    fn new(data: Map<String, Value>) -> Self {
        UnsafeContext {
            _data: Value::Object(data),
        }
    }
}

pub fn build_context(config: &Config) -> io::Result<Context> {
    let mut context_json = Map::new();
    context_json.insert("__template__".to_string(), to_value(config)?);

    for question in &config.questions {
        let answer = ask(question)?;
        insert_into_context(&mut context_json, question.path.names(), answer);
    }

    Ok(unsafe { transmute(UnsafeContext::new(context_json)) })
}

fn ask(question: &Question) -> io::Result<Value> {
    match &question.spec {
        QuestionSpec::Identifier => ask_for_text(question, true),
        QuestionSpec::Text => ask_for_text(question, false),
        QuestionSpec::Option => ask_for_option(question),
        QuestionSpec::Selection {
            items,
            multi_select,
        } => ask_for_selection(question, items, *multi_select),
    }
}

fn ask_for_text(question: &Question, must_be_identifier: bool) -> io::Result<Value> {
    let prompt = question
        .pretty
        .map(|it| it.to_string())
        .unwrap_or(question.path.names().join("."));

    loop {
        let result_text = Input::<String>::new()
            .with_prompt(&prompt)
            .interact_text()?;

        let result_trimmed = result_text.trim();

        if must_be_identifier && !is_identifier(result_trimmed) {
            eprintln!("Not a valid identifier: {}", result_trimmed);
            continue;
        } else if result_trimmed.is_empty() {
            eprintln!("You must enter a value");
            continue;
        }

        return Ok(Value::String(result_trimmed.to_string()));
    }
}

fn ask_for_option(question: &Question) -> io::Result<Value> {
    Confirm::new()
        .with_prompt(question.pretty.unwrap_or(&question.path.names().join(".")))
        .interact()
        .map(|value| Value::Bool(value))
}

fn ask_for_selection(question: &Question, items: &[&str], multi_select: bool) -> io::Result<Value> {
    let prompt = question
        .pretty
        .map(|it| it.to_string())
        .unwrap_or(question.path.names().join("."));

    let selection = if multi_select {
        MultiSelect::new().with_prompt(prompt).interact()?
    } else {
        Select::new()
            .with_prompt(prompt)
            .interact()
            .map(|it| vec![it])?
    };

    let mut result_map = Map::new();
    for i in selection {
        result_map.insert(items[i].to_string(), Value::Bool(true));
    }

    Ok(Value::Object(result_map))
}

fn insert_into_context(context: &mut Map<String, Value>, path: &[&str], value: Value) {
    let name = *path.first().unwrap();

    if path.len() == 1 {
        context.insert(name.to_string(), value);
    } else {
        let new_context = if let Some(item) = context.get_mut(name) {
            match item {
                Value::Object(map) => map,
                _ => panic!(),
            }
        } else {
            {
                context.insert(name.to_string(), Value::Object(Map::new()));
            }

            match context.get_mut(name).unwrap() {
                Value::Object(map) => map,
                _ => panic!(),
            }
        };

        insert_into_context(new_context, &path[1..], value);
    }
}
