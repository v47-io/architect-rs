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

impl Into<Context> for UnsafeContext {
    fn into(self) -> Context {
        unsafe { transmute(self) }
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
        QuestionSpec::Identifier => ask_for_text(question, true),
        QuestionSpec::Text => ask_for_text(question, false),
        QuestionSpec::Option => ask_for_option(question),
        QuestionSpec::Selection {
            items,
            multi: multi_select,
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

        if must_be_identifier && !result_trimmed.split(".").all(|it| is_identifier(it)) {
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
