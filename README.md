# CIViewer

CIViewer is a high-performance, modern image viewer built with Rust and GTK4 (via Relm4). Designed for a seamless viewing experience, it supports browsing local image directories as well as images directly inside ZIP archives without decompression.

## Features

- **Fast & Lightweight**: Built with Rust and GTK4 for speed and efficiency.
- **Archive Support**: Direct viewing of images within ZIP files.
- **Reading Modes**:
    - Single page view.
    - Spread view (two pages) ideal for manga/comics.
    - Support for Right-to-Left (RTL) reading direction.
- **Efficient Navigation**: Sidebar file tree for quick directory switching.
- **Customizable**: Settings for shortcuts and view preferences saved automatically (powered by PoloDB).
- **Format Support**: JPEG, PNG, GIF, BMP, WebP.

## Installation & Build

### Requirements
- Rust (latest stable)
- GTK4 development headers

### Build locally
```bash
cargo build --release
```

### Build with Docker
A `Dockerfile` and build script are provided for a consistent build environment.
```bash
./build.sh
```

## Usage
Run the application from the terminal or your application launcher.
```bash
cargo run --release
```

## Development
This project is developed with the assistance of **Gemini 3 Pro**.
