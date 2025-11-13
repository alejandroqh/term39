# Testing TERM39 Framebuffer Backend on Debian ARM64

## What Was Built

✅ **ARM64 Linux binary** with framebuffer support: `target/aarch64-unknown-linux-gnu/release/term39` (1.9 MB)
✅ **Debian package** for ARM64: `target/debian/term39_0.6.6-1_arm64.deb` (658 KB)

### Updates Made:
1. **Gzip font support** - Can read `.psf.gz` compressed fonts
2. **Debian font paths** - Looks in `/usr/share/consolefonts/` (primary) and `/usr/share/kbd/consolefonts/` (fallback)
3. **Updated font names** - Uses Debian font names (`Uni3-Terminus16`, etc.)
4. **Short flag** - Added `-f` / `--framebuffer` flag to enable framebuffer mode

## Installation on Debian ARM64

### Transfer the Package

Copy the .deb file to your Debian ARM64 system:

```bash
# From macOS to Debian (using scp)
scp target/debian/term39_0.6.6-1_arm64.deb user@debian-host:~/

# Or use USB drive, network share, etc.
```

### Install on Debian

```bash
# Install the package
sudo dpkg -i term39_0.6.6-1_arm64.deb

# If dependencies are missing, fix them:
sudo apt-get install -f

# Verify installation
which term39
# Should output: /usr/bin/term39

term39 --version
```

## Testing on Linux Console (TTY)

### Prerequisites

Ensure console fonts are installed:

```bash
# Install kbd package (provides console fonts)
sudo apt install kbd

# Verify fonts exist
ls /usr/share/consolefonts/Uni3-Terminus*.psf.gz
```

### Test Steps

#### 1. Switch to Console

Press `Ctrl+Alt+F2` to switch to TTY2 (or F3, F4, etc.)

Login with your username and password.

#### 2. Grant Framebuffer Access

```bash
# Option A: Run with sudo (quick test)
sudo term39 -f --fb-mode=80x25

# Option B: Add user to video group (recommended)
sudo usermod -a -G video $USER
# Log out and log back in, then:
term39 -f --fb-mode=80x25
```

#### 3. Test Different Modes

**Classic DOS Modes:**
```bash
# Standard DOS mode (80x25)
sudo term39 -f --fb-mode=80x25

# High density mode (80x50)
sudo term39 -f --fb-mode=80x50

# Wide character mode (40x25)
sudo term39 -f --fb-mode=40x25

# Medium density (80x43)
sudo term39 -f --fb-mode=80x43
```

**High-Resolution Modes:**
```bash
# Double-wide standard (160x50)
sudo term39 -f --fb-mode=160x50

# High resolution (160x100)
sudo term39 -f --fb-mode=160x100

# Ultra-wide (320x100)
sudo term39 -f --fb-mode=320x100

# Maximum resolution (320x200) - for high-res displays
sudo term39 -f --fb-mode=320x200
```

**With Scaling:**
```bash
# Auto-scale (automatically calculates best fit)
sudo term39 -f --fb-mode=320x200

# Explicit scaling (e.g., 2x scale)
sudo term39 -f --fb-mode=160x50 --fb-scale=2

# For 2560x1600 displays, 320x200 with auto-scale fills the screen perfectly!
sudo term39 -f --fb-mode=320x200
```

#### 4. Test Without Framebuffer

```bash
# Should work in terminal mode (no framebuffer)
term39

# Even with feature compiled, terminal mode is default without -f flag
term39 --fb-mode=80x25  # Still uses terminal, -f not specified
```

#### 5. Return to GUI

Press `Ctrl+Alt+F1` or `Ctrl+Alt+F7` to return to graphical desktop.

## Expected Behavior

### ✅ Success Indicators:
- Application starts without errors
- See message: `Framebuffer backend initialized: 80x25` (or your selected mode)
- Pixel-perfect DOS-style rendering visible
- Can create terminal windows with `t` key
- Mouse works (via GPM if installed)
- Keyboard works normally

### ⚠️ Common Issues:

**"Failed to open /dev/fb0"**
- Not on a Linux console (TTY)
- Running in terminal emulator or over SSH
- Need sudo or video group membership

**"No suitable font found"**
- `kbd` package not installed: `sudo apt install kbd`
- Fonts not in `/usr/share/consolefonts/`

**Automatically falls back to terminal mode**
- This is expected if framebuffer can't initialize
- Application will still work in terminal mode

## Command Reference

```bash
# Enable framebuffer (short flag)
term39 -f

# Enable framebuffer (long flag)
term39 --framebuffer

# Specify text mode
term39 -f --fb-mode=80x50

# Terminal mode (default, even with framebuffer-backend compiled)
term39

# Show help
term39 --help
```

## Debugging

### Check /dev/fb0 access:

```bash
ls -l /dev/fb0
# Should show: crw-rw---- 1 root video ...

# Check if you're in video group:
groups
```

### Check available fonts:

```bash
ls /usr/share/consolefonts/Uni3-Terminus*.psf.gz
```

### Test font loading manually:

```bash
# Load a font to test
sudo setfont /usr/share/consolefonts/Uni3-Terminus16.psf.gz
```

### Check framebuffer info:

```bash
cat /sys/class/graphics/fb0/virtual_size
cat /sys/class/graphics/fb0/bits_per_pixel
```

## What's Different from Terminal Mode

| Aspect | Terminal Mode | Framebuffer Mode |
|--------|--------------|------------------|
| Access | Any terminal, SSH | Console (TTY) only |
| Rendering | Character-based | Pixel-based |
| Fonts | Terminal's font | PSF2 bitmap fonts |
| Permissions | User | Root or video group |
| Resolution | Variable | Fixed text mode |
| Remote | Yes | No |
| Performance | Good | Excellent |

## Notes

- Framebuffer mode is **experimental**
- Only works on Linux console (TTY1-6)
- Requires `/dev/fb0` access
- PSF2 fonts must be in `/usr/share/consolefonts/`
- GPM for mouse support: `sudo apt install gpm && sudo systemctl start gpm`
- The `-f` flag is required to enable framebuffer mode
- Without `-f`, uses standard terminal backend even if framebuffer-backend feature is compiled

## Support

If you encounter issues:
1. Make sure you're on a real Linux console (not terminal emulator)
2. Verify `kbd` package is installed
3. Check `/dev/fb0` permissions
4. Try with `sudo` first
5. Check for fonts: `ls /usr/share/consolefonts/`

Enjoy your DOS-like framebuffer experience! 🎮
