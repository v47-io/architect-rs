use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs::{copy, create_dir_all, read_to_string, File};
use std::io;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use handlebars::{Context, Handlebars};
use indicatif::{MultiProgress, ProgressBar};
use walkdir::WalkDir;

use crate::helpers::{DIR_IF_HELPER, DIR_IF_YES};
use crate::utils::NEW_LINE_REGEX;

pub fn render(
    root_dir: &Path,
    target_dir: &Path,
    context: &Context,
    verbose: bool,
) -> io::Result<RenderResult> {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("dir-if", Box::new(DIR_IF_HELPER));

    let mut render_specs: HashMap<PathBuf, Vec<RenderSpec>> = HashMap::new();
    let mut dir_context_stack = vec![DirContext {
        source_path: root_dir.to_path_buf(),
        target_path: target_dir.to_path_buf(),
    }];

    let walk = WalkDir::new(root_dir);

    for entry_result in walk
        .into_iter()
        .filter_entry(|entry| entry.path() != root_dir.join(".git"))
    {
        let entry = entry_result?;
        let metadata = entry.metadata()?;

        while !dir_context_stack.is_empty()
            && !entry
                .path()
                .starts_with(&dir_context_stack.last().unwrap().source_path)
        {
            dir_context_stack.pop();
        }

        if verbose {
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
                create_entry_target_dir_name(&entry_dir_name, &handlebars, &context)
            } else {
                entry_dir_name
            };

            dir_context_stack.push(DirContext {
                source_path: entry.path().to_path_buf(),
                target_path: dir_context_stack
                    .last()
                    .unwrap()
                    .target_path
                    .join(entry_target_dir_name),
            })
        } else {
            let current_dir_ctx = dir_context_stack.last().unwrap();

            let source_file_name = entry.file_name().to_string_lossy().to_string();
            let is_template = source_file_name.to_lowercase().ends_with(".hbs");

            let target_file_name = if it_contains_template(&source_file_name) {
                create_entry_target_file_name(&source_file_name, &handlebars, &context)
            } else {
                source_file_name
            };

            let singular_target_path = current_dir_ctx.target_path.join(target_file_name);

            let render_specs_vec = get_render_specs_vec(&mut render_specs, &singular_target_path);

            render_specs_vec.push(RenderSpec {
                source: entry.into_path(),
                target: get_numbered_path(singular_target_path, render_specs_vec.len()),
                is_template,
            })
        };
    }

    let parallelism = min(1, max(4, num_cpus::get() / 2));

    let (input_tx, input_rx) = channel::<RenderSpec>();
    let input_rx = Arc::new(Mutex::new(input_rx));

    let rendered_files = Arc::new(Mutex::new(Vec::<RenderSpec>::new()));

    let all_progress = MultiProgress::new();

    let conflicts: Vec<RenderConflict> = crossbeam::scope(|scope| {
        let handlebars = &handlebars;

        (0..parallelism).for_each(|_| {
            let input_rx = Arc::clone(&input_rx);
            let rendered_files = Arc::clone(&rendered_files);

            let progress = all_progress.add(ProgressBar::new_spinner());

            scope.spawn(move |_| loop {
                progress.set_message("Waiting...");

                match input_rx.lock().unwrap().recv() {
                    Ok(render_spec) => {
                        if render_spec.is_template {
                            progress.set_message(format!(
                                "Rendering > {}",
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
                                "Copying   > {}",
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
                        progress.finish_with_message("No more work");
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
                                input_tx.send(it.clone()).unwrap();
                                it.source
                            })
                            .collect(),
                    })
                } else {
                    render_specs
                        .into_iter()
                        .for_each(|it| input_tx.send(it).unwrap());

                    None
                }
            })
            .collect();

        drop(input_tx);

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

#[derive(Clone, Debug)]
struct DirContext {
    source_path: PathBuf,
    target_path: PathBuf,
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

fn create_entry_target_dir_name(
    source_name: &str,
    handlebars: &Handlebars,
    context: &Context,
) -> String {
    let (result, was_rendered) = render_line_template(source_name, handlebars, context);

    if was_rendered && result == DIR_IF_YES {
        String::from(".")
    } else {
        result
    }
}

fn create_entry_target_file_name(
    source_name: &str,
    handlebars: &Handlebars,
    context: &Context,
) -> String {
    let (result, was_rendered) = render_line_template(&source_name, handlebars, context);

    if was_rendered && result.contains(DIR_IF_YES) {
        eprintln!("File name template contains 'dir-if': {}", source_name);
        source_name.to_string()
    } else {
        result
    }
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

fn get_numbered_path(base_path: PathBuf, number: usize) -> PathBuf {
    if number == 0 {
        base_path
    } else {
        let parent = base_path.parent().unwrap();
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

        parent.join(numbered_name)
    }
}

pub struct RenderResult {
    pub rendered_files: Vec<RenderSpec>,
    pub conflicts: Vec<RenderConflict>,
}

#[derive(Clone, Debug)]
pub struct RenderSpec {
    pub source: PathBuf,
    pub target: PathBuf,
    is_template: bool,
}

pub struct RenderConflict {
    pub intended_target: PathBuf,
    pub sources: Vec<PathBuf>,
}
