# Context

Architect builds a Handlebars context using some provided information and the values defined 
when answering configured questions.

This context is available to all instances where Handlebars templates are processed, so you 
can access specified values in conditions, or file/directory names.

Context consists of multiple nested objects and properties which contain values.

## Default Context

By default, Architect provides the following data in the context, provided values are configured:

```json
{
  "__template__": {
    "name": "The template name from in .architect.json or undefined",
    "version": "The template version from .architect.json or undefined"
  }
}
```

You cannot add to the `__template__` object using your questions, and Architect will reject any
attempt to do so. However, you can still define a `__template__` object in any nested object 
further down the line.

Something like this would be possible:

```json
{
  "__template__": ...,
  "yourOwnObject": {
    "__template__": {
      "yourProperty": "some data"
    }
  }
}
```

## File Context

In addition to the default context Architect adds some information about the current template file 
to the context:

```json
{
  "__template__": {
    ...,
    "file": {
      "rootDir": "the working directory, not the target directory",
      "sourceName": "the original file name of the template file",
      "sourcePath": "the original path of the template file",
      "targetName": "the final file name of the rendered template file",
      "targetPath": "the final path of the rendered template file"
    }
  }
}
```
