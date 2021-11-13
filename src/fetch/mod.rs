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

use std::path::Path;

use anyhow::Context;
use dircpy::copy_dir;
use git2::Repository;

use crate::fetch::embedded::is_git_repo;
use crate::spec::TemplateSpec;
use crate::utils::errors::ArchResult;
use crate::utils::ToolConfig;

mod embedded;
mod installed;

pub struct FetchOptions<'f, 't> {
    pub branch: Option<&'f str>,
    pub dirty: bool,
    pub local_git: bool,
    pub tool_config: &'t ToolConfig<'t>,
}

impl<'spec> TemplateSpec<'spec> {
    pub fn fetch(&self, into: &Path, options: FetchOptions) -> ArchResult<()> {
        let (local_repo, is_local) = match self {
            TemplateSpec::Local(local_path) => (is_git_repo(local_path, options.tool_config), true),
            _ => (false, false),
        };

        println!("Fetching template from {}", self);

        if is_local && (!local_repo || options.dirty) {
            if let TemplateSpec::Local(local_path) = self {
                copy_dir(local_path, into)
                    .context(format!("Failed to copy directory {}", local_path.display()))
            } else {
                panic!()
            }
        } else if options.local_git {
            installed::fetch(self, into, &options)
                .context(format!("Failed to fetch using local Git: {}", self))
        } else {
            embedded::fetch(self, into, &options)
                .context(format!("Failed to fetch using embedded Git: {}", self))
        }
    }
}

pub fn copy_git_directory(
    parent_dir: &Path,
    target_dir: &Path,
    tool_config: &ToolConfig,
) -> ArchResult<()> {
    if embedded::is_git_repo(parent_dir, tool_config) {
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

pub fn init_git_repository(dir: &Path, tool_config: &ToolConfig) -> ArchResult<()> {
    if tool_config.verbose {
        println!("Initializing Git repository in target directory");
    }

    match Repository::init(dir) {
        Ok(_) => (),
        Err(err) => {
            eprintln!(
                "Failed to initialize Git repository in {} ({})",
                dir.display(),
                err
            );
        }
    }

    Ok(())
}
