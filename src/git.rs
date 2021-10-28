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
    pub tool_config: &'t ToolConfig,
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
    // todo: test_fetch
    // todo: test_copy_git_directory
    // todo: test_init_git_repository
}
