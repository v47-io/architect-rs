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

Architect uses this [Handlebars implementation](https://docs.rs/crate/handlebars/4.1.3).

For more information you can also refer to [handlebarsjs.com](https://handlebarsjs.com/), however, not all features may
be supported.

## [Documentation](https://v47-io.github.io/architect-rs/)

## License and Contributions

Architect is provided under the terms of the BSD 3-Clause License.

Contributions are welcome, but any contributor must have all rights to the contributed material and agree to provide it
under the terms of the aforementioned BSD 3-Clause License.
