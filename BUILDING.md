# Building from Source

## Prerequisites

1. Install [Rust](https://rustup.rs/) (latest stable version)
2. Install Vulkan development packages:

   ```bash
   # Ubuntu/Debian
   sudo apt install vulkan-tools libvulkan-dev

   # Fedora
   sudo dnf install vulkan-tools vulkan-loader-devel

   # Arch Linux
   sudo pacman -S vulkan-devel
   ```

## Building

Clone the repository and build with cargo:

```bash
git clone https://github.com/bvpav/raydar.git
cd raydar
cargo build --release  # Use --release for significantly faster CPU rendering
```

This will create two binaries in `target/release/`:

- `raydar` - Headless renderer
- `raydar_editor` - Scene editor

## Development Build

For development, you can build and run without `--release`:

```bash
cargo run -- [FLAGS] [scene-file]            # Run headless renderer
cargo run --bin raydar_editor -- [FLAGS]     # Run editor
```

Note that the CPU renderer will be significantly slower in development builds. Always use `--release` for production rendering:

```bash
cargo run --release -- [FLAGS] [scene-file]  # Much faster CPU rendering
```

## Troubleshooting

### Vulkan Issues

If you encounter Vulkan-related errors:

1. Make sure you have a Vulkan-capable GPU
2. Verify Vulkan is properly installed: `vulkaninfo`
3. Use the `--cpu` flag to fall back to CPU rendering
