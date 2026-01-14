# Releasing Mole-RS

## Manual Release Process

### 1. Update Version
Edit `Cargo.toml`:
```toml
version = "0.1.0"  # Update this
```

### 2. Build Release Binary
```bash
cargo build --release
```

### 3. Create Release Tarball
```bash
VERSION="0.1.0"
PLATFORM="linux-x86_64"  # or linux-aarch64

mkdir -p "mole-rs-${VERSION}-${PLATFORM}"
cp target/release/mo "mole-rs-${VERSION}-${PLATFORM}/"
cp README.md LICENSE "mole-rs-${VERSION}-${PLATFORM}/" 2>/dev/null || true
tar -czvf "mole-rs-${VERSION}-${PLATFORM}.tar.gz" "mole-rs-${VERSION}-${PLATFORM}"
rm -rf "mole-rs-${VERSION}-${PLATFORM}"
```

### 4. Create GitHub Release
1. Go to https://github.com/thirukguru/mole-rs/releases/new
2. Tag: `v0.1.0` (must start with 'v')
3. Title: `Mole-RS v0.1.0`
4. Upload the `.tar.gz` files
5. Publish release

### 5. Verify Installation
```bash
# Test the install script
curl -fsSL https://raw.githubusercontent.com/thirukguru/mole-rs/main/install.sh | bash

# Or build from source
curl -fsSL https://raw.githubusercontent.com/thirukguru/mole-rs/main/install.sh | bash -s -- -s
```

## Quick Release Script
```bash
#!/bin/bash
VERSION="${1:-0.1.0}"
cargo build --release
mkdir -p "mole-rs-${VERSION}-linux-x86_64"
cp target/release/mo "mole-rs-${VERSION}-linux-x86_64/"
tar -czvf "mole-rs-${VERSION}-linux-x86_64.tar.gz" "mole-rs-${VERSION}-linux-x86_64"
rm -rf "mole-rs-${VERSION}-linux-x86_64"
echo "Created: mole-rs-${VERSION}-linux-x86_64.tar.gz"
echo "Upload to: https://github.com/thirukguru/mole-rs/releases/new"
```
