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

use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::Command;
use std::str;

use dircpy::copy_dir;
use git2::Repository;

use crate::spec::TemplateSpec;
use crate::utils::ToolConfig;

pub struct FetchOptions<'f, 't> {
    pub branch: Option<&'f str>,
    pub dirty: bool,
    pub tool_config: &'t ToolConfig<'t>,
}

pub fn fetch(template_spec: &TemplateSpec, target: &Path, options: FetchOptions) -> io::Result<()> {
    let (copy_dirty, local_path) = match template_spec {
        TemplateSpec::Local(local_path) => (
            options.dirty || !is_git_repo(local_path, options.tool_config.verbose),
            Some(local_path),
        ),
        _ => (false, None),
    };

    println!("Fetching template from {}", template_spec);

    if copy_dirty {
        let local_path = local_path.unwrap();

        if options.tool_config.verbose {
            println!("Copying dirty template from \"{}\"", local_path.display());
        }

        copy_dir(local_path, target)?
    } else {
        if options.tool_config.verbose {
            println!("Cloning template using Git from \"{}\"", template_spec);
        }

        let mut command = Command::new("git");
        command.arg("clone");
        command.arg(match template_spec {
            TemplateSpec::Local(path) => path.as_os_str(),
            &TemplateSpec::Remote(spec) => OsStr::new(spec),
        });
        command.arg(target.as_os_str());

        if let Some(branch) = options.branch {
            command.args(&["--branch", branch]);
        }

        let mut child = command.spawn()?;
        while child.try_wait()?.is_none() {}
    }

    Ok(())
}

fn is_git_repo(path: &Path, verbose: bool) -> bool {
    if verbose {
        println!(
            "Checking if directory \"{}\" contains a Git repository",
            path.display()
        );
    }

    let result = Repository::open(path).is_ok();

    if verbose {
        println!(
            "Directory {} a Git repository",
            if result {
                "contains"
            } else {
                "doesn't contain"
            }
        );
    }

    result
}

pub fn copy_git_directory(
    parent_dir: &Path,
    target_dir: &Path,
    tool_config: &ToolConfig,
) -> io::Result<()> {
    if is_git_repo(parent_dir, false) {
        if tool_config.verbose {
            println!("Copying .git directory from working directory to target directory")
        }

        copy_dir(parent_dir.join(".git"), target_dir.join(".git"))?;
        remove_remotes(target_dir);
    } else if tool_config.verbose {
        println!("Template not a Git repository. Not copying .git directory to target")
    }

    Ok(())
}

fn remove_remotes(dir: &Path) {
    match Repository::open(dir) {
        Ok(repo) => {
            repo.remotes()
                .map(|remotes| {
                    remotes
                        .iter()
                        .flatten()
                        .for_each(|remote| match repo.remote_delete(remote) {
                            Ok(_) => (),
                            Err(err) => {
                                eprintln!("Failed to remove remote {} ({})", remote, err);
                            }
                        })
                })
                .unwrap_or_else(|err| {
                    eprintln!("Failed to retrieve remotes ({})", err);
                });
        }
        Err(err) => {
            eprintln!(
                "Failed to open Git repository at {} ({})",
                dir.display(),
                err
            )
        }
    };
}

pub fn init_git_repository(target_dir: &Path, tool_config: &ToolConfig) -> io::Result<()> {
    if tool_config.verbose {
        println!("Initializing Git repository in target directory");
    }

    match Repository::init(target_dir) {
        Ok(_) => (),
        Err(err) => {
            eprintln!(
                "Failed to initialize Git repository in {} ({})",
                target_dir.display(),
                err
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::utils::tests::RESOURCES_DIR;

    use super::*;

    const FETCH_URL: &str = "https://github.com/v47-io/architect-test-template.git";
    const TOOL_CONFIG: ToolConfig = ToolConfig {
        template: None,
        verbose: true,
        no_history: false,
        no_init: false,
        ignore_checks: false,
    };

    #[test]
    fn test_fetch_local() -> io::Result<()> {
        let target_temp_dir = tempdir()?;
        let target_path = target_temp_dir.path();

        let spec = TemplateSpec::Local(RESOURCES_DIR.join("auto-template.input"));
        let options = FetchOptions {
            dirty: false,
            branch: None,
            tool_config: &TOOL_CONFIG,
        };

        fetch(&spec, target_path, options)?;

        let architect_file_path = target_path.join(".architect.json");
        assert!(architect_file_path.exists());

        assert!(!is_git_repo(target_path, true));

        init_git_repository(target_path, &TOOL_CONFIG)?;

        assert!(is_git_repo(target_path, true));

        Ok(())
    }

    #[test]
    fn test_fetch_remote() -> io::Result<()> {
        let target_temp_dir = tempdir()?;
        let target_path = target_temp_dir.path();

        let spec = TemplateSpec::Remote(FETCH_URL);
        let options = FetchOptions {
            dirty: false,
            branch: None,
            tool_config: &TOOL_CONFIG,
        };

        fetch(&spec, target_path, options)?;

        let architect_file_path = target_path.join(".architect.json");
        assert!(architect_file_path.exists());

        assert!(is_git_repo(target_path, true));

        let second_temp_dir = tempdir()?;
        let second_dir = second_temp_dir.path();

        copy_git_directory(target_path, second_dir, &TOOL_CONFIG)?;

        assert!(is_git_repo(second_dir, true));

        let second_repo = Repository::open(second_dir).unwrap();
        assert!(second_repo.remotes().unwrap().is_empty());

        Ok(())
    }
}
