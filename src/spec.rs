use std::fmt::{Display, Error, Formatter};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
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
        Err(err) => {
            eprintln!("Not pointing to a valid path: {} ({})", template_spec, err);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_template_spec_as_path_win() {
        assert_eq!(
            template_spec_as_path("C:\\Windows\\System32"),
            Some(PathBuf::from("\\\\?\\C:\\Windows\\System32"))
        );

        assert_eq!(
            template_spec_as_path("C:\\Windows\\..\\Data\\SomeRandomPath"),
            None
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_parse_template_spec_win() {
        let path_spec =
            TemplateSpec::Local(template_spec_as_path("C:\\Windows\\System32").unwrap());

        assert_eq!(parse_template_spec("C:\\Windows\\System32"), path_spec);
    }

    #[test]
    fn test_parse_template_spec() {
        let remote_spec = "git@github.com:v47-io/architect-rs.git";

        assert_eq!(
            parse_template_spec(remote_spec),
            TemplateSpec::Remote(remote_spec)
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_is_valid_template_spec_win() {
        assert_eq!(is_valid_template_spec("C:\\Windows\\System32"), true);
        assert_eq!(is_valid_template_spec("/Windows/System32"), false);
    }

    #[test]
    fn test_is_valid_template_spec() {
        assert_eq!(
            is_valid_template_spec("git@github.com:v47-io/architect-rs.git"),
            true
        );
        assert_eq!(
            is_valid_template_spec("git@github.com/v47-io/architect-rs.git"),
            false
        );
        assert_eq!(
            is_valid_template_spec("https://github.com/v47-io/architect-rs.git"),
            true
        );
        assert_eq!(is_valid_template_spec("https://github.com/v47-io/"), false);
    }
}
