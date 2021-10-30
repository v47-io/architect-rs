# Rendering

Architect renders files and names using Handlebars and treats all files as potential Handlebars templates.

To find out whether a file actually is a template Architect looks at its contents to look for the "mustaches"
(`{{` and `}}`). Should Architect find both in that order on the same line it will treat the file as a template.

File and directory names are also potential templates that are rendered using Handlebars if they contain the mustaches.

You can also specify to handle certain files as they are and not render them. This is useful when you want to include
actual Handlebars templates in your source repository that should be included in the target as-is. Those files are
identified by their file extension (default: `.hbs`) and will not be rendered, only copied. You can change the extension
and the behavior in the [configuration](../configuration/) file.

<!--@formatter:off-->
```ts
{{#include ../../../../src/schema/architect.ts:79:94}}
```
<!--@formatter:on-->

Should you enable rendering those files as well, they will be rendered like all other templates and Architect will strip
the extension from the file name.
