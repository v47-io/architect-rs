use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::Command;
use std::str;

use dircpy::copy_dir;
use git2::{Error, Repository};

use crate::spec::TemplateSpec;

pub struct FetchOptions<'f> {
    pub branch: Option<&'f str>,
    pub dirty: bool,
    pub verbose: bool,
}

pub fn fetch(template_spec: &TemplateSpec, target: &Path, options: FetchOptions) -> io::Result<()> {
    let (copy_dirty, local_path) = match template_spec {
        TemplateSpec::Local(local_path) => (
            options.dirty || !is_git_repo(local_path, options.verbose),
            Some(local_path),
        ),
        _ => (false, None),
    };

    println!("Fetching template from {}", template_spec);

    if copy_dirty {
        let local_path = local_path.unwrap();

        if options.verbose {
            println!("Copying dirty template from \"{}\"", local_path.display())
        }

        copy_dir(local_path, target)?
    } else {
        if options.verbose {
            println!("Cloning template using Git from \"{}\"", template_spec)
        }

        let mut command = Command::new("git");
        command.arg("clone");
        command.arg(match template_spec {
            TemplateSpec::Local(path) => path.as_os_str(),
            TemplateSpec::Remote(spec) => OsStr::new(spec),
        });

        match options.branch {
            Some(branch) => {
                command.args(&["--branch", branch]);
            }
            _ => (),
        }

        let mut child = command.spawn()?;
        while let None = child.try_wait()? {}
    }

    Ok(())
}

pub fn get_default_branch_name() -> io::Result<String> {
    let global_config = Command::new("git")
        .args(&["config", "--global", "--get", "init.defaultBranch"])
        .output()?;

    let global_config_value = str::from_utf8(global_config.stdout.as_slice())
        .unwrap()
        .trim();

    let result = if global_config_value.is_empty() {
        let system_config = Command::new("git")
            .args(&["config", "--system", "--get", "init.defaultBranch"])
            .output()?;

        let system_config_value = str::from_utf8(system_config.stdout.as_slice())
            .unwrap()
            .trim();

        if system_config_value.is_empty() {
            String::from("main")
        } else {
            system_config_value.to_owned()
        }
    } else {
        global_config_value.to_owned()
    };

    Ok(result)
}

pub fn open_git_repo_or_init(path: &Path, verbose: bool) -> Result<Repository, Error> {
    if verbose {
        println!("Trying to open \"{}\" as a Git repository", path.display());
    }

    match Repository::open(path) {
        Ok(repo) => {
            if verbose {
                println!("Opened Git repository");
            }

            repo.remote_delete("origin").unwrap_or_else(|_| ());

            Ok(repo)
        }
        Err(err) => {
            if verbose {
                eprintln!("Failed to open Git repository ({})", err);
                println!("Initializing Git repository in \"{}\"", path.display());
            }

            Repository::init(path)
        }
    }
}

fn is_git_repo(path: &Path, verbose: bool) -> bool {
    if verbose {
        println!(
            "Checking if directory \"{}\" contains a Git repository",
            path.display()
        );
    }

    let result = match Repository::open(path) {
        Ok(_) => true,
        Err(_) => false,
    };

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
