# Oxid

Oxid is an experimental `no_std` x86_64 kernel written in Rust.

The kernel has no external Rust crate dependencies. Limine is used as an external bootloader/tooling dependency so the kernel can focus on memory, interrupts, drivers, scheduling, and system architecture instead of owning firmware and disk loading from day one.

## Requirements

- Rust nightly with `rust-src`
- QEMU for x86_64
- `curl`
- Windows: PowerShell for ZIP extraction
- Linux: `unzip`

The first `cargo image` run downloads the pinned Limine binary release into `.tools/`.

### Windows Setup

```powershell
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

Install QEMU and make sure `qemu-system-x86_64` is available in `PATH`.

### Linux Setup

```sh
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
sudo apt install qemu-system-x86 unzip curl
```

Use the equivalent package names for non-Debian distributions.

## Commands

```sh
cargo test
```

Runs host-side unit tests.

```sh
cargo kbuild
```

Builds the bare-metal kernel ELF:

```text
target/x86_64-unknown-none/debug/oxid
```

```sh
cargo ktest-build
```

Builds the bare-metal test binary without trying to execute it on the host.

```sh
cargo image
```

Builds `target/oxid.img`, a raw BIOS-bootable disk image with Limine and the Oxid kernel.

```sh
cargo run-qemu
```

Builds the image and boots it with QEMU.

## Architecture

- `src/boot`: Limine protocol requests and the kernel-facing `BootInfo`.
- `src/arch`: x86_64 CPU and port I/O code.
- `src/drivers`: serial and framebuffer console drivers.
- `src/kernel`: kernel lifecycle and panic handling.
- `src/memory`: boot memory map model.
- `src/text`: fixed-capacity text utilities for `no_std` code.
- `tools/xtask`: host-side build/image/QEMU automation with no external crates.

## Policy

- Kernel code stays `no_std`.
- No external Rust crates are used.
- All project code, comments, and documentation are written in English.
- Bootloader-specific data is converted into Oxid-owned types before entering core kernel code.
