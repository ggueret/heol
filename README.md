# heol

Solar-synchronized lighting controller for plant grow lights.

Heol ("sun" in Breton) is a daemon that calculates the sun's position and
automatically adjusts your grow lights' brightness and color temperature
throughout the day.

## Features

- Realistic solar color curve (or define your own keyframes)
- Supports mono, dual warm/cold, RGB, and wRGB lights
- GPIO via pigpiod (TCP, any platform) and deCONZ/Zigbee backends
- Multiple zones with mixed light types
- TOML configuration with env var overrides
- Async, lightweight — runs on Raspberry Pi

## Quick Start

```bash
cargo build --release
cp heol.toml.example heol.toml
# Edit heol.toml with your setup
./target/release/heol run
```

## Dry Run

```bash
RUST_LOG=heol=debug heol run --dry-run
```

## Configuration

See `heol.toml.example` for a complete example.

## Development

```bash
make check   # fmt + clippy + tests
make build   # release build
```

## Release

Tag a version to trigger the CI release workflow, which builds binaries for
Linux (x86_64, aarch64, armv7), macOS (x86_64, aarch64) and publishes a
GitHub release with checksums:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## License

MIT
