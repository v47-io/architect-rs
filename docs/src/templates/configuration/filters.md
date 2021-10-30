# Filters

The Architect configuration file allows you to define filters in multiple ways to control whether certain files get
actually added to the target.

The filters are:

- `exclude` (Exclusion of files, strongest)
- `includeHidden` (Inclusion of certain hidden files)
- `conditionalFiles` (Inclusion of files if a certain condition is true)

These filters have a precedence assigned to them, exclusions are the strongest. Files that match an exclusion rule are
never added to the target. Hidden files can only be added through a condition if they have also been matched by an
explicit `includeHidden` expression. If you include a hidden file, but a condition that matches it is false, it's not
added to the target.

Keep in mind that the top-level `.git` directory cannot be matched by any of those filters and is always excluded from
explicit processing. Architect handles that `.git` directory separately.

All filters use _glob_ expressions that match the entire relative path to the root directory of the template.

Please see [this](https://docs.rs/globset/0.4.8/globset/#syntax) for more information on the supported syntax for _glob_
expressions (Architect enables the `literal_separator` option and case-insensitive matching).

## Exclusions

Define _glob_ expressions to match files you don't want to have in the target.

## Include Hidden

Architect by default excludes all hidden files or directories. This applies to both file/directory names starting with a
dot or files explicitly hidden in file system (Windows).

Define _glob_ expressions to match hidden files you want to include anyway.

Please keep in mind that you cannot include the top-level `.git` directory or any of its descendants using this.

## Conditional Files

Architect will only include matched files if the specified condition returns a "truthy" result. Here, ordering matters,
as Architect will only evaluate the condition for the first match.

Here you define _glob_ expressions to match files, and Handlebars expressions to determine whether these files should be
included. These Handlebars expression don't need to be delimited by `{{` and `}}`, and have full access to the context.

The _glob_ expression will be applied to the file in the source repository, so before any possible Handlebars templates
in file or directory names are evaluated.

Please keep in mind that when working with hidden files conditions can only be applied to files included
using `includeHidden`.

Format in the configuration file:

<!--@formatter:off-->
```ts
// Config
{{#include ../../../../src/schema/architect.ts:66}}

// ConditionalFiles
{{#include ../../../../src/schema/architect.ts:120:134}}
```
<!--@formatter:on-->
