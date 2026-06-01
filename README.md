# heol

Solar-synchronized lighting controller daemon.

Heol ("sun" in Breton) is a daemon that calculates the sun's position and
automatically adjusts your lights' brightness and color temperature throughout
the day, following natural circadian rhythms.

Use it for grow lights, circadian lighting in poorly sunlit spaces, jet lag
recovery, or anywhere you want light that follows the sun.

## Features

- Realistic solar color curve (or define your own keyframes)
- Supports single, CCT (warm/cold), RGB, and wRGB lights
- GPIO via pigpiod (TCP, any platform) and deCONZ/Zigbee backends
- Multiple zones with mixed light types
- TOML configuration with env var overrides for secrets
- Async, lightweight — runs on Raspberry Pi

## Requirements

- **Rust 1.85+** (to build from source)
- **pigpiod** running on the target host (for GPIO backend)
- **deCONZ gateway** reachable on the network (for Zigbee backend)

## Install

### Pre-built binaries

Download the latest release for your platform from the
[Releases](https://github.com/ggueret/heol/releases) page.

Available targets: x86_64, aarch64, armv7, armv6 (Linux), x86_64 and aarch64 (macOS).

```bash
tar xzf heol-*.tar.gz
chmod +x heol
```

### cargo install

```bash
cargo install heol
```

### Docker

```bash
docker run -v ./heol.toml:/heol.toml ghcr.io/ggueret/heol
```

### Build from source

```bash
cargo build --release
```

## Quick Start

```bash
cp heol.toml.example heol.toml
# Edit heol.toml with your location, backends, zones and lights
./heol run
```

## Dry Run

Log commands without sending them to backends:

```bash
RUST_LOG=heol=debug ./heol run --dry-run
```

## Configuration

See [`heol.toml.example`](heol.toml.example) for a complete example.

### Environment variable overrides

Backend connection settings can be overridden via environment variables,
which is the recommended way to manage secrets like API keys:

```bash
HEOL_DECONZ_HOME_API_KEY=your_secret_key ./heol run
```

The naming pattern is `HEOL_<BACKEND_TYPE>_<PROFILE>_<FIELD>`:

| Variable | Overrides |
|----------|-----------|
| `HEOL_GPIO_LOCAL_HOST` | `[backends.gpio.local] host` |
| `HEOL_GPIO_LOCAL_PORT` | `[backends.gpio.local] port` |
| `HEOL_DECONZ_HOME_HOST` | `[backends.deconz.home] host` |
| `HEOL_DECONZ_HOME_PORT` | `[backends.deconz.home] port` |
| `HEOL_DECONZ_HOME_API_KEY` | `[backends.deconz.home] api_key` |

### Signals

| Signal | Effect |
|--------|--------|
| `SIGTERM` / `SIGINT` | Graceful shutdown |
| `SIGHUP` | Force immediate recalculation |

## Development

```bash
make check   # fmt + clippy + tests
make build   # release build
```

## License

MIT
