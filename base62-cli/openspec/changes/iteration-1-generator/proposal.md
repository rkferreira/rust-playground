## Why

We need a fast, reliable way to represent binary hashes as compact, URL‑safe strings. Base62 encoding provides a space‑efficient, alphanumeric representation without padding, making it ideal for identifiers, filenames, and short URLs. A Rust implementation can be used both as a command‑line utility for ad‑hoc encoding/decoding and as a library for other Rust projects.

## What Changes

- Add a **single binary** that bundles both CLI and library functionality.
- Provide **encode** and **decode** commands.
- Expose a **library API** (`base62_hash`) for other crates.
- Include unit tests, CI pipeline, and a release workflow.

## Capabilities

### New Capabilities
- `base62-generator`: Implements Base62 encoding/decoding of arbitrary binary data and exposes a library crate.

### Modified Capabilities
- *(none)*

## Impact

- Adds a new binary target to the repository (`src/main.rs`).
- Introduces a new library crate under `src/lib.rs`.
- Adds `clap` for argument parsing, `base62` for encoding, and `anyhow` for error handling as dependencies.
- Updates CI configuration to run tests and build the binary for release.
