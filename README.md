# Mole-RS 🐹

**Deep clean and optimize your Ubuntu system**

A Rust-based system cleanup tool inspired by [tw93/Mole](https://github.com/tw93/Mole), designed specifically for Ubuntu Linux.

## Features

- **Clean** - Free up disk space by cleaning caches, trash, and temp files
- **Analyze** - Explore disk usage with visual breakdown
- **Status** - Monitor system health in real-time (CPU, memory, disk, network)
- **Purge** - Clean development project artifacts (node_modules, target, venv, etc.)
- **Optimize** - Run system maintenance tasks

## Installation

```bash
# Build from source
cargo build --release

# The binary will be at ./target/release/mo
```

## Usage

### Interactive Mode
```bash
mo
```
Launches the interactive TUI menu.

### CLI Commands
```bash
mo clean              # Deep system cleanup
mo clean --dry-run    # Preview without deleting
mo analyze            # Analyze home directory
mo analyze /path      # Analyze specific path
mo status             # Live system monitor
mo purge              # Clean dev artifacts
mo purge --dry-run    # Preview purge
mo optimize           # System maintenance
mo optimize --dry-run # Preview optimize
```

## Keyboard Controls

### TUI Menu
| Key | Action |
|-----|--------|
| ↑/↓ or j/k | Navigate |
| Enter | Select |
| 1-5 | Quick select |
| q | Quit |

### Status Monitor
| Key | Action |
|-----|--------|
| Ctrl+C | Exit |

## Configuration

Config file: `~/.config/mole-rs/config.toml`

```toml
# Paths to never delete
whitelist = []

# Directories to scan for dev artifacts
project_paths = [
    "~/Projects",
    "~/Development",
]

# Skip files newer than N days
skip_recent_days = 7

# Max journal log size
journal_max_size = "100M"
```

## Requirements

- Ubuntu 20.04, 22.04, or 24.04
- Rust 1.70+ (for building)

## License

MIT
