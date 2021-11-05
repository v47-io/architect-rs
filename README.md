# Architect

[![License][license-badge]][license-url]
[![.github/workflows/rust.yml][build-badge]][workflow-url]
[![codecov][codecov-badge]][codecov-url]
[![Latest Release][release-badge]][release-url]

[license-badge]: https://img.shields.io/github/license/v47-io/architect-rs

[license-url]: https://github.com/v47-io/architect-rs/blob/master/LICENSE

[build-badge]: https://github.com/v47-io/architect-rs/actions/workflows/tests.yml/badge.svg

[workflow-url]: https://github.com/v47-io/architect-rs/actions/workflows/tests.yml

[codecov-badge]: https://codecov.io/gh/v47-io/architect-rs/branch/master/graph/badge.svg?token=FDASC57M7H

[codecov-url]: https://codecov.io/gh/v47-io/architect-rs

[release-badge]: https://img.shields.io/github/v/release/v47-io/architect-rs?include_prereleases

[release-url]: https://github.com/v47-io/architect-rs/releases

Architect is a straightforward and technology-agnostic project scaffolding tool.

This means you can prepare templates for projects using any technology and Architect will spit out a perfect new
project.

For its templating Architect uses the Handlebars templating language. Please check the documentation for more
information.

## [&gt;&gt; 📚 Documentation &lt;&lt;](https://v47-io.github.io/architect-rs/)

## TL;DR

Architect uses Handlebars and Git to create proper projects from template repositories.

1. Add any file to your template repository and add Handlebars expressions to it. ([docs][rendering-docs])
2. Add an `.architect.json` configuration file with your questions ([docs][config-docs])
3. Download the `architect` executable for your platform from the
   latest [Release](https://github.com/v47-io/architect-rs/releases).
4. Execute `architect <PATH-TO-REPO>` in your desired local directory, answer user-defined questions, et voila, you got
   a fully functional project created from a template. ([docs][cli-docs])

[rendering-docs]: https://v47-io.github.io/architect-rs/templates/rendering/

[config-docs]: https://v47-io.github.io/architect-rs/templates/configuration/

[cli-docs]: https://v47-io.github.io/architect-rs/cli/index.html

### Sample `.architect.json`

```json
{
  "name": "My Awesome Microservice Template",
  "version": "1.0",
  "questions": [
    {
      "name": "author.name",
      "type": "Text",
      "pretty": "What's your name?"
    },
    {
      "name": "author.email",
      "type": "Text",
      "pretty": "What's your email address?"
    },
    {
      "name": "project.package",
      "type": "Identifier",
      "pretty": "What should be the root package for your Kotlin sources?"
    },
    {
      "name": "project.features",
      "type": "Selection",
      "pretty": "Which features would you like to use?",
      "items": [
        "jdbc",
        "kafka",
        "redis",
        "mySpecialLibrary"
      ],
      "multi": true
    },
    {
      "name": "project.tests",
      "type": "Option",
      "pretty": "Do you want to generate test stubs?"
    }
  ],
  "filters": {
    "conditionalFiles": [
      {
        "condition": "project.features.mySpecialLibrary",
        "matcher": "libs/mySpecialLibrary-*.jar"
      }
    ],
    "includeHidden": [
      ".github/**",
      ".gitignore"
    ],
    "nonTemplates": [
      "**/*.{gradle.kts,jar}"
    ]
  }
}
```

## License and Contributions

Architect is provided under the terms of the BSD 3-Clause License.

Contributions are welcome, but you must have all rights to the contributed material and agree to provide it under the
terms of the aforementioned BSD 3-Clause License.

A good idea would be to check out the [Architect 1.0](https://github.com/v47-io/architect-rs/projects/1) project, it
tracks all the work needed to get Architect to 1.0.
