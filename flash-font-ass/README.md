# flash-font-ass

[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE-MIT)

A command-line tool that orchestrates the `flash-font` ecosystem to automatically load required fonts for ASS (Advanced SubStation Alpha) subtitle files before playback.

When provided with an ASS subtitle file, `flash-font-ass` will:
1. Parse the subtitle using `ass-font` and extract all required fonts.
2. Synchronize your underlying font database using `flash-font`.
3. Query the database to find the physical paths for the required fonts.
4. Temporarily inject those fonts into your Windows session using `flash-font-injector`.

## Installation

```bash
cargo install flash-font-ass --locked
```

## Setup & Configuration

Before you start using `flash-font-ass`, you must initialize the configuration. This creates a config file and specifies the root directory where your fonts are stored.

```bash
flash-font-ass init
```

The application will prompt you to enter the full absolute path to your font directory. This path will be saved along with an automatically determined location for the SQLite database (`fonts.db`).

## Usage

Once configured, you can load fonts for an ASS file before turning on your video player:

```bash
flash-font-ass load --subtitle "path/to/subtitle.ass"
```

The fonts will remain loaded in the Windows system font table as long as the `flash-font-ass` process stays alive, or they will be automatically unloaded when it terminates (subject to the `FontManagerConfig` behavior).

## License

MIT OR Apache-2.0