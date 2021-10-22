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

use std::cmp::{max, min};
use std::collections::HashMap;
use std::env::var;
use std::fs::{copy, create_dir_all, read_to_string, File};
use std::io;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use handlebars::{Context, Handlebars, RenderError};
use handlebars_misc_helpers::register;
use indicatif::{MultiProgress, ProgressBar};
use lazy_static::lazy_static;
use path_absolutize::Absolutize;
use walkdir::{DirEntry, WalkDir};

use crate::config::{ConditionalFilesSpec, Config};
use crate::helpers::PACKAGE_HELPER;
use crate::utils::reader::BufReader;
use crate::utils::{ToolConfig, NEW_LINE_REGEX};

lazy_static! {
    static ref RENDER_PARALLELISM: usize =
        if let Ok(Ok(value)) = var("RENDER_PARALLELISM").map(|raw| usize::from_str(&raw)) {
            value
        } else {
            min(1, max(4, num_cpus::get() / 2))
        };

    static ref TEMPLATE_INSPECT_MAX_LINES: usize =
        if let Ok(Ok(value)) = var("TEMPLATE_INSPECT_MAX_LINES").map(|raw| usize::from_str(&raw)) {
            value
        } else {
            // this feels like a sensible default, like who puts a Handlebars expression at the end of a 1000 line template?
            25
        };
}

pub fn render(
    root_dir: &Path,
    target_dir: &Path,
    config: &Config,
    context: &Context,
    tool_config: &ToolConfig,
) -> io::Result<RenderResult> {
    let mut handlebars = Handlebars::new();
    register(&mut handlebars);
    handlebars.register_helper("package", Box::new(PACKAGE_HELPER));

    let all_progress = MultiProgress::new();

    let render_specs_progress = all_progress.add(ProgressBar::new_spinner());
    render_specs_progress.set_message("Building render specifications...");

    let render_specs = build_render_specs(
        root_dir,
        target_dir,
        config,
        &handlebars,
        context,
        tool_config,
    )?;

    all_progress.remove(&render_specs_progress);

    // Creating new Handlebars instance without helpers that shouldn't be used in templates
    handlebars = Handlebars::new();
    register(&mut handlebars);

    let (rspec_sender, rspec_receiver) = channel::<RenderSpec>();
    let rspec_receiver = Arc::new(Mutex::new(rspec_receiver));

    let rendered_files = Arc::new(Mutex::new(Vec::<RenderSpec>::new()));

    let conflicts: Vec<RenderConflict> = crossbeam::scope(|scope| {
        let handlebars = &handlebars;

        (0..*RENDER_PARALLELISM).for_each(|worker_num| {
            let rspec_receiver = Arc::clone(&rspec_receiver);
            let rendered_files = Arc::clone(&rendered_files);

            let progress = all_progress.add(ProgressBar::new_spinner());

            scope.spawn(move |_| loop {
                progress.set_message(format!("[{}] Waiting...", worker_num));

                match rspec_receiver.lock().unwrap().recv() {
                    Ok(render_spec) => {
                        if render_spec.is_template {
                            progress.set_message(format!(
                                "[{}] Rendering > {}",
                                worker_num,
                                render_spec.target.display()
                            ));

                            match render_template_to_file(
                                &render_spec.source,
                                &render_spec.target,
                                handlebars,
                                context,
                            ) {
                                Ok(_) => rendered_files.lock().unwrap().push(render_spec),
                                Err(err) => {
                                    eprintln!(
                                        "Failed to render template '{}' to '{}' ({})",
                                        render_spec.source.display(),
                                        render_spec.target.display(),
                                        err
                                    )
                                }
                            }
                        } else {
                            progress.set_message(format!(
                                "[{}] Copying   > {}",
                                worker_num,
                                render_spec.target.display()
                            ));

                            match copy(&render_spec.source, &render_spec.target) {
                                Ok(_) => (),
                                Err(err) => {
                                    eprintln!(
                                        "Failed to copy '{}' to '{}' ({})",
                                        render_spec.source.display(),
                                        render_spec.target.display(),
                                        err
                                    )
                                }
                            }
                        };
                    }
                    Err(_) => {
                        progress.finish_with_message(format!("[{}] No more work", worker_num));
                        break;
                    }
                }
            });
        });

        let result = render_specs
            .into_iter()
            .filter_map(|(intended_target, render_specs)| {
                match create_dir_all(intended_target.parent().unwrap()) {
                    Ok(_) => (),
                    Err(err) => {
                        eprintln!(
                            "Failed to create parent director(ies) of '{}' ({})",
                            intended_target.display(),
                            err
                        );

                        return None;
                    }
                }

                if render_specs.len() > 1 {
                    Some(RenderConflict {
                        intended_target,
                        sources: render_specs
                            .into_iter()
                            .map(|it| {
                                rspec_sender.send(it.clone()).unwrap();
                                it.source
                            })
                            .collect(),
                    })
                } else {
                    render_specs
                        .into_iter()
                        .for_each(|it| rspec_sender.send(it).unwrap());

                    None
                }
            })
            .collect();

        drop(rspec_sender);

        result
    })
    .unwrap();

    all_progress.clear()?;

    let rendered_files = Arc::try_unwrap(rendered_files)
        .unwrap()
        .into_inner()
        .unwrap();

    Ok(RenderResult {
        rendered_files,
        conflicts,
    })
}

fn render_template_to_file(
    source: &Path,
    target: &Path,
    hbs: &Handlebars,
    ctx: &Context,
) -> io::Result<()> {
    let template = read_to_string(source)?;

    let rendered = match hbs.render_template_with_context(&template, ctx) {
        Ok(result) => result,
        Err(err) => return Err(Error::new(ErrorKind::InvalidData, format!("{}", err))),
    };

    let mut target_file = File::create(target)?;
    target_file.write_all(rendered.as_bytes())
}

fn build_render_specs(
    root_dir: &Path,
    target_dir: &Path,
    config: &Config,
    hbs: &Handlebars,
    ctx: &Context,
    tool_config: &ToolConfig,
) -> io::Result<HashMap<PathBuf, Vec<RenderSpec>>> {
    let mut render_specs: HashMap<PathBuf, Vec<RenderSpec>> = HashMap::new();
    let mut dir_context_stack = vec![DirContext {
        source_path: root_dir.to_path_buf(),
        target_path: Some(target_dir.to_path_buf()),
    }];

    let walk = WalkDir::new(root_dir);

    for entry_result in walk
        .into_iter()
        .filter_entry(|entry| filter_dir_entry(root_dir, config, hbs, ctx, tool_config, entry))
    {
        let entry = entry_result?;
        let metadata = entry.metadata()?;

        while dir_context_stack
            .last()
            .map_or(false, |it| !entry.path().starts_with(&it.source_path))
        {
            dir_context_stack.pop();
        }

        if let None = dir_context_stack.last().unwrap().target_path {
            if tool_config.verbose {
                println!("Skipping path:     {}", entry.path().display())
            }

            continue;
        }

        if tool_config.verbose {
            println!("Processing path:   {}", entry.path().display());
            println!("Directory context: {:?}", dir_context_stack.last())
        }

        if metadata.is_dir() {
            let entry_dir_name = entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let entry_target_dir_name = if it_contains_template(&entry_dir_name) {
                create_entry_target_dir_name(&entry_dir_name, hbs, ctx)
            } else {
                Some(entry_dir_name)
            };

            let target_path = entry_target_dir_name.map(|it| {
                create_proper_target_path(
                    target_dir,
                    dir_context_stack
                        .last()
                        .unwrap()
                        .target_path
                        .as_ref()
                        .unwrap(),
                    &it,
                )
                .unwrap_or_else(|| {
                    dir_context_stack
                        .last()
                        .unwrap()
                        .target_path
                        .as_ref()
                        .unwrap()
                        .join(entry.file_name())
                })
            });

            dir_context_stack.push(DirContext {
                source_path: entry.path().to_path_buf(),
                target_path,
            })
        } else {
            let current_dir_ctx = dir_context_stack.last().unwrap();

            let source_file_name = entry.file_name().to_string_lossy().to_string();

            let is_template = is_hbs_template(entry.path())?;

            let target_file_name =
                strip_handlebars_xt(if it_contains_template(&source_file_name) {
                    create_entry_target_file_name(&source_file_name, hbs, ctx)
                } else {
                    source_file_name
                });

            let singular_target_path = create_proper_target_path(
                target_dir,
                current_dir_ctx.target_path.as_ref().unwrap(),
                &target_file_name,
            )
            .unwrap_or_else(|| {
                current_dir_ctx
                    .target_path
                    .as_ref()
                    .unwrap()
                    .join(entry.file_name())
            });

            let render_specs_vec = get_render_specs_vec(&mut render_specs, &singular_target_path);

            render_specs_vec.push(RenderSpec {
                source: entry.into_path(),
                target: get_numbered_path(singular_target_path, render_specs_vec.len()),
                is_template,
            })
        };
    }

    Ok(render_specs)
}

fn filter_dir_entry(
    root_dir: &Path,
    config: &Config,
    hbs: &Handlebars,
    ctx: &Context,
    tool_config: &ToolConfig,
    entry: &DirEntry,
) -> bool {
    is_not_git_dir_in_root(root_dir, entry)
        && is_not_hidden_or_included(root_dir, config, hbs, ctx, tool_config, entry)
        && is_not_excluded(root_dir, config, tool_config, entry)
}

#[inline]
fn is_not_git_dir_in_root(root_dir: &Path, entry: &DirEntry) -> bool {
    entry.path() != root_dir.join(".git")
}

fn is_not_hidden_or_included(
    root_dir: &Path,
    config: &Config,
    hbs: &Handlebars,
    ctx: &Context,
    tool_config: &ToolConfig,
    entry: &DirEntry,
) -> bool {
    !entry.file_name().to_string_lossy().starts_with(".")
        || if let Some(cond_spec) = find_condition_spec(
            entry.path().strip_prefix(root_dir).unwrap(),
            &config.conditional_files,
        ) {
            let skip = match eval_condition(cond_spec, hbs, ctx) {
                Ok(is_match) => !is_match,
                Err(e) => {
                    eprintln!(
                        "Failed to evaluate condition \"{}\" for {} ({})",
                        cond_spec.condition,
                        entry.path().display(),
                        e
                    );

                    !tool_config.ignore_checks
                }
            };

            if skip && tool_config.verbose {
                println!("Skipping path:     {}", entry.path().display())
            }

            // inverting because filter needs false to exclude
            !skip
        } else {
            false
        }
}

fn is_not_excluded(
    root_dir: &Path,
    config: &Config,
    tool_config: &ToolConfig,
    entry: &DirEntry,
) -> bool {
    let globbing_path = entry.path().strip_prefix(root_dir).unwrap();

    if let Some(_) = config
        .exclude
        .iter()
        .find(|&glob| glob.is_match(globbing_path))
    {
        if tool_config.verbose {
            println!("Skipping path:     {}", entry.path().display())
        }

        false
    } else {
        true
    }
}

fn find_condition_spec<'a>(
    path: &Path,
    conditional_files_specs: &'a Vec<ConditionalFilesSpec<'a>>,
) -> Option<&'a ConditionalFilesSpec<'a>> {
    conditional_files_specs
        .iter()
        .find(|&cond_files_spec| cond_files_spec.matcher.is_match(path))
}

fn eval_condition(
    conditional_files_spec: &ConditionalFilesSpec,
    handlebars: &Handlebars,
    context: &Context,
) -> Result<bool, RenderError> {
    match handlebars.render_template_with_context(conditional_files_spec.condition, context) {
        Ok(rendered) => Ok(is_truthy(&rendered)),
        Err(e) => Err(e),
    }
}

fn is_truthy(value: &str) -> bool {
    value.trim() != ""
        && value != "0"
        && value != "{}"
        && value != "[]"
        && value != "false"
        && value != "null"
}

fn it_contains_template(value: &str) -> bool {
    if let Some(start_i) = value.find("{{") {
        if let Some(end_i) = value.rfind("}}") {
            end_i > start_i
        } else {
            false
        }
    } else {
        false
    }
}

fn get_render_specs_vec<'spec>(
    render_specs: &'spec mut HashMap<PathBuf, Vec<RenderSpec>>,
    target_path: &PathBuf,
) -> &'spec mut Vec<RenderSpec> {
    if render_specs.contains_key(target_path) {
        render_specs.get_mut(target_path).unwrap()
    } else {
        {
            render_specs.insert(target_path.clone(), vec![]);
        }

        render_specs.get_mut(target_path).unwrap()
    }
}

fn create_entry_target_dir_name(
    source_name: &str,
    handlebars: &Handlebars,
    context: &Context,
) -> Option<String> {
    let (result, was_rendered) = render_line_template(source_name, handlebars, context);

    if was_rendered {
        Some(result)
    } else {
        None
    }
}

fn create_entry_target_file_name(
    source_name: &str,
    handlebars: &Handlebars,
    context: &Context,
) -> String {
    let (result, _) = render_line_template(&source_name, handlebars, context);
    result
}

fn is_hbs_template(path: &Path) -> io::Result<bool> {
    Ok(BufReader::open(path)?
        .into_iter()
        .take(*TEMPLATE_INSPECT_MAX_LINES)
        .find(|line| match line {
            Ok(line) => {
                if let Some(start) = line.find("{{") {
                    if let Some(end) = line.rfind("}}") {
                        return end > start;
                    }
                }

                false
            }
            Err(err) => {
                eprintln!("Failed to read line into the buffer ({})", err);

                false
            }
        })
        .is_some())
}

fn strip_handlebars_xt(name: String) -> String {
    if let Some(i) = name.to_lowercase().rfind(".hbs") {
        if i == name.len() - 4 {
            return String::from(&name[..i]);
        }
    }

    name
}

fn render_line_template(template: &str, handlebars: &Handlebars, ctx: &Context) -> (String, bool) {
    match handlebars.render_template_with_context(template, ctx) {
        Ok(result) => (NEW_LINE_REGEX.replace_all(&result, " ").to_string(), true),
        Err(error) => {
            eprintln!(
                "Failed to render small template: '{}' ({})",
                template, error
            );
            (template.to_string(), false)
        }
    }
}

fn create_proper_target_path(
    target_dir: &Path,
    parent_dir: &Path,
    target_name: &str,
) -> Option<PathBuf> {
    let result = match parent_dir.join(target_name).absolutize() {
        Ok(path) => path.to_path_buf(),
        Err(_) => {
            eprintln!(
                "Failed to create absolute path of '{}'",
                parent_dir.join(target_dir).display()
            );

            return None;
        }
    };

    if result.starts_with(target_dir) {
        Some(result)
    } else {
        eprintln!(
            "Generated path '{}' would leave target directory '{}'",
            result.display(),
            target_dir.display()
        );

        None
    }
}

fn get_numbered_path(base_path: PathBuf, number: usize) -> PathBuf {
    if number == 0 {
        base_path
    } else {
        let file_name = base_path.file_name().unwrap().to_string_lossy().to_string();
        let numbered_name = if let Some(dot_i) = file_name.rfind(".") {
            format!(
                "{} ({}).{}",
                file_name[..dot_i].to_string(),
                number,
                file_name[dot_i + 1..].to_string()
            )
        } else {
            format!("{} ({})", file_name, number)
        };

        base_path.parent().map_or_else(
            || PathBuf::from(&numbered_name),
            |parent| parent.join(&numbered_name),
        )
    }
}

#[derive(Clone, Debug)]
pub struct RenderSpec {
    pub source: PathBuf,
    pub target: PathBuf,
    is_template: bool,
}

#[derive(Clone, Debug)]
struct DirContext {
    source_path: PathBuf,
    target_path: Option<PathBuf>,
}

pub struct RenderResult {
    pub rendered_files: Vec<RenderSpec>,
    pub conflicts: Vec<RenderConflict>,
}

pub struct RenderConflict {
    pub intended_target: PathBuf,
    pub sources: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use globset::Glob;
    use lazy_static::lazy_static;
    use serde_json::{Map, Number, Value};
    use tempfile::{tempdir, TempDir};

    use crate::context::UnsafeContext;
    use crate::utils::tests::RESOURCES_DIR;

    use super::*;

    lazy_static! {
        static ref HANDLEBARS: Handlebars<'static> = Handlebars::new();
        static ref TEMP_DIR: TempDir = tempdir().unwrap();
    }

    // todo: test_render
    // todo: test_filter_dir_entry
    // todo: test_is_hbs_template

    #[test]
    fn test_render_to_file() {
        let mut context_map = Map::new();
        context_map.insert("lang".into(), Value::String("en".into()));

        let mut en_content_map = Map::new();
        en_content_map.insert("title".into(), Value::String("A simple template".into()));
        en_content_map.insert(
            "heading".into(),
            Value::String("Welcome at the simple template!".into()),
        );

        let mut content_map = Map::new();
        content_map.insert("en".into(), Value::Object(en_content_map));

        context_map.insert("content".into(), Value::Object(content_map));

        let context = UnsafeContext::new(context_map).into();

        let source_path = RESOURCES_DIR.join("simple-template.input/simple-template.html.hbs");
        let target_path = TEMP_DIR.path().join("simple-template.html");

        render_template_to_file(&source_path, &target_path, &HANDLEBARS, &context).unwrap();

        let en_expected_content =
            read_to_string(RESOURCES_DIR.join("simple-template.expected/en/simple-template.html"))
                .unwrap();

        let en_actual_content = read_to_string(target_path).unwrap();

        assert_eq!(en_expected_content, en_actual_content);
    }

    #[test]
    fn test_eval_condition() {
        let mut context_map = Map::new();
        context_map.insert("simple".into(), Value::Number(Number::from(2)));
        context_map.insert("falsy".into(), Value::Bool(false));

        let context = UnsafeContext::new(context_map).into();

        assert_eq!(
            true,
            eval_condition(
                &ConditionalFilesSpec {
                    condition: "{{ simple }}",
                    matcher: Glob::new("").unwrap().compile_matcher()
                },
                &HANDLEBARS,
                &context
            )
            .unwrap_or(false)
        );

        assert_eq!(
            false,
            eval_condition(
                &ConditionalFilesSpec {
                    condition: "{{ falsy }}",
                    matcher: Glob::new("").unwrap().compile_matcher()
                },
                &HANDLEBARS,
                &context
            )
            .unwrap_or(false)
        );

        assert_eq!(
            false,
            eval_condition(
                &ConditionalFilesSpec {
                    condition: "{{ asdasd",
                    matcher: Glob::new("").unwrap().compile_matcher()
                },
                &HANDLEBARS,
                &context
            )
            .unwrap_or(false)
        );
    }

    #[test]
    fn test_render_line_template() {
        let mut context_map = Map::new();
        context_map.insert("newLine".into(), Value::String("multi\n  \nline".into()));
        context_map.insert("simple".into(), Value::Number(Number::from(2)));

        let context = UnsafeContext::new(context_map).into();

        assert_eq!(
            ("multi line".into(), true),
            render_line_template("{{ newLine }}", &HANDLEBARS, &context)
        );

        assert_eq!(
            ("{{ asdasd".into(), false),
            render_line_template("{{ asdasd", &HANDLEBARS, &context)
        );

        assert_eq!(
            ("2".into(), true),
            render_line_template("{{ simple }}", &HANDLEBARS, &context)
        );
    }

    #[test]
    fn test_create_proper_target_path() {
        let temp_dir = tempdir().unwrap();

        let target_dir = temp_dir.path().join("some-target-dir");

        assert_eq!(
            Some(target_dir.join("this-is-a-file-name")),
            create_proper_target_path(temp_dir.path(), &target_dir, "this-is-a-file-name")
        );

        assert_eq!(
            None,
            create_proper_target_path(temp_dir.path(), &target_dir, "../../abcdef")
        );

        assert_eq!(
            None,
            create_proper_target_path(temp_dir.path(), temp_dir.path(), &"../".repeat(15))
        );
    }

    #[test]
    fn test_get_numbered_path() {
        let path_with_parent = PathBuf::from("some/path.xml");
        let path_without_extension = PathBuf::from("just-a-random-file");

        assert_eq!(
            PathBuf::from("some/path (2).xml"),
            get_numbered_path(path_with_parent.clone(), 2)
        );

        assert_eq!(
            PathBuf::from("just-a-random-file (11)"),
            get_numbered_path(path_without_extension.clone(), 11)
        );

        assert_eq!(
            PathBuf::from("some/path.xml"),
            get_numbered_path(path_with_parent.clone(), 0)
        );

        assert_eq!(
            PathBuf::from("just-a-random-file"),
            get_numbered_path(path_without_extension.clone(), 0)
        );
    }
}
