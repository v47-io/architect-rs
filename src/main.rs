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

use std::ffi::OsString;
use std::io::{Error, ErrorKind};
use std::process::exit;
use std::{env, io};

use clap::{crate_authors, crate_version, App, Arg};
use handlebars::Context;
use serde_json::Value;
use tempfile::tempdir;

use crate::config::{load_config_file, read_config};
use crate::context::build_context;
use crate::dirs::{create_target_dir, is_valid_target_dir};
use crate::git::{copy_git_directory, FetchOptions};
use crate::spec::{is_valid_template_spec, parse_template_spec};

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
    let matches = App::new("Architect")
        .version(crate_version!())
        .author(crate_authors!(",\n"))
        .about("Scaffolds your projects using platform agnostic handlebars templates")
        .arg(
            Arg::with_name("REPOSITORY")
                .help("The Git repository to use as the project template")
                .long_help(
                    r#"The git repository to use as the project template.

This can be specified in any way that you can refer to a git repository,
i.e. an HTTP(S) URL, ssh connection string, or a local path.

Example: git@github.com:some-user/his-template-repo.git"#,
                )
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("branch")
                .long("branch")
                .short("b")
                .takes_value(true)
                .help("The remote branch to fetch instead of the default branch"),
        )
        .arg(
            Arg::with_name("dirty")
                .long("dirty")
                .help("Uses the template repository in it's current (dirty) state")
                .long_help(
                    r#"Uses the template repository in it's current (dirty) state.

This only has an effect if a local path is specified as the repository. In that
case Architect won't perform a clean clone but will just copy the directory,
regardless of the local state.

This is most useful to test a template locally, for remote repositories this
option doesn't make sense."#,
                ),
        )
        .arg(
            Arg::with_name("TARGET")
                .help("The target directory for the final output")
                .long_help(
                    r#"The target directory for the final output.

This defaults to the Git repository name as a child of the current working directory."#,
                )
                .index(2),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .help("Enables verbose output"),
        )
        .get_matches_from(args);

    let verbose = matches.is_present("verbose");
    if verbose {
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

    if verbose {
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

    if verbose {
        println!("Scaffolding into directory   \"{}\"", target_dir.display());
    }

    let working_dir = tempdir()?;
    if verbose {
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
            verbose,
        },
    )?;

    let config_json = load_config_file(working_dir.path(), verbose)?;
    let config = if let Some(config_json) = &config_json {
        Some(read_config(config_json)?)
    } else {
        None
    };

    let context = config.map_or(Ok(Context::null()), |it| build_context(&it))?;

    if verbose && *context.data() != Value::Null {
        println!(
            "Using context\n{}",
            serde_json::to_string_pretty(context.data())?
        );
    }

    let render_result =
        render::render(working_dir.path(), target_dir.as_path(), &context, verbose)?;

    copy_git_directory(working_dir.path(), &target_dir, verbose)?;

    if verbose {
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
