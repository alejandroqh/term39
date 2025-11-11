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

## Installation

TERM39 is published on the Arch User Repository (AUR) in two packages:

- **[term39](https://aur.archlinux.org/packages/term39)** - Source package (builds from source using cargo)
- **[term39-bin](https://aur.archlinux.org/packages/term39-bin)** - Binary package (pre-built binaries, faster installation)

### Using an AUR helper (recommended)

```bash
# Binary package (recommended - faster installation)
yay -S term39-bin

# Source package (builds from source)
yay -S term39
```

You can use any AUR helper like `yay`, `paru`, `pamac`, etc.

### Manual installation from AUR

```bash
# Binary package (recommended)
git clone https://aur.archlinux.org/term39-bin.git
cd term39-bin
makepkg -si

# Source package
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
