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

use std::borrow::Cow;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::Context;
use crossterm::style::{Attribute, Stylize};
use dialoguer::{Input, Password};
use git2::build::{CheckoutBuilder, CloneLocal, RepoBuilder};
use git2::{self, BranchType, ErrorClass, Repository, RepositoryOpenFlags};
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;

use crate::fetch::FetchOptions;
use crate::spec::TemplateSpec;
use crate::term::write_check_ln;
use crate::utils::errors::ArchResult;
use crate::utils::ToolConfig;

pub fn is_git_repo(path: &Path, tool_config: &ToolConfig) -> bool {
    let status_callback = if tool_config.verbose {
        Some(write_check_ln("Local directory is Git repository:", &[Attribute::Dim]).unwrap())
    } else {
        None
    };

    let result =
        { Repository::open_ext(path, RepositoryOpenFlags::NO_SEARCH, &[] as &[&OsStr]).is_ok() };

    if tool_config.verbose {
        status_callback.unwrap()(if result { "YES" } else { "NO" }, result).unwrap();
    }

    result
}

pub fn fetch(spec: &TemplateSpec, target: &Path, options: &FetchOptions) -> ArchResult<()> {
    if options.tool_config.verbose {
        println!("{}", "Using embedded Git".dim());
    }

    let (url, git_config, local) = match spec {
        TemplateSpec::Local(local_path) => {
            (local_path.to_string_lossy(), git2::Config::new()?, true)
        }
        &TemplateSpec::Remote(remote) => (Cow::from(remote), git2::Config::open_default()?, false),
    };

    let mut repo = None;
    with_fetch_options(&git_config, &url, &mut |fetch_options| {
        let checkout = git2::build::CheckoutBuilder::new();

        let mut repo_builder = RepoBuilder::new();
        repo_builder.with_checkout(checkout);
        repo_builder.fetch_options(fetch_options);

        if local {
            repo_builder.clone_local(CloneLocal::Local);
        }

        repo = Some(repo_builder.clone(&url, target)?);

        Ok(())
    })?;

    let repo = repo.unwrap();
    reset(&repo, options)
}

fn reset(repo: &git2::Repository, options: &FetchOptions) -> ArchResult<()> {
    if let Ok(mut git_config) = repo.config() {
        git_config.set_bool("core.autocrlf", false)?;
    }

    if let Some(branch) = options.branch {
        let remote_branch = repo
            .find_branch(branch, BranchType::Remote)
            .or_else(|_| repo.find_branch(&format!("origin/{}", branch), BranchType::Remote))
            .context(format!("Branch not found: {}", branch))?;

        let remote_branch_name = remote_branch.name()?.unwrap();
        let remote_branch_tree = repo.revparse_single(remote_branch_name)?;

        let mut checkout = CheckoutBuilder::new();
        checkout.force();

        repo.reset(
            &remote_branch_tree,
            git2::ResetType::Hard,
            Some(&mut checkout),
        )?;
    }

    Ok(())
}

//region yoink'd code
// Yoink'd from https://github.com/rust-lang/cargo/blob/master/src/cargo/sources/git/utils.rs
fn with_authentication<T, F>(url: &str, cfg: &git2::Config, mut f: F) -> ArchResult<T>
where
    F: FnMut(&mut git2::Credentials<'_>) -> ArchResult<T>,
{
    let mut cred_helper = git2::CredentialHelper::new(url);
    cred_helper.config(cfg);

    let mut ssh_agent_attempts = Vec::new();
    let mut any_attempts = false;
    let mut tried_sshkey = false;
    let mut url_attempt = None;

    let orig_url = url;
    let res = f(&mut |url, username, allowed| {
        any_attempts = true;

        if url != orig_url {
            url_attempt = Some(url.to_string());
        }

        // libgit2's "USERNAME" authentication actually means that it's just
        // asking us for a username to keep going. This is currently only really
        // used for SSH authentication and isn't really an authentication type.
        // The logic currently looks like:
        //
        //      let user = ...;
        //      if (user.is_null())
        //          user = callback(USERNAME, null, ...);
        //
        //      callback(SSH_KEY, user, ...)
        //
        // So if we're being called here then we know that (a) we're using ssh
        // authentication and (b) no username was specified in the URL that
        // we're trying to clone. We need to guess an appropriate username here,
        // but that may involve a few attempts. Unfortunately we can't switch
        // usernames during one authentication session with libgit2, so to
        // handle this we bail out of this authentication session after setting
        // the flag `ssh_username_requested`, and then we handle this below.
        if allowed.contains(git2::CredentialType::USERNAME) {
            debug_assert!(username.is_none());
            return git2::Cred::username(&read_username()?);
        }

        // An "SSH_KEY" authentication indicates that we need some sort of SSH
        // authentication. This can currently either come from the ssh-agent
        // process or from a raw in-memory SSH key. Cargo only supports using
        // ssh-agent currently.
        //
        // If we get called with this then the only way that should be possible
        // is if a username is specified in the URL itself (e.g., `username` is
        // Some), hence the unwrap() here. We try custom usernames down below.
        if allowed.contains(git2::CredentialType::SSH_KEY) && !tried_sshkey {
            // If ssh-agent authentication fails, libgit2 will keep
            // calling this callback asking for other authentication
            // methods to try. Make sure we only try ssh-agent once,
            // to avoid looping forever.
            tried_sshkey = true;
            let username = username.unwrap();

            ssh_agent_attempts.push(username.to_string());
            return git2::Cred::ssh_key_from_agent(username);
        }

        // Sometimes libgit2 will ask for a username/password in plaintext. This
        // is where Cargo would have an interactive prompt if we supported it,
        // but we currently don't! Right now the only way we support fetching a
        // plaintext password is through the `credential.helper` support, so
        // fetch that here.
        //
        // If ssh-agent authentication fails, libgit2 will keep calling this
        // callback asking for other authentication methods to try. Check
        // cred_helper_bad to make sure we only try the git credentail helper
        // once, to avoid looping forever.
        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            let r = git2::Cred::credential_helper(cfg, url, username);

            return if r.is_err() {
                let mut final_username = username.map(|it| it.to_string());
                let final_password: Option<String>;

                if final_username.is_none() {
                    final_username = Some(read_username()?);
                }

                match Password::new().with_prompt("Password").interact() {
                    Ok(password) => final_password = Some(password),
                    Err(err) => {
                        return Err(git2::Error::from_str(&format!(
                            "failed to enter password ({})",
                            err
                        )));
                    }
                }

                git2::Cred::userpass_plaintext(&final_username.unwrap(), &final_password.unwrap())
            } else {
                r
            };
        }

        // I'm... not sure what the DEFAULT kind of authentication is, but seems
        // easy to support?
        if allowed.contains(git2::CredentialType::DEFAULT) {
            return git2::Cred::default();
        }

        // Whelp, we tried our best
        Err(git2::Error::from_str("no authentication available"))
    });

    let mut err = match res {
        Ok(e) => return Ok(e),
        Err(e) => e,
    };

    // In the case of an authentication failure (where we tried something) then
    // we try to give a more helpful error message about precisely what we
    // tried.
    if any_attempts {
        let mut msg = "failed to authenticate when downloading \nrepository".to_string();

        if let Some(attempt) = &url_attempt {
            if url != attempt {
                msg.push_str(": ");
                msg.push_str(attempt);
            }
        }

        msg.push('\n');
        if !ssh_agent_attempts.is_empty() {
            let names = ssh_agent_attempts
                .iter()
                .map(|s| format!("`{}`", s))
                .collect::<Vec<_>>()
                .join(", ");
            msg.push_str(&format!(
                "\n* attempted ssh-agent authentication, but \
                 no usernames succeeded: {}",
                names
            ));
        }

        msg.push_str("\n\n");
        msg.push_str(
            "if the git CLI succeeds then running Architect with `--local-git` may help here",
        );

        err = err.context(msg);

        // Otherwise if we didn't even get to the authentication phase them we may
        // have failed to set up a connection, in these cases hint on the
        // `net.git-fetch-with-cli` configuration option.
    } else if let Some(e) = err.downcast_ref::<git2::Error>() {
        match e.class() {
            ErrorClass::Net
            | ErrorClass::Ssl
            | ErrorClass::Submodule
            | ErrorClass::FetchHead
            | ErrorClass::Ssh
            | ErrorClass::Callback
            | ErrorClass::Http => {
                let mut msg = "network failure seems to have happened\n".to_string();
                msg.push_str(
                    "if a proxy or similar is necessary, run Architect with `--local-git`",
                );
                err = err.context(msg);
            }
            _ => {}
        }
    }

    Err(err)
}
//endregion

fn read_username() -> core::result::Result<String, git2::Error> {
    match Input::<String>::new()
        .with_prompt("Username")
        .interact_text()
    {
        Ok(res) => Ok(res),
        Err(err) => Err(git2::Error::from_str(&format!(
            "failed to read username ({})",
            err
        ))),
    }
}

static PROGRESS_BAR_TEMPLATE: &str = "{prefix} {wide_bar} {pos}/{len}";

lazy_static! {
    static ref DELTA_PROGRESS_STYLE: ProgressStyle = ProgressStyle::default_bar()
        .template(PROGRESS_BAR_TEMPLATE)
        .progress_chars(".. ");
    static ref OBJECTS_PROGRESS_STYLE: ProgressStyle = ProgressStyle::default_bar()
        .template(PROGRESS_BAR_TEMPLATE)
        .progress_chars("█░.");
}

fn with_fetch_options(
    git_config: &git2::Config,
    url: &str,
    cb: &mut dyn FnMut(git2::FetchOptions<'_>) -> ArchResult<()>,
) -> ArchResult<()> {
    let progress_bar = ProgressBar::new(0);

    with_authentication(url, git_config, |f| {
        let mut rcb = git2::RemoteCallbacks::new();
        rcb.credentials(f);

        let mut mode = 0;

        rcb.transfer_progress(|progress| {
            if progress.indexed_deltas() > 0 {
                if mode != 0 {
                    progress_bar.reset();
                    progress_bar.set_prefix("Resolving Deltas");
                    progress_bar.set_style(DELTA_PROGRESS_STYLE.clone());
                    mode = 0;
                }

                let indexed_deltas = u64::try_from(progress.indexed_deltas()).unwrap();
                let total_deltas = u64::try_from(progress.total_deltas()).unwrap();

                progress_bar.set_position(indexed_deltas);
                progress_bar.set_length(total_deltas);
            } else {
                if mode != 1 {
                    progress_bar.reset();
                    progress_bar.set_prefix("Receiving Objects");
                    progress_bar.set_style(OBJECTS_PROGRESS_STYLE.clone());
                    mode = 1;
                }

                let indexed_objects = u64::try_from(progress.indexed_objects()).unwrap();
                let total_objects = u64::try_from(progress.total_objects()).unwrap();

                progress_bar.set_position(indexed_objects);
                progress_bar.set_length(total_objects);
            }

            true
        });

        let mut opts = git2::FetchOptions::new();
        opts.remote_callbacks(rcb);
        cb(opts)
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use path_absolutize::Absolutize;

    use crate::utils::tests::{REMOTE_TEMPLATE_URL, RESOURCES_DIR};

    use super::*;

    const TOOL_CONFIG: ToolConfig<'_> = ToolConfig {
        template: None,
        verbose: true,
        no_history: false,
        no_init: false,
        dry_run: false,
        ignore_checks: false,
    };

    #[test]
    fn test_is_git_repo() {
        let work_dir = PathBuf::from(".").absolutize().unwrap().to_path_buf();

        assert!(is_git_repo(&work_dir, &TOOL_CONFIG));
        assert!(!is_git_repo(&RESOURCES_DIR, &TOOL_CONFIG));
    }

    //noinspection DuplicatedCode
    #[test]
    fn test_fetch() {
        let tempdir = tempfile::tempdir().unwrap();
        let spec = TemplateSpec::Remote(REMOTE_TEMPLATE_URL);

        assert!(fetch(
            &spec,
            tempdir.path(),
            &FetchOptions {
                branch: None,
                tool_config: &TOOL_CONFIG,
                dirty: false,
                local_git: true
            }
        )
        .is_ok());

        assert!(!tempdir
            .path()
            .join("io/v47/test/added-file-in-branch.txt")
            .exists());

        drop(tempdir);

        let tempdir = tempfile::tempdir().unwrap();

        assert!(fetch(
            &spec,
            tempdir.path(),
            &FetchOptions {
                branch: Some("test-branch"),
                tool_config: &TOOL_CONFIG,
                dirty: false,
                local_git: true
            }
        )
        .is_ok());

        assert!(tempdir
            .path()
            .join("io/v47/test/added-file-in-branch.txt")
            .exists());
    }
}
