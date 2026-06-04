# Contributing to Oxid

Oxid is intentionally small, explicit, and dependency-light. Contributions should keep the kernel easy to reason about on both Windows and Linux.

## Requirements

- Rust nightly
- `rust-src`
- QEMU for x86_64
- `curl`
- Windows: PowerShell
- Linux: `unzip`

## Setup

Windows:

```powershell
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

Install QEMU and ensure `qemu-system-x86_64` is available in `PATH`.

Linux:

```sh
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
sudo apt install qemu-system-x86 unzip curl
```

Use equivalent package names for non-Debian distributions.

## Validation

Run the full local validation before committing:

```sh
cargo check-all
cargo smoke-qemu
```

`cargo check-all` runs host tests, xtask tests, bare-metal kernel build, bare-metal test build, and image generation.

`cargo smoke-qemu` boots the generated image headlessly and verifies that serial output contains `Oxid kernel initialized`.

## Commit Policy

- Prefer small logical commits.
- Each commit should leave `cargo check-all` passing.
- Run `cargo smoke-qemu` for changes touching boot, linker, image generation, console, serial, or kernel entry.
- Use concise conventional-style messages such as `build: add smoke qemu task` or `kernel: add exception handlers`.

## Code Policy

- Kernel code stays `no_std`.
- Do not add external Rust crates without an explicit design discussion.
- Keep project code, comments, docs, commit messages, and user-facing text in English.
- Convert bootloader-specific data into Oxid-owned types before passing it into core kernel code.
- Keep platform-specific host tooling inside `tools/xtask` or scripts, not inside kernel modules.
