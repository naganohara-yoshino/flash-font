# flash-font Workspace

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

A workspace containing a suite of Rust crates for caching, managing, and flexibly loading system fonts—especially tailored for use cases involving subtitles (ASS) on Windows systems. The core functionality centers around high-performance font scanning, caching metadata via SQLite, and temporarily injecting fonts into the system runtime.

## Crates

This workspace is composed of the following crates:

- **[flash-font](flash-font/README.md)**: The core library providing high-performance font metadata caching using a SQLite database. It quickly scans font directories and allows for fast retrieval of font file paths by their family names.
- **[flash-font-injector](flash-font-injector/README.md)**: A low-level library that provides an API for temporarily injecting/loading fonts into the system for the current session or process.
- **[ass-font](ass-font/README.md)**: A utility library dedicated to parsing ASS (Advanced SubStation Alpha) subtitle files and extracting the names of the fonts they require.
- **[flash-font-ass](flash-font-ass/README.md)**: A command-line interface (CLI) tool that orchestrates the above libraries to automatically parse an ASS subtitle file and load its required fonts temporarily.

## Requirements

- **Rust**: Latest stable recommended.
- **OS**: Primarily targeting Windows (utilizes Windows APIs for font injection).

## Building

To build the entire workspace, you can use Cargo from the root directory:

```bash
cargo build --release
```

## License

This workspace is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-APACHE)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
