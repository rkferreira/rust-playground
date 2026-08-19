## 1. Setup

- [x] 1.1 Create new Cargo library crate `base62_generator` and binary target `base62-cli`
- [x] 1.2 Add dependencies `base62 = "0.2"`, `clap = "4"`, `anyhow = "1"` to Cargo.toml

## 2. Core Implementation

- [x] 2.1 Implement `encode(data: &[u8]) -> String` using `base62::encode`
- [x] 2.2 Implement `decode(s: &str) -> Result<Vec<u8>, anyhow::Error>` with validation of alphabet
- [x] 2.3 Add unit tests for encode/decode covering normal data, empty input, and large inputs

## 3. CLI Integration

- [x] 3.1 Wire up `clap` subcommands `encode <hex>` and `decode <base62>` in `src/main.rs`
- [x] 3.2 Parse hex input to bytes for encoding; output Base62 string to stdout
- [x] 3.3 Parse Base62 input for decoding; output hex representation to stdout
- [x] 3.4 Add error handling to report invalid inputs via `anyhow`

## 4. Testing & CI

- [ ] 4.1 Write integration test invoking the compiled binary with sample inputs and verifying stdout
- [ ] 4.2 Add GitHub Actions workflow to run `cargo test --all` on push and pull request

## 5. Release Preparation

- [ ] 5.1 Build release binary with `cargo build --release`
- [ ] 5.2 Create a GitHub release draft and attach the binary
- [ ] 5.3 Update README with usage examples and installation instructions
