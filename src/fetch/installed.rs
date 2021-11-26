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
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Error};
use crossterm::style::Stylize;

use crate::fetch::FetchOptions;
use crate::spec::TemplateSpec;
use crate::utils::errors::ArchResult;

pub fn fetch(spec: &TemplateSpec, target: &Path, options: &FetchOptions) -> ArchResult<()> {
    if options.tool_config.verbose {
        println!("{}", "  > Using local Git installation".stylize().dim());
    }

    let mut command = Command::new("git");
    command.arg("clone");
    command.arg(match spec {
        TemplateSpec::Local(path) => path.as_os_str(),
        &TemplateSpec::Remote(spec) => OsStr::new(spec),
    });
    command.arg(target.as_os_str());

    if let Some(branch) = options.branch {
        command.args(&["--branch", branch]);
    }

    let mut child = command.spawn()?;
    let exit_status = match child.try_wait() {
        Ok(Some(status)) => Ok(status),
        Ok(None) => child.wait(),
        Err(err) => Err(err),
    }
    .context("failed to wait for completion of Git process")?;

    if exit_status.success() {
        Ok(())
    } else {
        Err(Error::msg("Git process failed"))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::tests::REMOTE_TEMPLATE_URL;
    use crate::ToolConfig;

    use super::*;

    const TOOL_CONFIG: ToolConfig<'_> = ToolConfig {
        template: None,
        verbose: true,
        no_history: false,
        no_init: false,
        ignore_checks: false,
    };

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
