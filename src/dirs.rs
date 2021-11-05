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

use std::fs::{create_dir_all, metadata};
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use path_absolutize::Absolutize;

use crate::spec::TemplateSpec;
use crate::utils::ToolConfig;

pub fn create_target_dir(
    base_dir: &Path,
    template_spec: &TemplateSpec,
    target_override: Option<&str>,
) -> io::Result<PathBuf> {
    if let Some(target_dir_raw) = target_override {
        let tmp_path = Path::new(target_dir_raw.trim());
        if tmp_path.is_absolute() {
            create_dir_all(tmp_path)?;
            Ok(tmp_path.to_path_buf())
        } else {
            let result = base_dir.join(target_dir_raw).absolutize()?.to_path_buf();
            create_dir_all(&result)?;

            Ok(result)
        }
    } else {
        match template_spec {
            TemplateSpec::Local(template_path) => match template_path.file_name() {
                Some(file_name) => {
                    let result = base_dir.join(file_name).absolutize()?.to_path_buf();
                    create_dir_all(&result)?;

                    Ok(result)
                }
                None => create_err(template_spec),
            },
            &TemplateSpec::Remote(remote_spec) => {
                if let Some(slash_index) = remote_spec.rfind('/') {
                    let dir_name = if let Some(dot_git_index) = remote_spec.rfind(".git") {
                        remote_spec[slash_index + 1..dot_git_index].to_string()
                    } else {
                        remote_spec[slash_index + 1..].to_string()
                    };

                    let result = base_dir.join(dir_name).absolutize()?.to_path_buf();
                    create_dir_all(&result)?;

                    Ok(result)
                } else {
                    create_err(template_spec)
                }
            }
        }
    }
}

pub fn is_valid_target_dir(path: &Path) -> io::Result<bool> {
    if metadata(path).is_ok() {
        if path.is_dir() {
            Ok(path.read_dir()?.next().is_none())
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

fn create_err(template_spec: &TemplateSpec) -> Result<PathBuf, Error> {
    Err(Error::new(
        ErrorKind::InvalidInput,
        format!(
            "Failed to extract target directory name from template specification \"{}\"\n",
            template_spec
        ),
    ))
}

pub fn find_template_dir(
    root_dir: &Path,
    tool_config: &ToolConfig<'_>,
) -> io::Result<(PathBuf, bool)> {
    if let Some(template) = tool_config.template {
        let template_dir = root_dir.join(template).absolutize()?.to_path_buf();
        if template_dir.join(".architect.json").is_file() {
            println!("Using template {} from repository", template);

            Ok((template_dir, true))
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid template name: {} ({})",
                    template, "Doesn't contain .architect.json file"
                ),
            ))
        }
    } else {
        Ok((root_dir.to_path_buf(), false))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;

    use path_absolutize::Absolutize;
    use tempfile::tempdir;

    use crate::utils::tests::RESOURCES_DIR;

    use super::*;

    #[test]
    fn test_create_target_dir() -> io::Result<()> {
        let base_dir = tempdir()?;

        let abs_target_override =
            PathBuf::from(format!("{}/some-directory", base_dir.path().display()))
                .absolutize()
                .unwrap()
                .to_string_lossy()
                .to_string();

        assert_eq!(
            PathBuf::from_str(&abs_target_override).unwrap(),
            create_target_dir(
                base_dir.path(),
                &TemplateSpec::Remote(""),
                Some(&abs_target_override)
            )?
        );

        assert!(PathBuf::from(abs_target_override).exists());

        let rel_target_override = "another-directory";

        assert_eq!(
            base_dir
                .path()
                .join(rel_target_override)
                .absolutize()
                .unwrap()
                .to_path_buf(),
            create_target_dir(
                base_dir.path(),
                &TemplateSpec::Remote(""),
                Some(rel_target_override)
            )?
        );

        let valid_local_spec = TemplateSpec::Local(PathBuf::from("/some/dir/project-name"));

        let local_check_path = base_dir
            .path()
            .join("project-name")
            .absolutize()
            .unwrap()
            .to_path_buf();

        assert_eq!(
            local_check_path,
            create_target_dir(base_dir.path(), &valid_local_spec, None)?
        );

        assert!(local_check_path.exists());

        let valid_remote_spec =
            TemplateSpec::Remote("git@github.com:v47-io/another-project-name.git");

        let remote_check_path = base_dir
            .path()
            .join("another-project-name")
            .absolutize()
            .unwrap()
            .to_path_buf();

        assert_eq!(
            remote_check_path,
            create_target_dir(base_dir.path(), &valid_remote_spec, None)?
        );

        assert!(remote_check_path.exists());

        let invalid_local_spec = TemplateSpec::Local(PathBuf::from("/"));

        assert!(create_target_dir(base_dir.path(), &invalid_local_spec, None).is_err());

        let invalid_remote_spec = TemplateSpec::Remote("git@github.com:project-name.git");

        assert!(create_target_dir(base_dir.path(), &invalid_remote_spec, None).is_err());

        Ok(())
    }

    #[test]
    fn test_is_valid_target_dir() -> io::Result<()> {
        let dir = tempdir()?;

        assert!(is_valid_target_dir(dir.path())?);

        let file_path = dir.path().join("random_file");
        fs::write(file_path, "test")?;

        assert!(!is_valid_target_dir(dir.path())?);
        assert!(!is_valid_target_dir(&dir.path().join("../.tmp000000"))?);

        Ok(())
    }

    #[test]
    fn test_find_template_dir() -> io::Result<()> {
        let project_dir = RESOURCES_DIR.join("..").absolutize()?.to_path_buf();
        let template_dir = RESOURCES_DIR
            .join("auto-template.input")
            .absolutize()?
            .to_path_buf();

        let tool_config = ToolConfig {
            template: None,
            verbose: true,
            no_history: false,
            no_init: false,
            ignore_checks: false,
        };

        assert!(find_template_dir(&template_dir, &tool_config).is_ok());
        assert!(find_template_dir(&project_dir, &tool_config).is_ok());

        let tool_config = ToolConfig {
            template: Some("auto-template.input"),
            verbose: true,
            no_history: false,
            no_init: false,
            ignore_checks: false,
        };

        assert!(find_template_dir(&RESOURCES_DIR, &tool_config).is_ok());
        assert!(find_template_dir(&project_dir, &tool_config).is_err());

        let tool_config = ToolConfig {
            template: Some("simple-template.input"),
            verbose: true,
            no_history: false,
            no_init: false,
            ignore_checks: false,
        };

        assert!(find_template_dir(&RESOURCES_DIR, &tool_config).is_err());

        Ok(())
    }
}
