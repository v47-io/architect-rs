# Architect

[![.github/workflows/rust.yml][build-badge]][workflow-url]
[![codecov][codecov-badge]][codecov-url]

[build-badge]: https://github.com/v47-io/architect-rs/actions/workflows/rust.yml/badge.svg
[workflow-url]: https://github.com/v47-io/architect-rs/actions/workflows/rust.yml

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

## Handlebars Templating

Architect uses Handlebars for all the templates in a repository. Any file with the extension `.hbs` will be picked up
and rendered using Handlebars.

Additionally, every file name is a potential Handlebars template. That means you can specify the curled braces in 
file names to dynamically specify them using your user-specific values.

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
Template: {{replace src.package "." "/"}}
Context: {
  "src": {
    "package": "io.v47"
  }
}
Result: io/v47
```

### Helpers

Architect provides the basic helpers included in the handlebars library, and also includes all helpers provided by
the [handlebars_misc_helpers](https://docs.rs/crate/handlebars_misc_helpers/0.12.1) library. The previously mentioned 
`replace` is such a helper.

#### dir-if

In addition to those helpers Architect offers one more helper: `dir-if`

This helper can be used to determine whether a certain directory/file tree should be included or not. 

The two possible outcomes are that either all children of the directory are included at the current location or the
entire content is ignored. This only works if the template containing `dir-if` is the only content of the directory 
name.

`dir-if` is not available in the actual Handlebars template files.

__Example (excluding children):__

```
Directory Tree:
  - some dir
  - another dir
  - {{dir-if include_tests}}
     - test1.txt
     - test2.txt
  - anotherfile.txt
Context: {
  "include_tests": false
}
Result Tree:
  - some dir
  - another dir
  - anotherfile.txt
```

__Example (including children):__

```
Directory tree:
  - some dir
  - another dir
  - {{dir-if include_tests}}
     - test1.txt
     - test2.txt
  - anotherfile.txt
Context: {
  "include_tests": true
}
Result Tree:
  - some dir
  - another dir
  - test1.txt
  - test2.txt
  - anotherfile.txt
```

## Configuration

Any template repository can contain an `.architect.json` file that defines the questions to ask before rendering the
templates.

### Format (described using TypeScript syntax):

```typescript
interface RootObject {
    /**
     * The name of the template
     */
    name: string;
    /**
     * The version of the template
     */
    version: string;

    questions: Question[];
}

interface Question {
    /**
     * The name that you use in the Handlebars templates to access the value(s) entered in response to the question
     */
    name: String;
    /**
     * What kind of value(s) are expected
     */
    type: QuestionType;
    /**
     * An optional pretty message printed when asking for a value
     */
    pretty: String | undefined;
}

enum QuestionType {
    Identifier,
    Option,
    Selection,
    Text
}

interface SimpleQuestion extends Question {
    type: QuestionType.Identifier | QuestionType.Option | QuestionType.Text;
}

interface SelectionQuestion extends Question {
    type: QuestionType.Selection;

    /**
     * A list of items that are available to be selected
     */
    items: String[];
    /**
     * Determines whether you can choose multiple items from the list of available items
     */
    multi: Boolean | undefined;
}
```

### Value Nesting

Each question specifies a unique name. You can use dot-separated names to define nested values in the Handlebars
templates.

__Example:__

```
question 1 name: "source.directory"
question 2 name: "source.package"
```

Handlebars Context:

```json

{
  "source": {
    "directory": "some value",
    "package": "another value"
  }
}
```

### Predefined Values

The Handlebars context contains the following predefined values that cannot be overwritten using questions:

```json
{
  "__template__": {
    "name": "name of the template",
    "version": "version of the template"
  }
}
```

### Question Types

- __Identifier__:

  Format: `^[a-zA-Z_$][a-zA-Z0-9_$]*$`

  Useful where the value is used as an identifier (e.g. variable name, class name, etc...) in a Handlebars template

- __Option__:

  Just a boolean flag. Will be visible as `true` (truthy) in the Handlebars template

- __Selection__:

  Item Format: `^[a-zA-Z_$][a-zA-Z0-9_$]*$`

  Useful where you want to offer a predefined list of values to choose from. The selected value(s) will be created as
  properties in the Handlebars template and have the value `true` (truthy).

- __Text__:

  Useful when you want to insert arbitrary text in a Handlebars template

## Usage

```
USAGE:
    architect [FLAGS] [OPTIONS] <REPOSITORY> [TARGET]

FLAGS:
        --dirty
            Uses the template repository in it's current (dirty) state.

            This only has an effect if a local path is specified as the repository. In that
            case Architect won't perform a clean clone but will just copy the directory,
            regardless of the local state.

            This is most useful to test a template locally, for remote repositories this
            option doesn't make sense.
    -h, --help
            Prints help information

    -V, --version
            Prints version information

        --verbose
            Enables verbose output


OPTIONS:
    -b, --branch <branch>
            The remote branch to fetch instead of the default branch

ARGS:
    <REPOSITORY>
            The git repository to use as the project template.

            This can be specified in any way that you can refer to a git repository,
            i.e. an HTTP(S) URL, ssh connection string, or a local path.

            Example: git@github.com:some-user/his-template-repo.git
    <TARGET>
            The target directory for the final output.

            This defaults to the Git repository name as a child of the current working directory.
```

## Development

How to build:

```shell
cargo build --bin architect
```

How to test:

```shell
cargo test --bin architect
```

The result should be an `architect(\.exe)?` file in the `./target/debug` directory, ready for execution.

## License and Contributions

Architect is provided under the terms of the BSD 3-Clause License.

Contributions are welcome, but any contributor must have all rights to the contributed material and agree to provide it
under the terms of the aforementioned BSD 3-Clause License.
