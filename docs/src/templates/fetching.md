# Fetching

Architect uses Git to fetch remote repositories.

Fetching a remote repository is pretty straightforward, but Architect also supports fetching 
templates from a local directory.

Architect will either fetch it using Git, using the directory like a remote repository, or copy the 
directories contents, e.g. if the directory doesn't contain a Git repository.

When fetching from a local directory you can force Architect to copy the contents instead of using
Git using the `--dirty` [CLI](../cli/README.md) flag. This is particularly useful when you want to 
test a template you are creating without having to commit your changes or pushing it to a remote 
Git repository.
