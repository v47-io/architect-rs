# Architect

[![.github/workflows/rust.yml][build-badge]][workflow-url]
[![codecov][codecov-badge]][codecov-url]

[build-badge]: https://github.com/v47-io/architect-rs/actions/workflows/tests.yml/badge.svg
[workflow-url]: https://github.com/v47-io/architect-rs/actions/workflows/tests.yml

[codecov-badge]: https://codecov.io/gh/v47-io/architect-rs/branch/master/graph/badge.svg?token=FDASC57M7H
[codecov-url]: https://codecov.io/gh/v47-io/architect-rs

> A straightforward and technology-agnostic project scaffolding tool

Architect works using a few simple features:

- Clone template using Git
- Ask any defined questions to determine some user-specific values
- Render Handlebars templates using those values as input

Architect uses this [Handlebars implementation](https://docs.rs/crate/handlebars/4.1.2).

For more information you can also refer to [handlebarsjs.com](https://handlebarsjs.com/), however, not all features may
be supported.

## [Documentation](https://v47-io.github.io/architect-rs/)

## Handlebars Templating

Architect uses Handlebars to render all templates in a repository. Basically any file is a potential 
template and will be processed.

Files that are explicitly marked as Handlebars templates by their extension are not rendered, only copied.
This behavior (and the extension) can be configured in the `.architect.json` file.

Additionally, every file name is a potential Handlebars template. That means you can specify the curled braces in 
file names to dynamically create them using your user-specific values.

__Example:__

```
Template: {{app.name}}.rs

Context: {
  "app": {
    "name": "my_app"
  }
}

Result: my_app.rs
```

It works the same way with directory names, but there you have the added benefit to potentially create multiple levels
of subdirectories. Should a path you create this way leave the target directory the template will be ignored.

__Example:__

```
Template: {{package src.package}}

Context: {
  "src": {
    "package": "io.v47"
  }
}

Result: io/v47
```

### Helpers

Architect provides the basic helpers included in the handlebars library, and also includes all helpers provided by
the [handlebars_misc_helpers](https://docs.rs/crate/handlebars_misc_helpers/0.12.1) library.

In addition to those helpers Architect offers another helper: `package`

#### package

This helper is used to create a path from dot-separated values, e.g. a Java package.

Use this to create a nested directory structure for your files.

`package` is not available in the actual Handlebars template files, there you should just use `replace` 
to swap the dots for any other character you like.

__Example:__

```
Template: {{package src.package}}

Context: {
  "src": {
    "package": "io.v47"
  }
}

Result: io/v47
```

## License and Contributions

Architect is provided under the terms of the BSD 3-Clause License.

Contributions are welcome, but any contributor must have all rights to the contributed material and agree to provide it
under the terms of the aforementioned BSD 3-Clause License.
