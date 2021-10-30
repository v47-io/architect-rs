# Structure

Architect treats the entire directory and all included files in a Git repository as templates.

This means that the original directory structure is preserved, and you can create additional nested
directory structures when using Handlebars expressions.

Because all directory and file names are treated as potential Handlebars templates themselves, you
can create nested directory structures, e.g. for Java packages.

__Example__:

Provided you have a directory in your template `src/main/java/{{ package javaPackage }}` there are few 
things to consider.

Let's consider `javaPackage` contains the value `com.github.example`. The `package` helper (described 
[here](rendering/helpers.md)) will transform that value to `com/github/example`. The resulting value 
will then be inserted into the path which will give us the final path `src/main/java/com/github/example`.

Architect will then create that path's nested directories copy all files from the source directory into 
this new, dynamically created directory.

## [.architect.json](configuration/)

The template directory can also contain a `.architect.json` file which can specify various configuration
values influencing template rendering.
