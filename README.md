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

## Usage
```bash
./civiewer <path>
```

## Build

### Requirements
- Docker

### Build with Docker
A `Dockerfile` and build script are provided for a consistent build environment.
```bash
cd build
./build.sh
```

## Development
This project is developed with the assistance of **Gemini 3 Pro**.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2026 arianpg.
This software is provided "as is", without warranty of any kind.

### Third-Party Licenses
This application includes open-source software.
- **GTK4, GLib, Pango, Cairo, GdkPixbuf**: LGPL v2.1 or later.
- **MinGW-w64 Runtime**: GPL v3 with GCC Runtime Library Exception.
- **Rust Crates**: Mostly MIT or Apache-2.0.

See `ThirdPartyNotices.txt` included in the distribution for full license details.
