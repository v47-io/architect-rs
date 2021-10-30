# CLI

Architect provides a simple command-line interface that uses sensible default values to get going
as quickly as possible.

To instantiate a template in a child directory of the working directory execute this command:

```shell
architect <PATH-OR-URL>
```

Architect follows the Git way of determining the output directory. If you don't explicitly specify 
an output directory, it will use the name of the source Git repository or the directory name of the
path as the target directory name.

__Example__:

- Git repository: `https://github.com/v47-io/architect-test-template.git`
- Output directory: `./architect-test-template`

Additionally, you can also specify a target directory yourself, e.g. if you don't want to use the name
of the source repository:

```shell
architect <PATH-OR-URL> some-directory
```

Architect behaves the same as Git in this instance, it will create a directory with this name, relative
to the working directory. Of course, you can also specify an absolute path.

By default, Architect will copy the entire Git history of the source repository to the target, or 
initialize the target as a Git repository. To prevent you from accidentally overwriting the template
Architect removes the original Git remotes from the target.

## Options

Architect offers some options to customize the behavior of Architect.

### --b, --branch &lt;branch&gt;

Specify a different remote branch to fetch instead of the default branch of the repository.

## Flags

To customize the behavior of Architect even further you can specify one or more flags as described here.

### -D, --dirty

Use the template source in its current (dirty) state.

This only has an effect if a local path is specified as the source repository. In that case Architect
won't perform a Git clone but will just copy the directory, regardless of the local state.

This is most useful to test a template locally while you are developing it. 

This option has no effect with remote repositories.

### -C, --ignore-checks

Ignore some failed checks that would prevent Architect from creating the target files.

These errors will be ignored:

- Condition evaluation errors (for conditional files)

### -H, --no-history

Don't copy the Git history from the source repository to the target.

Instead, the target will be initialized as a new Git repository.

### -I, --no-init

Don't initialize the target directory as a Git repository.

This requires the `--no-history` flag to be present as well.

### --verbose

Enables verbose output.

This is very technical at places. Make sure to specify this option before reporting a bug.
