# Templates

Architect makes it easy to scaffold entire projects from Git repositories, here referred to as templates.

All Git repositories are potential templates, and all files in them are also treated as Handlebars templates if they
contain Handlebars template expressions indicated by `{{` and `}}`. All other files are simply copied as long as they
are not excluded.

On top of that all file names are also potential Handlebars templates that are rendered when generating sources from a
template.

Architect preserves the original directory structure but allows the creation of new nested directory structures by way
of Handlebars expressions in directory names.

## Template Repositories

Instead of just using the entire Git repository as a template, Architect provides the option to maintain multiple
templates in one repository and referring to them by their directory name when generating a project.

When using template repositories, each template must contain an `.architect.json` file to be usable as a template.

Looking at the following example you can then specify the option `--template template-1` to use the specified template
instead of the repository.

Example for a directory structure with multiple templates:

```text
repository-dir
├── template-1
|  └── .architect.json
├── template-2
|  └── .architect.json
└── README.md
```

You can still use the entire repository as a template (by not specifying a template name), but all subdirectories
containing an `.architect.json` file will be ignored.

Although nesting templates is not possible, you can still group multiple templates in directories.

To use one of the grouped templates of the following example you would specify the
option `--template template-group-1/template-1`.

```text
repository-dir
├── template-group-1
|  ├── template-1
|  |  └── .architect.json
|  └── template-2
|     └── .architect.json
├── template-3
|  └── .architect.json
└── README.md
```
