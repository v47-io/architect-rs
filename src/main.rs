use std::io::{Error, ErrorKind};
use std::process::exit;
use std::{env, io};

use clap::{crate_authors, crate_version, App, Arg};
use handlebars::Context;
use tempfile::tempdir;

use crate::config::{load_config_file, read_config};
use crate::context::build_context;
use crate::dirs::{create_target_dir, is_valid_target_dir};
use crate::git::{open_git_repo_or_init, FetchOptions};
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
    exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Error ({:?}): {}", err.kind(), err.to_string());
            1
        }
    })
}

fn run() -> io::Result<()> {
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
            Arg::with_name("local-branch")
                .long("local-branch")
                .short("B")
                .takes_value(true)
                .default_value("main")
                .help("The name of the local branch where to commit the final project")
                .long_help(
                    r#"The name of the local branch where to commit the final project.

This can be different from the remote branch that serves as the template source.
In that case the new branch is created on top of the remote branch."#,
                ),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .help("Enables verbose output"),
        )
        .get_matches();

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

    let repo = open_git_repo_or_init(&target_dir, verbose)
        .map_err(|err| Error::new(ErrorKind::Other, err.message()))?;

    if verbose {
        println!(
            "Opened template Git repository at \"{}\"",
            target_dir.display()
        );
    }

    let config_json = load_config_file(working_dir.path(), verbose)?;
    let config = if let Some(config_json) = &config_json {
        Some(read_config(config_json)?)
    } else {
        None
    };

    let context = config.map_or(Ok(Context::null()), |it| build_context(&it))?;

    todo!()
}
