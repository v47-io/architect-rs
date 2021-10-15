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

use std::ffi::OsString;
use std::io::{Error, ErrorKind};
use std::process::exit;
use std::{env, io};

use handlebars::Context;
use serde_json::Value;
use tempfile::tempdir;

use crate::config::{load_config_file, read_config};
use crate::context::build_context;
use crate::dirs::{create_target_dir, is_valid_target_dir};
use crate::git::{copy_git_directory, FetchOptions};
use crate::spec::{is_valid_template_spec, parse_template_spec};
use crate::utils::ToolConfig;

mod args;
mod config;
mod context;
mod dirs;
mod git;
mod helpers;
mod render;
mod spec;
mod utils;

fn main() {
    exit(match run(&mut env::args_os()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error ({:?}): {}", err.kind(), err.to_string());
            1
        }
    })
}

fn run<I, T>(args: I) -> io::Result<i32>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = crate::args::get_matches(args);

    let tool_config = ToolConfig {
        ignore_checks: matches.is_present("ignore-checks"),
        verbose: matches.is_present("verbose"),
    };

    if tool_config.ignore_checks {
        println!("Ignoring some checks")
    }

    if tool_config.verbose {
        println!("Verbose output enabled")
    }

    let template_spec_raw = matches.value_of("REPOSITORY").unwrap().trim();

    if !is_valid_template_spec(template_spec_raw) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Invalid template specification \"{}\"\n", template_spec_raw),
        ));
    }

    let template_spec = parse_template_spec(template_spec_raw);

    if tool_config.verbose {
        println!("Using template specification \"{}\"", template_spec);
    }

    let target_dir = create_target_dir(
        &env::current_dir()?,
        matches.value_of("TARGET"),
        &template_spec,
    )?;

    if !is_valid_target_dir(&target_dir)? {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Invalid target directory \"{}\"", target_dir.display()),
        ));
    }

    if tool_config.verbose {
        println!("Scaffolding into directory   \"{}\"", target_dir.display());
    }

    let working_dir = tempdir()?;
    if tool_config.verbose {
        println!(
            "Using temporary directory    \"{}\"",
            working_dir.path().canonicalize()?.display()
        )
    }

    git::fetch(
        &template_spec,
        working_dir.path(),
        FetchOptions {
            branch: matches.value_of("branch"),
            dirty: matches.is_present("dirty"),
            tool_config: &tool_config,
        },
    )?;

    let config_json = load_config_file(working_dir.path(), &tool_config)?;
    let config = if let Some(config_json) = &config_json {
        Some(read_config(config_json)?)
    } else {
        None
    };

    let context = match &config {
        Some(c) => build_context(c),
        None => Ok(Context::null()),
    }?;

    if tool_config.verbose && *context.data() != Value::Null {
        println!(
            "Using context\n{}",
            serde_json::to_string_pretty(context.data())?
        );
    }

    let render_result = render::render(
        working_dir.path(),
        target_dir.as_path(),
        &(config.map_or_else(|| vec![], |c| c.conditional_files)),
        &context,
        &tool_config,
    )?;

    copy_git_directory(working_dir.path(), &target_dir, &tool_config)?;

    if tool_config.verbose {
        println!("Rendered {} files:", render_result.rendered_files.len());
        render_result
            .rendered_files
            .iter()
            .for_each(|render_spec| println!("  - {}", render_spec.target.display()));
    }

    if render_result.conflicts.len() > 0 {
        eprintln!("There were conflicts:");
        render_result.conflicts.iter().for_each(|conflict| {
            eprintln!("  - {}:", conflict.intended_target.display());
            conflict
                .sources
                .iter()
                .for_each(|source| println!("      - {}", source.display()))
        });

        working_dir.into_path();
    }

    println!(
        "Finished scaffolding into directory {}",
        target_dir.display()
    );

    Ok(if render_result.conflicts.is_empty() {
        0
    } else {
        2
    })
}
