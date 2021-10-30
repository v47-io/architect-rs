# Templates

Architect makes it easy to scaffold entire projects from Git repositories, here referred to as templates.

All Git repositories are potential templates, and all files in them are also treated as Handlebars templates
if they contain Handlebars template expressions indicated by `{{` and `}}`. All other files are simply
copied as long as they are not excluded.

On top of that all file names are also potential Handlebars templates that are rendered when generating
sources from a template.

Architect preserves the original directory structure but allows the creation of new nested directory structures
by way of Handlebars expressions in directory names.
