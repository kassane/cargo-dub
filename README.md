# cargo-dub

A Rust-based wrapper for the D language package manager (DUB)

## Requirements
- Rust 1.74 or higher
- DUB 1.30.0 or higher
- D compiler (dmd, gdc, ldc)

## Installation

```console
cargo install cargo-dub
```

## Features

- Build and run D packages (`cargo dub run`)
- Build-only mode (`cargo dub build`) 
- Format conversion between dub.json and dub.sdl (`cargo dub convert`)
- Pass-through mode for raw DUB commands (`cargo dub raw`)
- Package dependency management (`cargo dub add`, `cargo dub remove`)
- Package initialization (`cargo dub init`)
- Build cache management (`cargo dub clean`)
- D-Scanner linting integration (`cargo dub lint`)
- Build description generation (`cargo dub describe`)
- Package fetching (`cargo dub fetch`)

## Helper

```bash
Usage: cargo-dub [COMMAND]

Commands:
  run       Build and run package
  build     Build package
  convert   Convert dub.json/dub.sdl
  raw       Pass raw arguments to dub
  describe  Print JSON build description for package and dependencies
  add       Add packages as dependencies
  remove    Remove packages from dependencies
  fetch     Fetch packages to a shared location
  init      Initialize an empty package
  clean     Remove cached build files
  lint      Run D-Scanner linter tests
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
