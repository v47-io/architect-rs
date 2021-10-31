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

use globset::GlobMatcher;
use handlebars::{Context, Handlebars, RenderError};
use indicatif::{MultiProgress, ProgressBar};
use lazy_static::lazy_static;
use path_absolutize::Absolutize;
use serde_json::{Map, Value};
use walkdir::WalkDir;

use crate::config::{ConditionalFilesSpec, Config};
use crate::context::UnsafeContext;
use crate::helpers::PACKAGE_HELPER;
use crate::utils::reader::BufReader;
use crate::utils::{ToolConfig, NEW_LINE_REGEX};

lazy_static! {
    static ref RENDER_PARALLELISM: usize =
        if let Ok(Ok(value)) = var("RENDER_PARALLELISM").map(|raw| usize::from_str(&raw)) {
            max(1, value)
        } else {
            max(1, min(4, num_cpus::get() / 2))
        };

    static ref TEMPLATE_INSPECT_MAX_LINES: usize =
        if let Ok(Ok(value)) = var("TEMPLATE_INSPECT_MAX_LINES").map(|raw| usize::from_str(&raw)) {
            max(1, value)
        } else {
            // this feels like a sensible default, like who puts a Handlebars expression at the end of a 1000 line template?
            25
        };

    static ref HANDLEBARS_XTS: Vec<&'static str> = vec![".hbs", ".handlebars"];
}

pub fn render(
    root_dir: &Path,
    target_dir: &Path,
    config: &Config,
    context: &Context,
    tool_config: &ToolConfig,
) -> io::Result<RenderResult> {
    let mut handlebars = create_hbs();
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

    render_specs_progress.finish_with_message("Render specifications built");

    // Creating new Handlebars instance without helpers that shouldn't be used in templates
    handlebars = create_hbs();

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

                            let file_context = build_file_context(context, &render_spec, root_dir);

                            match render_template_to_file(
                                &render_spec.source,
                                &render_spec.target,
                                handlebars,
                                &file_context,
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

fn create_hbs<'a>() -> Handlebars<'a> {
    let mut instance = Handlebars::new();
    handlebars_misc_helpers::register(&mut instance);

    instance
}

fn build_file_context(base_ctx: &Context, render_spec: &RenderSpec, root_dir: &Path) -> Context {
    let mut context_map = match base_ctx.data() {
        Value::Object(map) => map.clone(),
        wrong_value => panic!(
            "Context data should be Value::Object, not {:?}",
            wrong_value
        ),
    };

    let mut file_map = Map::new();
    file_map.insert(
        "rootDir".into(),
        Value::String(root_dir.to_string_lossy().into()),
    );
    file_map.insert(
        "sourceName".into(),
        Value::String(
            render_spec
                .source
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into(),
        ),
    );
    file_map.insert(
        "sourcePath".into(),
        Value::String(render_spec.source.to_string_lossy().into()),
    );
    file_map.insert(
        "targetName".into(),
        Value::String(
            render_spec
                .target
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into(),
        ),
    );
    file_map.insert(
        "targetPath".into(),
        Value::String(render_spec.target.to_string_lossy().into()),
    );

    if !context_map.contains_key("__template__") {
        context_map.insert("__template__".into(), Value::Object(Map::new()));
    }

    let template_map = match context_map.get_mut("__template__") {
        Some(Value::Object(map)) => map,
        _ => unreachable!(),
    };

    template_map.insert("file".into(), Value::Object(file_map));

    UnsafeContext::new(context_map).into()
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

    // We already add the root directory to the stack here so we don't have to deal with it
    // in the loop body, which would make it even more complex than it already is
    let mut dir_context_stack = vec![DirContext {
        source_path: root_dir.to_path_buf(),
        target_path: Some(target_dir.to_path_buf()),
    }];

    let walk = WalkDir::new(root_dir);

    for entry_result in walk
        .into_iter()
        .filter_entry(|entry| {
            include_dir_entry(
                entry.path(),
                entry.metadata().map_or(false, |meta| meta.is_dir()),
                root_dir,
                config,
                hbs,
                ctx,
                tool_config,
            )
        })
        // We skip the root directory here, otherwise we would have to handle it separately in the loop body
        .skip(1)
    {
        let entry = entry_result?;
        let metadata = entry.metadata()?;

        while dir_context_stack
            .last()
            .map_or(false, |it| !entry.path().starts_with(&it.source_path))
        {
            dir_context_stack.pop();
        }

        if dir_context_stack.last().unwrap().target_path.is_none() {
            if tool_config.verbose {
                println!("Skipping path:        {}", entry.path().display())
            }

            continue;
        }

        if tool_config.verbose {
            println!("Processing path:      {}", entry.path().display());
            println!("Directory context:    {:?}", dir_context_stack.last())
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
                source_path: entry.path().absolutize()?.to_path_buf(),
                target_path,
            })
        } else {
            let current_dir_ctx = dir_context_stack.last().unwrap();

            let source_file_name = entry.file_name().to_string_lossy().to_string();

            let is_potential_template = if let Some(templates) = &config.filters.templates {
                matches_globs(templates, root_dir, entry.path())
            } else if let Some(non_templates) = &config.filters.non_templates {
                !matches_globs(non_templates, root_dir, entry.path())
            } else {
                true
            };

            let is_template = is_potential_template && is_hbs_template(entry.path())?;

            let mut target_file_name = if it_contains_template(&source_file_name) {
                create_entry_target_file_name(&source_file_name, hbs, ctx)
            } else {
                source_file_name
            };

            if is_template {
                target_file_name = strip_handlebars_xt(target_file_name);
            }

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
                source: entry.into_path().absolutize()?.to_path_buf(),
                target: get_numbered_path(singular_target_path, render_specs_vec.len()),
                is_template,
            })
        };
    }

    Ok(render_specs)
}

fn include_dir_entry(
    path: &Path,
    path_is_dir: bool,
    root_dir: &Path,
    config: &Config,
    hbs: &Handlebars,
    ctx: &Context,
    tool_config: &ToolConfig,
) -> bool {
    is_not_git_dir_in_root(path, root_dir)
        && is_not_hidden_or_is_included(path, path_is_dir, root_dir, config, tool_config)
        && is_not_excluded(path, root_dir, config, tool_config)
        && is_conditionally_included(path, root_dir, config, hbs, ctx, tool_config)
}

#[inline]
fn is_not_git_dir_in_root(path: &Path, root_dir: &Path) -> bool {
    path != root_dir.join(".git")
}

fn is_not_hidden_or_is_included(
    path: &Path,
    path_is_dir: bool,
    root_dir: &Path,
    config: &Config,
    tool_config: &ToolConfig,
) -> bool {
    path_is_dir
        || path
            .strip_prefix(root_dir)
            .map(|globbing_path| {
                let check_glob = || {
                    let result = config
                        .filters
                        .include_hidden
                        .iter()
                        .any(|matcher| matcher.is_match(globbing_path));

                    if !result && tool_config.verbose {
                        println!("Skipping hidden path: {}", path.display())
                    }

                    result
                };

                globbing_path
                    .iter()
                    .all(|name| !name.to_string_lossy().starts_with('.'))
                    || check_glob()
            })
            .unwrap_or_else(|err| {
                eprintln!(
                    "Failed to strip root dir prefix from path {} ({})",
                    path.display(),
                    err
                );

                false
            })
}

fn is_conditionally_included(
    path: &Path,
    root_dir: &Path,
    config: &Config,
    hbs: &Handlebars,
    ctx: &Context,
    tool_config: &ToolConfig,
) -> bool {
    path.strip_prefix(root_dir)
        .map(|globbing_path| {
            if let Some(cond_spec) =
                find_condition_spec(globbing_path, &config.filters.conditional_files)
            {
                let skip = match eval_condition(cond_spec, hbs, ctx) {
                    Ok(is_match) => !is_match,
                    Err(e) => {
                        eprintln!(
                            "Failed to evaluate condition \"{}\" for {} ({})",
                            cond_spec.condition,
                            path.display(),
                            e
                        );

                        !tool_config.ignore_checks
                    }
                };

                if skip && tool_config.verbose {
                    println!("Skipping path:        {}", path.display())
                }

                // inverting because filter needs false to exclude
                !skip
            } else {
                // Didn't find a condition spec so file is not excluded
                true
            }
        })
        .unwrap_or_else(|err| {
            eprintln!(
                "Failed to strip root dir prefix from path {} ({})",
                path.display(),
                err
            );

            false
        })
}

fn is_not_excluded(
    path: &Path,
    root_dir: &Path,
    config: &Config,
    tool_config: &ToolConfig,
) -> bool {
    path.strip_prefix(root_dir)
        .map(|globbing_path| {
            if config
                .filters
                .exclude
                .iter()
                .any(|glob| glob.is_match(globbing_path))
            {
                if tool_config.verbose {
                    println!("Skipping path:        {}", path.display())
                }

                false
            } else {
                true
            }
        })
        .unwrap_or_else(|err| {
            eprintln!(
                "Failed to strip root dir prefix from path {} ({})",
                path.display(),
                err
            );

            false
        })
}

fn find_condition_spec<'a>(
    path: &Path,
    conditional_files_specs: &'a [ConditionalFilesSpec<'a>],
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
    match handlebars.render_template_with_context(
        &format!("{{{{ {} }}}}", conditional_files_spec.condition),
        context,
    ) {
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

fn matches_globs(globs: &[GlobMatcher], root_dir: &Path, path: &Path) -> bool {
    path.strip_prefix(root_dir)
        .map(|globbing_path| globs.iter().any(|it| it.is_match(globbing_path)))
        .unwrap_or_else(|err| {
            eprintln!(
                "Failed to strip root dir prefix from path {} ({})",
                path.display(),
                err
            );

            false
        })
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
    target_path: &Path,
) -> &'spec mut Vec<RenderSpec> {
    if !render_specs.contains_key(target_path) {
        render_specs.insert(target_path.to_path_buf(), vec![]);
    }

    render_specs.get_mut(target_path).unwrap()
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
    render_line_template(source_name, handlebars, context).0
}

fn is_hbs_template(path: &Path) -> io::Result<bool> {
    Ok(BufReader::open(path)?
        .into_iter()
        .take(*TEMPLATE_INSPECT_MAX_LINES)
        .any(|line| match line {
            Ok(line) => it_contains_template(line.trim()),
            Err(err) => {
                eprintln!("Failed to read line into the buffer ({})", err);
                false
            }
        }))
}

fn strip_handlebars_xt(name: String) -> String {
    let name_lower = name.to_lowercase();

    HANDLEBARS_XTS
        .iter()
        .find(|&&xt| name_lower.ends_with(xt))
        .map(|xt| String::from(&name[..name.len() - xt.len()]))
        .unwrap_or(name)
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
        let numbered_name = if let Some(dot_i) = file_name.rfind('.') {
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

#[derive(Debug)]
pub struct RenderResult {
    pub rendered_files: Vec<RenderSpec>,
    pub conflicts: Vec<RenderConflict>,
}

#[derive(Debug)]
pub struct RenderConflict {
    pub intended_target: PathBuf,
    pub sources: Vec<PathBuf>,
}

#[cfg(test)]
#[allow(clippy::redundant_closure_call)]
mod tests {
    use std::fs::read_to_string;

    use itertools::Itertools;
    use lazy_static::lazy_static;
    use serde_json::{Map, Number, Value};
    use tempfile::{tempdir, TempDir};

    use crate::config::Filters;
    use crate::context::UnsafeContext;
    use crate::utils::glob;
    use crate::utils::tests::RESOURCES_DIR;

    use super::*;

    lazy_static! {
        static ref HANDLEBARS: Handlebars<'static> = (|| {
            let mut handlebars = create_hbs();
            handlebars.register_helper("package", Box::new(PACKAGE_HELPER));

            handlebars
        })();
        static ref TEMP_DIR: TempDir = tempdir().unwrap();
    }

    #[test]
    fn test_render() -> io::Result<()> {
        let target_temp_dir = tempdir()?;
        let target_path = target_temp_dir.path().join("my-project");

        let source_path = RESOURCES_DIR
            .join("auto-template.input")
            .absolutize()?
            .to_path_buf();

        let mut context_map = Map::new();
        let mut author_map = Map::new();
        author_map.insert("name".into(), Value::String("Some dude!".into()));

        context_map.insert("author".into(), Value::Object(author_map));
        context_map.insert("somePackage".into(), Value::String("io.v47.test".into()));

        let context = UnsafeContext::new(context_map).into();

        let config = Config {
            name: Some("Auto Template"),
            version: Some("0.x"),
            questions: vec![],
            filters: Filters {
                include_hidden: vec![glob("**/*still-included*").unwrap()],
                exclude: vec![glob("*excluded*").unwrap()],
                conditional_files: vec![],
                templates: None,
                non_templates: Some(vec![glob("**/*.handlebars").unwrap()]),
            },
        };

        let tool_config = ToolConfig {
            no_history: false,
            no_init: false,
            ignore_checks: false,
            verbose: true,
        };

        let render_result = render(&source_path, &target_path, &config, &context, &tool_config)?;

        assert_eq!(4, render_result.rendered_files.len());
        assert_eq!(1, render_result.conflicts.len());

        let render_conflict = &render_result.conflicts[0];

        assert_eq!(
            target_path
                .join("io/v47/test/file-in-generated-path.txt")
                .absolutize()?,
            render_conflict.intended_target
        );
        assert_eq!(2, render_conflict.sources.len());

        let override_template_path = target_path
            .join("templates/override-template.txt.handlebars")
            .absolutize()?
            .to_path_buf();

        assert!(override_template_path.exists());
        assert!(is_hbs_template(&override_template_path)?);

        let sep = std::path::MAIN_SEPARATOR;

        let check_target_dir_content = vec![
            format!(".hidden-dir{}but-still-included.txt", sep),
            format!("io{}v47{}test{}file-in-explicit-path.txt", sep, sep, sep),
            format!(
                "io{}v47{}test{}file-in-generated-path (1).txt",
                sep, sep, sep
            ),
            format!("io{}v47{}test{}file-in-generated-path.txt", sep, sep, sep),
            format!("templates{}override-template.txt.handlebars", sep),
            format!("templates{}some-template.txt", sep),
        ];

        let target_dir_content = WalkDir::new(&target_path)
            .into_iter()
            .skip(1)
            .filter_ok(|entry| !entry.metadata().unwrap().is_dir())
            .map(|entry| {
                entry
                    .unwrap()
                    .path()
                    .strip_prefix(&target_path)
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .sorted()
            .collect::<Vec<_>>();

        assert_eq!(check_target_dir_content, target_dir_content);

        Ok(())
    }

    #[test]
    fn test_build_file_context() -> io::Result<()> {
        let target_temp_dir = tempdir()?;
        let target_path = target_temp_dir.path().join("my-project");

        let source_path = RESOURCES_DIR
            .join("auto-template.input")
            .absolutize()?
            .to_path_buf();

        let context_map = Map::new();
        let context = UnsafeContext::new(context_map).into();

        let file_context = build_file_context(
            &context,
            &RenderSpec {
                source: source_path.join("abcdef.hbs"),
                target: target_path.join("abcdef"),
                is_template: true,
            },
            &source_path,
        );

        let file_context_map = match file_context.data() {
            Value::Object(map) => map,
            _ => panic!(),
        };

        let template_map = match file_context_map.get("__template__") {
            Some(Value::Object(map)) => map,
            _ => panic!(),
        };

        let file_map = match template_map.get("file") {
            Some(Value::Object(map)) => map,
            _ => panic!(),
        };

        let mut check_file_map = Map::new();
        check_file_map.insert(
            "rootDir".into(),
            Value::String(source_path.to_string_lossy().into()),
        );
        check_file_map.insert("sourceName".into(), Value::String("abcdef.hbs".into()));
        check_file_map.insert(
            "sourcePath".into(),
            Value::String(source_path.join("abcdef.hbs").to_string_lossy().into()),
        );
        check_file_map.insert("targetName".into(), Value::String("abcdef".into()));
        check_file_map.insert(
            "targetPath".into(),
            Value::String(target_path.join("abcdef").to_string_lossy().into()),
        );

        assert_eq!(&check_file_map, file_map);

        Ok(())
    }

    #[test]
    fn test_build_render_specs() -> io::Result<()> {
        let target_temp_dir = tempdir()?;
        let target_path = target_temp_dir.path().join("my-project");

        let source_path = RESOURCES_DIR
            .join("auto-template.input")
            .absolutize()?
            .to_path_buf();

        let mut context_map = Map::new();
        let mut author_map = Map::new();
        author_map.insert("name".into(), Value::String("Some dude!".into()));

        context_map.insert("author".into(), Value::Object(author_map));
        context_map.insert("somePackage".into(), Value::String("io.v47.test".into()));

        let context = UnsafeContext::new(context_map).into();

        let config = Config {
            name: Some("Auto Template"),
            version: Some("0.x"),
            questions: vec![],
            filters: Filters {
                include_hidden: vec![glob("**/*still-included*").unwrap()],
                exclude: vec![glob("*excluded*").unwrap()],
                conditional_files: vec![],
                templates: None,
                non_templates: Some(vec![glob("**/some-template.txt.hbs").unwrap()]),
            },
        };

        let tool_config = ToolConfig {
            no_history: false,
            no_init: false,
            ignore_checks: false,
            verbose: true,
        };

        let render_specs = build_render_specs(
            &source_path,
            &target_path,
            &config,
            &HANDLEBARS,
            &context,
            &tool_config,
        )?;

        // todo: actually verify result
        //       test other code paths
        println!("{:?}", render_specs);

        let sep = std::path::MAIN_SEPARATOR;

        let check_target_paths = vec![
            format!(".hidden-dir{}but-still-included.txt", sep),
            format!("io{}v47{}test{}file-in-explicit-path.txt", sep, sep, sep),
            format!("io{}v47{}test{}file-in-generated-path.txt", sep, sep, sep),
            format!("templates{}override-template.txt", sep),
            format!("templates{}some-template.txt.hbs", sep),
        ]
        .iter()
        .map(|it| target_path.join(it))
        .collect::<Vec<PathBuf>>();

        let mut expected_target_paths = check_target_paths.iter().collect::<Vec<&PathBuf>>();
        expected_target_paths.sort();

        let mut actual_target_paths = render_specs.keys().collect::<Vec<&PathBuf>>();
        actual_target_paths.sort();

        assert_eq!(expected_target_paths, actual_target_paths);

        let conflict_render_specs = render_specs.get(&check_target_paths[2]).unwrap();
        assert_eq!(2, conflict_render_specs.len());
        assert_eq!(
            1,
            conflict_render_specs
                .iter()
                .filter(|&it| it.is_template)
                .count()
        );

        let other_render_specs = render_specs.get(&check_target_paths[1]).unwrap();
        assert_eq!(1, other_render_specs.len());

        Ok(())
    }

    #[test]
    fn test_include_dir_entry() -> io::Result<()> {
        let temp_root_dir = tempdir()?;
        let root_dir = temp_root_dir.path();

        let config = Config {
            name: None,
            version: None,
            questions: vec![],
            filters: Filters {
                conditional_files: vec![
                    ConditionalFilesSpec {
                        condition: "someValue",
                        matcher: glob("matched_file").unwrap(),
                    },
                    ConditionalFilesSpec {
                        condition: "not someValue",
                        matcher: glob("unmatched_file").unwrap(),
                    },
                ],
                include_hidden: vec![glob(".github").unwrap()],
                exclude: vec![glob("excluded_file").unwrap()],
                templates: None,
                non_templates: None,
            },
        };

        let mut context_map = Map::new();
        context_map.insert("someValue".into(), Value::Bool(true));

        let context = UnsafeContext::new(context_map).into();

        let tool_config = ToolConfig {
            verbose: true,
            ignore_checks: false,
            no_history: false,
            no_init: false,
        };

        assert!(!include_dir_entry(
            &root_dir.join(".git"),
            true,
            root_dir,
            &config,
            &HANDLEBARS,
            &context,
            &tool_config
        ));

        assert!(!include_dir_entry(
            &root_dir.join(".gradle/build-cache"),
            false,
            root_dir,
            &config,
            &HANDLEBARS,
            &context,
            &tool_config
        ));

        assert!(include_dir_entry(
            &root_dir.join(".github"),
            true,
            root_dir,
            &config,
            &HANDLEBARS,
            &context,
            &tool_config
        ));

        assert!(include_dir_entry(
            &root_dir.join("matched_file"),
            false,
            root_dir,
            &config,
            &HANDLEBARS,
            &context,
            &tool_config
        ));

        assert!(!include_dir_entry(
            &root_dir.join("unmatched_file"),
            false,
            root_dir,
            &config,
            &HANDLEBARS,
            &context,
            &tool_config
        ));

        assert!(!include_dir_entry(
            &root_dir.join("excluded_file"),
            false,
            root_dir,
            &config,
            &HANDLEBARS,
            &context,
            &tool_config
        ));

        Ok(())
    }

    #[test]
    fn test_is_hbs_template() -> io::Result<()> {
        let template_file = RESOURCES_DIR.join("simple-template.input/simple-template.html.hbs");

        assert!(is_hbs_template(&template_file)?);

        let non_template_file =
            RESOURCES_DIR.join("simple-template.expected/en/simple-template.html");

        assert!(!is_hbs_template(&non_template_file)?);

        Ok(())
    }

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

        assert!(eval_condition(
            &ConditionalFilesSpec {
                condition: "simple",
                matcher: glob("").unwrap()
            },
            &HANDLEBARS,
            &context
        )
        .unwrap_or(false));

        assert!(!eval_condition(
            &ConditionalFilesSpec {
                condition: "falsy",
                matcher: glob("").unwrap()
            },
            &HANDLEBARS,
            &context
        )
        .unwrap_or(false));
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
            get_numbered_path(path_with_parent, 0)
        );

        assert_eq!(
            PathBuf::from("just-a-random-file"),
            get_numbered_path(path_without_extension, 0)
        );
    }
}
