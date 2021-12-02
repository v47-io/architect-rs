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

use std::io;
use std::mem::transmute;

use dialoguer::{Confirm, Input, MultiSelect, Select};
use handlebars::Context;
use regex::Regex;
use serde_json::{to_value, Map, Value};

use crate::config::{Config, Question, QuestionSpec};
use crate::term::theme::WithFormat;
use crate::utils::is_identifier;

pub(crate) struct UnsafeContext {
    _data: Value,
}

impl UnsafeContext {
    pub(crate) fn new(data: Map<String, Value>) -> Self {
        UnsafeContext {
            _data: Value::Object(data),
        }
    }

    pub(crate) fn empty() -> Self {
        UnsafeContext {
            _data: Value::Object(Map::new()),
        }
    }
}

impl From<UnsafeContext> for Context {
    fn from(uc: UnsafeContext) -> Self {
        unsafe { transmute(uc) }
    }
}

pub fn build_context(config: &Config) -> io::Result<Context> {
    let mut context_json = Map::new();
    context_json.insert("__template__".to_string(), to_value(config)?);

    for question in &config.questions {
        let answer = ask(question)?;
        insert_into_context(&mut context_json, question.path.names(), answer);
    }

    Ok(UnsafeContext::new(context_json).into())
}

fn ask(question: &Question) -> io::Result<Value> {
    match &question.spec {
        QuestionSpec::Identifier { default } => ask_for_text(question, true, default),
        QuestionSpec::Text { default } => ask_for_text(question, false, default),
        QuestionSpec::Option { default } => ask_for_option(question, default),
        QuestionSpec::Selection {
            items,
            multi: multi_select,
            default,
        } => ask_for_selection(question, items, *multi_select, default),
        QuestionSpec::Custom { format, default } => ask_for_custom(question, *format, default),
    }
}

fn ask_for_text(
    question: &Question,
    must_be_identifier: bool,
    default: &Option<String>,
) -> io::Result<Value> {
    let prompt = question.prompt();

    let mut text_input = Input::<String>::with_theme(&crate::term::theme::INSTANCE);
    text_input.with_prompt(prompt);

    if let Some(value) = default {
        text_input.default(value.clone());
    }

    if must_be_identifier {
        text_input.validate_with(|value: &String| -> Result<(), &str> {
            if value.split('.').all(is_identifier) {
                Ok(())
            } else {
                Err("Not a valid identifier")
            }
        });
    }

    Ok(Value::String(text_input.interact_text()?))
}

fn ask_for_option(question: &Question, default: &Option<bool>) -> io::Result<Value> {
    let mut confirm_prompt = Confirm::with_theme(&crate::term::theme::INSTANCE);
    confirm_prompt.with_prompt(question.prompt());

    if let Some(default) = default {
        confirm_prompt.default(*default);
    }

    confirm_prompt.interact().map(Value::Bool)
}

fn ask_for_selection(
    question: &Question,
    items: &[&str],
    multi_select: bool,
    default: &[String],
) -> io::Result<Value> {
    let prompt = question.prompt();

    let defaults = items
        .iter()
        .map(|&item| default.contains(&item.to_string()))
        .collect::<Vec<_>>();

    let selection = if multi_select {
        MultiSelect::with_theme(&crate::term::theme::INSTANCE)
            .with_prompt(prompt)
            .items(items)
            .defaults(&*defaults)
            .interact()?
    } else {
        let mut select = Select::with_theme(&crate::term::theme::INSTANCE);
        select.with_prompt(prompt);
        select.items(items);

        if !default.is_empty() {
            let first_item_in_defaults = &*default[0];
            select.default(
                items
                    .iter()
                    .position(|&item| item == first_item_in_defaults)
                    .unwrap(),
            );
        }

        select.interact().map(|it| vec![it])?
    };

    let mut result_map = Map::new();
    for i in selection {
        result_map.insert(items[i].into(), Value::Bool(true));
    }

    Ok(Value::Object(result_map))
}

fn ask_for_custom(
    question: &Question,
    format: &str,
    default: &Option<String>,
) -> io::Result<Value> {
    let prompt = question.prompt().with_format(format);

    let mut text_input = Input::<String>::with_theme(&crate::term::theme::INSTANCE);
    text_input.with_prompt(prompt);

    if let Some(value) = default {
        text_input.default(value.clone());
    }

    let regex = Regex::new(format.trim()).unwrap();

    text_input.validate_with(move |value: &String| -> Result<(), String> {
        if regex.is_match(value) {
            return Ok(());
        }

        Err(format!("Expected format: {}", format))
    });

    Ok(Value::String(text_input.interact()?))
}

fn insert_into_context(context: &mut Map<String, Value>, path: &[&str], value: Value) {
    let name = *path.first().unwrap();

    if path.len() == 1 {
        context.insert(name.to_string(), value);
    } else {
        let new_context = if let Some(item) = context.get_mut(name) {
            match item {
                Value::Object(map) => map,
                _ => panic!("not an object"),
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

impl Question<'_> {
    fn prompt(&self) -> String {
        self.pretty
            .map(|it| it.to_string())
            .unwrap_or_else(|| self.path.names().join("."))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Number;

    use super::*;

    #[test]
    fn test_into_context() {
        let context_map = create_test_value();
        let check_map = context_map.clone();

        let context: Context = UnsafeContext::new(context_map).into();

        assert_eq!(context.data(), &Value::Object(check_map));
    }

    #[test]
    fn test_insert_into_context() {
        let mut context = Map::new();

        insert_into_context(
            &mut context,
            &["test"],
            Value::String(String::from("value")),
        );

        insert_into_context(
            &mut context,
            &["a", "container", "value"],
            Value::Bool(true),
        );

        insert_into_context(
            &mut context,
            &["a", "container", "child"],
            Value::Number(Number::from_f64(256.0).unwrap()),
        );

        let check_map = create_test_value();

        assert_eq!(context, check_map)
    }

    #[test]
    #[should_panic(expected = "not an object")]
    fn test_insert_into_context_not_an_object() {
        let mut context = Map::new();

        insert_into_context(
            &mut context,
            &["test"],
            Value::String(String::from("value")),
        );

        insert_into_context(
            &mut context,
            &["test", "property"],
            Value::String(String::from("won't work")),
        );
    }

    fn create_test_value() -> Map<String, Value> {
        let mut check_map = Map::new();
        check_map.insert(String::from("test"), Value::String(String::from("value")));

        let mut container_map = Map::new();
        container_map.insert(String::from("value"), Value::Bool(true));
        container_map.insert(
            String::from("child"),
            Value::Number(Number::from_f64(256.0).unwrap()),
        );

        let mut a_map = Map::new();
        a_map.insert(String::from("container"), Value::Object(container_map));

        check_map.insert(String::from("a"), Value::Object(a_map));

        check_map
    }
}
