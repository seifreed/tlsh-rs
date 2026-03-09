<p align="center">
  <img src="https://img.shields.io/badge/TLSH--RS-Fuzzy%20Hashing-blue?style=for-the-badge" alt="TLSH-RS">
</p>

<h1 align="center">tlsh-rs</h1>

<p align="center">
  <strong>Pure Rust TLSH for crates, services, malware pipelines, and similarity analysis</strong>
</p>

<p align="center">
  <a href="https://github.com/seifreed/tlsh-rs/releases"><img src="https://img.shields.io/github/v/release/seifreed/tlsh-rs?style=flat-square&logo=github" alt="GitHub Release"></a>
  <a href="https://crates.io/crates/tlsh-rs"><img src="https://img.shields.io/crates/v/tlsh-rs?style=flat-square&logo=rust" alt="Crates.io Version"></a>
  <a href="https://github.com/seifreed/tlsh-rs/blob/main/Cargo.toml"><img src="https://img.shields.io/badge/rust-edition%202024-orange?style=flat-square&logo=rust" alt="Rust Edition"></a>
  <a href="https://github.com/seifreed/tlsh-rs/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/seifreed/tlsh-rs/ci.yml?style=flat-square&logo=github&label=CI" alt="CI Status"></a>
  <a href="https://github.com/seifreed/tlsh-rs/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/seifreed/tlsh-rs/release.yml?style=flat-square&logo=github&label=Release" alt="Release Status"></a>
  <img src="https://img.shields.io/badge/coverage-99.88%25-brightgreen?style=flat-square" alt="Coverage">
</p>

<p align="center">
  <a href="https://github.com/seifreed/tlsh-rs/stargazers"><img src="https://img.shields.io/github/stars/seifreed/tlsh-rs?style=flat-square" alt="GitHub Stars"></a>
  <a href="https://github.com/seifreed/tlsh-rs/issues"><img src="https://img.shields.io/github/issues/seifreed/tlsh-rs?style=flat-square" alt="GitHub Issues"></a>
  <a href="https://buymeacoffee.com/seifreed"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-support-yellow?style=flat-square&logo=buy-me-a-coffee&logoColor=white" alt="Buy Me a Coffee"></a>
</p>

---

## Overview

**tlsh-rs** is a pure Rust implementation of **TLSH** (Trend Locality Sensitive Hash), built for projects that need fuzzy hashing as a native crate instead of binding to a C or C++ implementation.

This port follows the upstream TLSH algorithm closely, keeps the implementation in safe and portable Rust, and exposes both a library API and a CLI.

### Key Features

| Feature | Description |
|---------|-------------|
| **Pure Rust** | No external crates required for the hashing core |
| **Edition 2024** | Modern Rust crate layout and tooling |
| **Multiple TLSH Profiles** | Supports `128/1`, `128/3`, `256/1`, and `256/3` |
| **Streaming Builder** | Incremental hashing with `TlshBuilder` |
| **Digest Parsing** | Handles raw digests and `T1`-prefixed digests |
| **Distance Calculation** | Compare hashes with and without length penalty |
| **CLI Included** | `hash`, `hash-many`, `diff`, and `xref` commands |
| **JSON + SARIF** | Machine-readable outputs for automation pipelines |
| **Real Test Vectors** | Validated against upstream-compatible reference vectors and local fixtures |

### Supported Targets in CI/CD

```
Windows   x64, ARM64
Linux     x64, ARM64
macOS     Intel, ARM64
```

---

## Installation

### From crates.io

```bash
cargo install tlsh-rs --bin tlsh
```

### From Source

```bash
git clone https://github.com/seifreed/tlsh-rs.git
cd tlsh-rs
cargo build --release
```

### Build the CLI

```bash
cargo build --release --bin tlsh
```

---

## Quick Start

```bash
# Hash a file
cargo run --bin tlsh -- hash ./fixtures/small.txt

# Compare two files
cargo run --bin tlsh -- diff ./fixtures/small.txt ./fixtures/small2.txt

# Produce SARIF
cargo run --bin tlsh -- diff --format sarif ./a.bin ./b.bin
```

---

## Usage

### Command Line Interface

```bash
# Standard TLSH hash
tlsh hash sample.bin

# Raw digest output
tlsh hash --raw sample.bin

# JSON output
tlsh hash --format json sample.bin

# Hash several files
tlsh hash-many file1.bin file2.bin file3.bin

# Compare files or digests
tlsh diff left.bin right.bin
tlsh diff T1... T1...

# Cross-reference several inputs
tlsh xref a.bin b.bin c.bin

# Read one input from stdin
cat sample.bin | tlsh hash -
```

### Available Commands

| Command | Description |
|--------|-------------|
| `hash` | Hash one file or `stdin` |
| `hash-many` | Hash multiple files |
| `diff` | Compare two inputs and return TLSH distance |
| `xref` | Compare every pair in a set of inputs |

### Available Options

| Option | Description |
|--------|-------------|
| `--profile` | Select `128-1`, `128-3`, `256-1`, or `256-3` |
| `--raw` | Return raw hex digest instead of `T1`-prefixed output |
| `--format` | Output as `text`, `json`, or `sarif` |
| `--no-length` | Exclude length penalty from TLSH distance |
| `--threshold N` | Filter `xref` results above `N` |
| `-` | Read one binary input from `stdin` |

---

## Rust Library

### Basic Usage

```rust
use tlsh_rs::{hash_bytes, TlshBuilder};

let digest = hash_bytes(b"example payload")?;

let mut builder = TlshBuilder::new();
builder.update(b"example ")?;
builder.update(b"payload")?;
let streamed = builder.finalize()?;

assert_eq!(digest, streamed);
```

### Profile-Specific Hashing

```rust
use tlsh_rs::{hash_bytes_with_profile, TlshProfile};

let digest = hash_bytes_with_profile(
    b"example payload",
    TlshProfile::full_256_3(),
)?;

println!("{}", digest.raw_hex());
```

### Parse and Compare Digests

```rust
use tlsh_rs::TlshDigest;

let left: TlshDigest = "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC".parse()?;
let right: TlshDigest = "T1C6A022A2E0008CC320C083A3E20AA888022A00000A0AB0088828022A0008A00022F22A".parse()?;

let diff = left.diff(&right);
let diff_no_length = left.diff_no_length(&right);
```

---

## Examples

### Hash with a specific profile

```bash
tlsh hash --profile 256-3 sample.bin
```

### Export `xref` as JSON

```bash
tlsh xref --format json a.bin b.bin c.bin
```

### Export `diff` as SARIF

```bash
tlsh diff --format sarif a.bin b.bin
```

### Use digest strings directly

```bash
tlsh diff T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC \
          T1C6A022A2E0008CC320C083A3E20AA888022A00000A0AB0088828022A0008A00022F22A
```

---

## Verification

- Validated against upstream-compatible reference vectors and local fixtures in [`./fixtures`](./fixtures)
- Real tests without mocks
- Current measured coverage:
  - lines: `99.88%`
  - functions: `100.00%`
  - regions: `99.18%`

---

## Upstream Credit

This project exists because the original **TLSH** project by **Trend Micro** exists first:

- Upstream repository: https://github.com/trendmicro/tlsh

Huge thanks to the Trend Micro team for publishing and maintaining TLSH. The original project made the algorithm accessible to the community and gave us a solid reference to validate against.

This Rust port was built so we could use TLSH naturally as a Rust crate inside our own projects, keep deployment simple, and integrate fuzzy hashing into malware analysis and similarity pipelines without depending on a C/C++ runtime.

To the upstream maintainers: your work is genuinely useful, technically elegant, and still doing real damage against boring malware workflows in the best possible way. Thanks for building it and sharing it.

---

## Requirements

- Rust stable
- Cargo
- Rust edition `2024`

---

## CI/CD

GitHub Actions builds release artifacts for:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `aarch64-pc-windows-msvc`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

Workflows:

- [`.github/workflows/ci.yml`](./.github/workflows/ci.yml) for push/PR validation
- [`.github/workflows/release.yml`](./.github/workflows/release.yml) for tag-driven GitHub Releases and crates.io publishing

Release flow:

1. Push a tag like `v0.1.0`
2. GitHub Actions runs verification
3. Native release binaries are built for all supported targets
4. A GitHub Release is created or updated with packaged artifacts
5. The crate is published to crates.io with `cargo publish --locked`

To publish to crates.io, configure this repository secret:

```text
CARGO_REGISTRY_TOKEN
```

That token must be created from your crates.io account with publish permissions for `tlsh-rs`.

The runner labels used are based on the current GitHub-hosted runner reference, including `ubuntu-24.04-arm`, `windows-11-arm`, `macos-15-intel`, and `macos-latest` from GitHub Docs:

- https://docs.github.com/actions/reference/runners/github-hosted-runners

This is an implementation detail inferred from the current GitHub-hosted runner catalog and may need adjustment if GitHub renames runner labels later.

---

## Contributing

Contributions are welcome. If you want to improve algorithm coverage, add more upstream compatibility vectors, or refine the CLI and release flow, open an issue or a pull request.

1. Fork the repository
2. Create your branch (`git checkout -b feature/amazing-change`)
3. Commit your changes (`git commit -m 'Add amazing change'`)
4. Push your branch (`git push origin feature/amazing-change`)
5. Open a Pull Request

---

## Support the Project

If this crate is useful in your malware analysis, similarity matching, or triage pipelines, you can support the work here:

<a href="https://buymeacoffee.com/seifreed" target="_blank">
  <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" height="50">
</a>

---

## License

This crate is published under:

```text
Apache-2.0 OR BSD-3-Clause
```

That matches the licensing model exposed by the upstream TLSH project.

---

<p align="center">
  <sub>Made for practical fuzzy hashing workflows in Rust</sub>
</p>
