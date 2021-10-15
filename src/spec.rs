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

use std::fmt::{Display, Error, Formatter};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum TemplateSpec<'spec> {
    Local(PathBuf),
    Remote(&'spec str),
}

impl Display for TemplateSpec<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            TemplateSpec::Local(path) => write!(f, "{}", path.display()),
            &TemplateSpec::Remote(spec) => write!(f, "{}", spec),
        }
    }
}

pub fn is_valid_template_spec(spec: &str) -> bool {
    if let Some(schema_delim_i) = spec.find("://") {
        if let Some(slash_index) = spec.rfind('/') {
            slash_index > schema_delim_i && slash_index < spec.len() - 1
        } else {
            false
        }
    } else if let Some(at_index) = spec.find('@') {
        if let Some(colon_index) = spec.rfind(':') {
            colon_index > at_index && colon_index < spec.len() - 1
        } else {
            false
        }
    } else {
        template_spec_as_path(spec).is_some()
    }
}

pub fn parse_template_spec(template_spec_raw: &str) -> TemplateSpec {
    match template_spec_as_path(template_spec_raw) {
        Some(dir) => TemplateSpec::Local(dir),
        None => TemplateSpec::Remote(template_spec_raw),
    }
}

fn template_spec_as_path(template_spec: &str) -> Option<PathBuf> {
    match Path::new(template_spec).canonicalize() {
        Ok(dir) => Some(dir),
        Err(err) => {
            eprintln!("Not pointing to a valid path: {} ({})", template_spec, err);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_template_spec_as_path_win() {
        assert_eq!(
            template_spec_as_path("C:\\Windows\\System32"),
            Some(PathBuf::from("\\\\?\\C:\\Windows\\System32"))
        );

        assert_eq!(
            template_spec_as_path("C:\\Windows\\..\\Data\\SomeRandomPath"),
            None
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_template_spec_as_path_unix() {
        assert_eq!(
            template_spec_as_path("/usr/bin"),
            Some(PathBuf::from("/usr/bin"))
        );

        assert_eq!(template_spec_as_path("/home/random/../invalid/dir"), None);
    }

    #[test]
    #[cfg(windows)]
    fn test_parse_template_spec_win() {
        let path_spec =
            TemplateSpec::Local(template_spec_as_path("C:\\Windows\\System32").unwrap());

        assert_eq!(parse_template_spec("C:\\Windows\\System32"), path_spec);
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_template_spec_unix() {
        let path_spec = TemplateSpec::Local(template_spec_as_path("/usr/bin").unwrap());

        assert_eq!(parse_template_spec("/usr/bin"), path_spec);
    }

    #[test]
    fn test_parse_template_spec() {
        let remote_spec = "git@github.com:v47-io/architect-rs.git";

        assert_eq!(
            parse_template_spec(remote_spec),
            TemplateSpec::Remote(remote_spec)
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_is_valid_template_spec_win() {
        assert!(is_valid_template_spec("C:\\Windows\\System32"));
        assert!(!is_valid_template_spec("Windows/System32"));
    }

    #[test]
    #[cfg(unix)]
    fn test_is_valid_template_spec_unix() {
        assert!(is_valid_template_spec("/usr/bin"));
        assert!(!is_valid_template_spec("C:\\Windows\\System32"));
    }

    #[test]
    fn test_is_valid_template_spec() {
        assert!(is_valid_template_spec(
            "git@github.com:v47-io/architect-rs.git"
        ),);
        assert!(!is_valid_template_spec(
            "git@github.com/v47-io/architect-rs.git"
        ));
        assert!(is_valid_template_spec(
            "https://github.com/v47-io/architect-rs.git"
        ));
        assert!(!is_valid_template_spec("https://github.com/v47-io/"));
    }
}
