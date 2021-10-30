# Helpers

Architect provides a suite of Handlebars helpers to make writing your templates a breeze.

By default, Architect provides all helpers supported by the [handlebars](https://crates.io/crates/handlebars)
and the [handlebars_misc_helpers](https://crates.io/crates/handlebars_misc_helpers) libraries.

Rhai scripting in templates (as provided by the handlebars library) is not supported!

Please take a look at the [handlebarsjs Language Guide](https://handlebarsjs.com/guide/) for guidance on how to actually
use helpers.

## package

In addition to all those helpers Architect provides a helper off its own: `package`. This helper is intended to help you
write better templated directory names.

You can use this helper to create nested directory structures for your files, its use is to create multiple, nested
directories from dot-separated values, e.g. a Java package.

This helper is not available in template files, only file or directory names.

__Example__:

- File path: `src/main/java/{{ package javaPackage }}/Main.java`
- Context: `"javaPackage": "com.github.example"`
- Result: `src/main/java/com/github/example/Main.java`
