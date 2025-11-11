# AUR Packages

This directory contains PKGBUILD files for publishing term39 to the Arch User Repository (AUR).

## Available Packages

### term39 (source package)
- **PKGBUILD**: Builds from source (supports all architectures)
- **Package name**: `term39`
- Users with any architecture can build this

### term39-bin (binary package)
- **PKGBUILD-bin**: Uses pre-built binaries from GitHub releases
- **Package name**: `term39-bin`
- Faster installation (no compilation needed)
- Supports: x86_64, aarch64

## Publishing to AUR

### First Time Setup

1. **Create AUR account** at https://aur.archlinux.org
2. **Add SSH key** to your AUR account
3. **Clone the AUR repository**:
   ```bash
   # For source package
   git clone ssh://aur@aur.archlinux.org/term39.git aur-term39

   # For binary package
   git clone ssh://aur@aur.archlinux.org/term39-bin.git aur-term39-bin
   ```

### Publishing a New Version

#### For term39 (source package):

```bash
cd aur-term39

# Copy PKGBUILD
cp ../term39/aur/PKGBUILD .

# Update pkgver in PKGBUILD
vim PKGBUILD  # Change pkgver to new version

# Generate source tarball checksum
curl -sL https://github.com/alejandroqh/term39/archive/v0.5.0.tar.gz | sha256sum

# Update sha256sums in PKGBUILD with the checksum

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Test build locally
makepkg -si

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Update to version 0.5.0"
git push
```

#### For term39-bin (binary package):

```bash
cd aur-term39-bin

# Copy PKGBUILD-bin as PKGBUILD
cp ../term39/aur/PKGBUILD-bin PKGBUILD

# Update pkgver in PKGBUILD
vim PKGBUILD  # Change pkgver to new version

# Generate checksums for both architectures
curl -sL https://github.com/alejandroqh/term39/releases/download/v0.5.0/term39-v0.5.0-linux-x86_64.tar.gz | sha256sum
curl -sL https://github.com/alejandroqh/term39/releases/download/v0.5.0/term39-v0.5.0-linux-arm64.tar.gz | sha256sum

# Update sha256sums_x86_64 and sha256sums_aarch64 in PKGBUILD

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Test build locally (will download binary)
makepkg -si

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Update to version 0.5.0"
git push
```

## User Installation

Once published, users can install with:

```bash
# Using an AUR helper (yay, paru, etc.)
yay -S term39        # Source package
yay -S term39-bin    # Binary package

# Manual installation
git clone https://aur.archlinux.org/term39.git
cd term39
makepkg -si
```

## Notes

- **Source package** (`term39`): Builds from source, works on all architectures, takes longer to install
- **Binary package** (`term39-bin`): Fast installation, only for x86_64 and aarch64
- Both packages conflict with each other (user can only install one)
- Always update checksums when publishing new versions
- Test locally with `makepkg -si` before pushing to AUR
