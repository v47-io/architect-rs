# Rendering

Architect renders files and names using Handlebars and treats all files as potential Handlebars templates.

To find out whether a file actually is a template Architect looks at its contents to look for the "mustaches"
(`{{` and `}}`). Should Architect find both in that order on the same line it will treat the file as a template.

File and directory names are also potential templates that are rendered using Handlebars if they contain the mustaches.
