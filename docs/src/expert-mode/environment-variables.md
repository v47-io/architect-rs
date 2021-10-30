# Environment Variables

Architect offers you the ability to modify some behaviors through the use of environment variables.

## RENDER_PARALLELISM

By default, Architect uses multiple threads to render the templates.

This is the formula: `max(1, min(4, number_of_threads / 2))`

`number_of_threads` is the number of threads available on your machine.

If you think four threads are too little or too much, you can change this to any other integer &gt; 0.

## TEMPLATE_INSPECT_MAX_LINES

Architect won't read entire files to find out if they are templates. By default, it'll only read 25 lines of each file
to find out.

If you have files where the Handlebars "mustaches" only start occurring after the 25th line, this environment variable
is for you. Just set it to any other integer &gt; 0.

## LINE_BUFFER_CAPACITY

Architect uses a buffer to read lines into memory one at a time. By default, this buffer is 256 bytes long.

Because the buffer expands automatically this isn't even something I'd recommend adjusting, but if you really want to,
set it to any other integer &gt; 0.
