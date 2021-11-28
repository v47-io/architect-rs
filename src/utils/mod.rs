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

use clap::ArgMatches;
use globset::{Error, GlobBuilder, GlobMatcher};
use lazy_static::lazy_static;
use regex::Regex;

use crate::{constants::*, TrimmedValueOf};

pub mod constants;
pub mod context;
pub mod errors;
pub mod reader;

pub struct ToolConfig<'tc> {
    pub template: Option<&'tc str>,
    pub no_history: bool,
    pub no_init: bool,
    pub ignore_checks: bool,
    pub dry_run: bool,
    pub verbose: bool,
}

impl<'tc> ToolConfig<'tc> {
    pub fn from_matches<'arg>(matches: &'arg ArgMatches<'arg>) -> ToolConfig<'tc>
    where
        'arg: 'tc,
    {
        ToolConfig {
            template: matches.value_of_trimmed(options::TEMPLATE),
            no_history: matches.is_present(flags::NO_HISTORY),
            no_init: matches.is_present(flags::NO_INIT),
            ignore_checks: matches.is_present(flags::IGNORE_CHECKS),
            dry_run: matches.is_present(flags::DRY_RUN),
            verbose: matches.is_present(flags::VERBOSE),
        }
    }
}

lazy_static! {
    pub static ref ID_REGEX: Regex = Regex::new("^[a-zA-Z_$][a-zA-Z0-9_$]*$").unwrap();
    pub static ref NEW_LINE_REGEX: Regex = Regex::new(r#"(\r?\n)(\s+|\r?\n)*"#).unwrap();
}

pub fn is_identifier(value: &str) -> bool {
    ID_REGEX.is_match(value)
}

pub fn glob(input: &str) -> Result<GlobMatcher, Error> {
    GlobBuilder::new(input)
        .case_insensitive(true)
        .literal_separator(true)
        .build()
        .map(|it| it.compile_matcher())
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::PathBuf;

    use lazy_static::lazy_static;

    use super::*;

    pub const REMOTE_TEMPLATE_URL: &str = "https://github.com/v47-io/architect-test-template.git";

    lazy_static! {
        pub static ref RESOURCES_DIR: PathBuf =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources");
    }

    #[test]
    fn test_is_identifier() {
        assert!(is_identifier("this_is_an_identifier_1$"));
        assert!(!is_identifier("1not_an_identifier"));
    }
}
