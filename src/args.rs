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

use clap::{crate_authors, crate_version, App, Arg, ArgMatches};

use constants::{args, flags, options};

use crate::utils::constants;

pub fn get_matches<'app, I, T>(args: I) -> ArgMatches<'app>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    App::new("Architect")
        .version(crate_version!())
        .author(crate_authors!(",\n"))
        .about("Scaffolds your projects using platform agnostic handlebars templates")
        .arg(
            Arg::with_name(args::REPOSITORY)
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
            Arg::with_name(options::BRANCH)
                .long(options::BRANCH)
                .short("b")
                .takes_value(true)
                .help("The remote branch to fetch instead of the default branch"),
        )
        .arg(
            Arg::with_name(flags::DRY_RUN)
                .long(flags::DRY_RUN)
                .help("Produces the same terminal output as normal operation without performing it")
                .long_help(
                    r#"Produces the same terminal output as normal operation without performing it.

This allows you to inspect the log output to determine whether Architect would
perform its operations as intended.

This takes all your input into account, it just stops shy of actually rendering
and copying files to the target directory."#,
                ),
        )
        .arg(
            Arg::with_name(flags::DIRTY)
                .long(flags::DIRTY)
                .help("Uses the template repository in it's current (dirty) state")
                .long_help(
                    r#"Uses the template repository in it's current (dirty) state.

This only has an effect if a local path is specified as the repository. In that
case Architect won't perform a clean clone but will just copy the directory,
regardless of the local state.

This is most useful to test a template locally, with remote repositories this
option doesn't have any effect"#,
                ),
        )
        .arg(
            Arg::with_name(flags::LOCAL_GIT)
                .long(flags::LOCAL_GIT)
                .help("Use the local Git installation instead of the embedded Git functions")
                .long_help(
                    r#"Use the local Git installation instead of the embedded Git functions.

Normally Architect uses its own embedded Git functionality to fetch templates. If you are
using unusual or unsupported authentication methods this might fail, so you can use this
as an escape hatch to have Architect use your local Git installation and environment to
fetch remote repositories"#,
                ),
        )
        .arg(
            Arg::with_name(options::TEMPLATE)
                .long(options::TEMPLATE)
                .short("t")
                .takes_value(true)
                .help("Specify a template (sub-directory) within the template repository")
                .long_help(
                    r#"Specify a template (sub-directory) within the template repository.

This will then treat that sub-directory within the repository as the template root directory
and look for an .architect.json file there, instead of in the repository root"#,
                ),
        )
        .arg(
            Arg::with_name(args::TARGET)
                .help("The target directory for the final output")
                .long_help(
                    r#"The target directory for the final output.

This defaults to the Git repository name as a child of the current working directory"#,
                )
                .index(2),
        )
        .arg(
            Arg::with_name(flags::NO_HISTORY)
                .long(flags::NO_HISTORY)
                .help("Don't copy over Git history from template to target")
                .long_help(
                    r#"Don't copy over Git history from template to target.

Instead the target directory will be initialized as a new Git repository"#,
                ),
        )
        .arg(
            Arg::with_name(flags::NO_INIT)
                .long(flags::NO_INIT)
                .requires("no-history")
                .help("Don't initialize the target directory as a Git repository")
                .long_help(
                    r#"Don't initialize the target directory as a Git repository.

This requires the --no-history flag to be specified as well"#,
                ),
        )
        .arg(
            Arg::with_name(flags::IGNORE_CHECKS)
                .long(flags::IGNORE_CHECKS)
                .help("Ignores some failed checks that would prevent generation otherwise")
                .long_help(
                    r#"Ignores some failed checks that would prevent generation otherwise.

These errors will be ignored:
  - Unexpected type of default value (for any question type)
  - Default value not matching the format (for custom questions)
  - Unknown default item (for selection questions)
  - Condition evaluation errors (for conditional files)"#,
                ),
        )
        .arg(
            Arg::with_name(flags::VERBOSE)
                .long(flags::VERBOSE)
                .help("Enables verbose output"),
        )
        .get_matches_from(args)
}

pub(crate) trait TrimmedValueOf<'app> {
    fn value_of_trimmed(&'app self, name: &str) -> Option<&'app str>;
}

impl<'app> TrimmedValueOf<'app> for ArgMatches<'app> {
    fn value_of_trimmed(&'app self, name: &str) -> Option<&'app str> {
        self.value_of(name)
            .map(|it| it.trim())
            .filter(|&it| !it.is_empty())
    }
}
