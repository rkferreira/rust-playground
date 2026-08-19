# Base62 Generator

A tiny, zero‑dependency ready‑to‑use Base62 encoder/decoder for arbitrary binary data.  
It ships both as a **library** (`base62_generator`) and a **CLI** (`base62-cli`).

---

## Features

- Encode any byte slice to a Base62 string, preserving leading zero bytes.
- Decode a Base62 string back to the original bytes.
- Small, fast, and works on stable Rust 2024.
- No external Base62 crate – uses `num-bigint` for arbitrary‑size integers.
- Fully tested (unit + integration) and CI‑validated on GitHub Actions.

---

## Installation

### From source
```bash
# Clone the repository
git clone https://github.com/rkferreira/rust-playground/base62-cli.git
cd base62-cli

# Build the binary
cargo build --release

# The binary will be at target/release/base62-cli
``` 

### As a library
Add the following to your `Cargo.toml`:
```toml
base62_generator = { path = "path/to/base62-cli" }
```
Then import:
```rust
use base62_generator::{encode, decode};
```

---

## Usage (CLI)

The binary provides two sub‑commands: `encode` and `decode`.

```bash
# Encode a hex string (the CLI expects raw text, not a 0x prefix)
base62-cli encode deadbeef
# → prints the Base62 representation

# Decode back to hex (the CLI prints the original hex string)
base62-cli decode <base62-output>
```

**Note:** The CLI works with raw binary data supplied as a hex string.  
All outputs are printed to `stdout` without trailing new‑lines.

---

## Library API

```rust
/// Encode arbitrary binary data to Base62.
/// Leading zero bytes are encoded as the character `'0'`.
pub fn encode(data: &[u8]) -> String;

/// Decode a Base62 string back to a `Vec<u8>`.
/// Returns `anyhow::Error` for invalid characters.
pub fn decode(s: &str) -> anyhow::Result<Vec<u8>>;
```

### Example
```rust
use base62_generator::{encode, decode};

let bytes = b"hello world";
let enc = encode(bytes);
println!("Encoded: {}", enc);
let dec = decode(&enc).expect("valid encoding");
assert_eq!(dec, bytes);
```

---

## Leading‑zero handling

When the input starts with one or more `0x00` bytes, they are represented by the character `'0'` **once per zero byte**.  
During decoding, those `'0'` characters are translated back to zero bytes, ensuring a perfect round‑trip:

```rust
let data = [0, 0, 1, 2];
let enc = encode(&data); // → "00..." (two leading '0's)
let dec = decode(&enc).unwrap();
assert_eq!(dec, data);
```

---

## Testing & CI

Run the full test suite locally:
```bash
cargo test --all
```
The repository includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that automatically runs the same tests on every push and pull‑request.

---

## Releasing

The project follows a conventional‑commit style versioning.  
To publish a new version:
1. Update `Cargo.toml` version.
2. Tag the commit (`git tag vX.Y.Z`).
3. Push tags (`git push --tags`).
4. GitHub Actions can be extended to publish to crates.io.

---

## License

MIT – see the `LICENSE` file for details.
