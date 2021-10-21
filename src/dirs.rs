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

use crate::spec::TemplateSpec;

pub fn create_target_dir(
    base_dir: &Path,
    target_match: Option<&str>,
    template_spec: &TemplateSpec,
) -> io::Result<PathBuf> {
    if let Some(target_dir_raw) = target_match {
        let tmp_path = Path::new(target_dir_raw.trim());
        if tmp_path.is_absolute() {
            Ok(tmp_path.to_path_buf().canonicalize()?)
        } else {
            let tmp_result = base_dir.join(target_dir_raw);
            create_dir_all(&tmp_result)?;

            Ok(tmp_result.canonicalize()?)
        }
    } else {
        match template_spec {
            TemplateSpec::Local(template_path) => {
                let result = base_dir.join(template_path.file_name().unwrap());
                create_dir_all(&result)?;

                Ok(result.canonicalize()?)
            }
            TemplateSpec::Remote(remote_spec) => {
                if let Some(slash_index) = remote_spec.rfind('/') {
                    let dir_name = if let Some(dot_git_index) = remote_spec.rfind(".git") {
                        remote_spec[slash_index + 1..dot_git_index].to_string()
                    } else {
                        remote_spec[slash_index + 1..].to_string()
                    };

                    let result = base_dir.join(dir_name);
                    create_dir_all(&result)?;

                    Ok(result.canonicalize()?)
                } else {
                    Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "Failed to extract target directory name from template specification \"{}\"\n",
                            template_spec
                        ),
                    ))
                }
            }
        }
    }
}

pub fn is_valid_target_dir(path: &Path) -> io::Result<bool> {
    if metadata(path).is_ok() {
        if path.is_dir() {
            let mut is_empty = true;

            for _ in path.read_dir()? {
                is_empty = false;
                break;
            }

            Ok(is_empty)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    // todo: test_create_target_dir
    // todo: test_is_valid_target_dir
}
