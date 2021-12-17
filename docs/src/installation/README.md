# Installation

## Prerequisites

Architect uses your existing Git installation, so you should have the `git` command on your `PATH`.

## Using pre-built binaries

You can easily get pre-built binaries for Architect if you don't want to build it yourself.

Simply navigate to the [Releases](https://github.com/v47-io/architect-rs/releases) page on GitHub and download 
the latest binary for your platform. 

Currently, the supported platforms are:
  
  - Linux (x86_64, libc)
  - Windows (x86_64)
  - macOS (darwin)

After downloading the executable file simply place it or a link to it in your `PATH` and you're ready to go.

Don't forget to mark the file as executable if you're on Linux or macOS:

```shell
chmod +x architect
```

## Building from source

If you're in the mood for some action, or you want to try the latest features before they appear in a release
you can get the source from GitHub and build Architect yourself.

Fork it or get the source from the [repository](https://github.com/v47-io/architect-rs), and you're ready to go.

Make sure you've got at least Rust 2021 installed and the `cargo` command is available.

### Building

```shell
cargo build --bin architect
```

This command should create an executable file in the `target/debug` directory, ready for use.

To create an optimized build add the `--release` flag to the command:

```shell
cargo build --release --bin architect
```

The executable file should then be located in the `target/release` directory.

### Testing

```shell
cargo test --bin architect
```
