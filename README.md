# cargo-dub

A Rust-based wrapper for the D language package manager (DUB)

## Requirements
- Rust 1.83 or higher
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

## Helper

```bash
A cargo subcommand for dub

Usage: cargo-dub [COMMAND]

Commands:
  run      Build and run the package (default)
  build    Only build the package
  convert  Convert between dub.json and dub.sdl
  raw      Pass-through to dub with raw arguments
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
