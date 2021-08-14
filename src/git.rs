use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::Command;
use std::str;

use dircpy::copy_dir;
use git2::Repository;

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
            println!("Copying dirty template from \"{}\"", local_path.display());
        }

        copy_dir(local_path, target)?
    } else {
        if options.verbose {
            println!("Cloning template using Git from \"{}\"", template_spec);
        }

        let mut command = Command::new("git");
        command.arg("clone");
        command.arg(match template_spec {
            TemplateSpec::Local(path) => path.as_os_str(),
            TemplateSpec::Remote(spec) => OsStr::new(spec),
        });
        command.arg(target.as_os_str());

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

pub fn copy_git_directory(parent_dir: &Path, target_dir: &Path, verbose: bool) -> io::Result<()> {
    if is_git_repo(parent_dir, false) {
        if verbose {
            println!("Copying .git directory from working directory to target directory")
        }

        copy_dir(parent_dir.join(".git"), target_dir.join(".git"))
    } else {
        Ok(())
    }
}
