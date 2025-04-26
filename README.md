# Oxide Mod Manager

A simple GUI tool built with [Slint](https://slint.dev/) and Rust to help uncompress game archives (RAR, ZIP, 7z) and move their contents into a target game directory.

![Screenshot](assets/program.png)

## Features

*   Select a source directory containing compressed game mod files (`.rar`, `.zip`, `.7z`).
*   Select a target game directory.
*   Uncompresses archives found in the source directory into a temporary location (`.temp/`).
*   Moves the uncompressed contents from the temporary directory into the target game directory.
*   Options:
    *   Overwrite existing files in the target directory.
    *   Extract directly to the game directory (faster, less disk usage, but prevents creating backups).
*   Cross-platform GUI built with Slint.
*   Safety first: Usage of symlinks instead of copying over extracted files.

## Getting Started

### Installation

**Recommended:** Download the latest pre-built binary for your operating system from the [**GitHub Releases**](https://github.com/pezfisk/OxideManager/releases) page.

<!-- TODO: Create releases with binaries for Windows, macOS, Linux if possible -->

**Alternatively:** Build from source (see below).

## Building from Source

### Prerequisites

1.  **Rust Toolchain:** Install Rust and Cargo from [rustup.rs](https://rustup.rs/).
2.  **Slint Dependencies:** Slint requires a C++ compiler and CMake. Follow the platform-specific instructions on the [Slint Setup Documentation](https://slint.dev/docs/get-started/setup). Common requirements:
    *   **Linux:** `build-essential`, `cmake`, `pkg-config`, `libfontconfig-dev`, `libfreetype6-dev`, `libxkbcommon-dev` (or equivalents for your distro).
    *   **Windows:** Visual Studio Build Tools (with C++ workload).
    *   **macOS:** Xcode Command Line Tools, CMake.
3.  **Git:** To clone the repository.

### Steps

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/pezfisk/OxideManager.git
    cd OxideManager
    ```

2.  **Build (Native):**
    ```bash
    cargo build --release
    ```
    The executable will be in `target/release/`.

### Cross-Compilation (e.g., Linux -> Windows)

Cross-compiling is borked.

### Slint Backend and Style Configuration

You can configure the [Slint backend](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/style/) (e.g., `winit`, `qt`, `gtk`) and style (`fluent`, `material`, `cupertino`, `cosmic`) at compile time:

1.  **Environment Variables (Temporary):**
    ```bash
    export SLINT_BACKEND=winit
    export SLINT_STYLE=fluent-dark
    cargo build --release
    ```
2.  **Cargo Configuration (Persistent):** Create/edit `.cargo/config.toml` in the project root:
    ```toml
    # .cargo/config.toml
    [env]
    SLINT_BACKEND = "winit"
    SLINT_STYLE = "fluent-dark"
    ```
    Then run `cargo clean && cargo build --release`.

## Contributions are welcome!

