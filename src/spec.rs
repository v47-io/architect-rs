use std::fmt::{Display, Error, Formatter};
use std::path::{Path, PathBuf};

pub enum TemplateSpec<'spec> {
    Local(PathBuf),
    Remote(&'spec str),
}

impl Display for TemplateSpec<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            TemplateSpec::Local(path) => write!(f, "{}", path.display()),
            TemplateSpec::Remote(spec) => write!(f, "{}", spec),
        }
    }
}

pub fn is_valid_template_spec(spec: &str) -> bool {
    if let Some(schema_delim_i) = spec.find("://") {
        if let Some(slash_index) = spec.rfind('/') {
            slash_index > schema_delim_i && slash_index < spec.len() - 1
        } else {
            false
        }
    } else if let Some(at_index) = spec.find('@') {
        if let Some(colon_index) = spec.rfind(':') {
            colon_index > at_index && colon_index < spec.len() - 1
        } else {
            false
        }
    } else {
        template_spec_as_path(spec).is_some()
    }
}

pub fn parse_template_spec(template_spec_raw: &str) -> TemplateSpec {
    match template_spec_as_path(template_spec_raw) {
        Some(dir) => TemplateSpec::Local(dir),
        None => TemplateSpec::Remote(template_spec_raw),
    }
}

fn template_spec_as_path(template_spec: &str) -> Option<PathBuf> {
    match Path::new(template_spec).canonicalize() {
        Ok(dir) => Some(dir),
        Err(_) => None,
    }
}
