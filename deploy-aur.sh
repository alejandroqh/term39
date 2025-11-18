#!/usr/bin/env bash
# Script to automate AUR package deployment for term39

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUR_SOURCE_DIR="${SCRIPT_DIR}/../aur-term39"
AUR_BIN_DIR="${SCRIPT_DIR}/../aur-term39-bin"

# Container settings
USE_CONTAINER=false
CONTAINER_CMD=""
BUILDER_IMAGE="term39-aur-builder"
PACMAN_CACHE_VOLUME="term39-pacman-cache"

# Function to print colored messages
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to extract version from Cargo.toml
get_version() {
    grep '^version = ' "${SCRIPT_DIR}/Cargo.toml" | head -n1 | sed 's/version = "\(.*\)"/\1/'
}

# Function to update PKGBUILD version
update_pkgbuild_version() {
    local pkgbuild_file="$1"
    local new_version="$2"

    sed -i.bak "s/^pkgver=.*/pkgver=${new_version}/" "$pkgbuild_file"
    sed -i.bak "s/^pkgrel=.*/pkgrel=1/" "$pkgbuild_file"
    rm "${pkgbuild_file}.bak"
}

# Function to update checksums in PKGBUILD
update_checksum() {
    local pkgbuild_file="$1"
    local checksum_var="$2"
    local new_checksum="$3"

    sed -i.bak "s|^${checksum_var}=.*|${checksum_var}=('${new_checksum}')|" "$pkgbuild_file"
    rm "${pkgbuild_file}.bak"
}

# Function to download file and calculate SHA256
download_and_hash() {
    local url="$1"
    local output_file="$2"

    info "Downloading: $url" >&2
    if curl -L -f -o "$output_file" "$url" 2>&1 | grep -v "^  " >&2; then
        # Use shasum on macOS, sha256sum on Linux
        if command -v sha256sum &> /dev/null; then
            sha256sum "$output_file" | awk '{print $1}'
        else
            shasum -a 256 "$output_file" | awk '{print $1}'
        fi
    else
        error "Failed to download $url" >&2
        return 1
    fi
}

# Function to detect container runtime
detect_container_runtime() {
    if command -v podman &> /dev/null; then
        echo "podman"
    elif command -v docker &> /dev/null; then
        echo "docker"
    else
        echo ""
    fi
}

# Function to create or verify pacman cache volume
ensure_pacman_cache_volume() {
    if ! $CONTAINER_CMD volume inspect "$PACMAN_CACHE_VOLUME" &> /dev/null; then
        info "Creating pacman cache volume: $PACMAN_CACHE_VOLUME"
        $CONTAINER_CMD volume create "$PACMAN_CACHE_VOLUME"
        success "Pacman cache volume created"
    else
        info "Using existing pacman cache volume: $PACMAN_CACHE_VOLUME"
    fi
}

# Function to build or verify builder image exists
ensure_builder_image() {
    if ! $CONTAINER_CMD image inspect "$BUILDER_IMAGE" &> /dev/null; then
        info "Building AUR builder image: $BUILDER_IMAGE"
        info "This will take a moment but only happens once..."

        # Create a temporary Containerfile/Dockerfile
        local tmp_containerfile=$(mktemp)
        cat > "$tmp_containerfile" <<'EOF'
FROM archlinux:latest

# Update system and install build dependencies
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm base-devel namcap && \
    pacman -Scc --noconfirm

# Create builder user
RUN useradd -m -G wheel builder && \
    echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

# Set up builder user environment
USER builder
WORKDIR /home/builder
EOF

        $CONTAINER_CMD build --platform linux/amd64 -t "$BUILDER_IMAGE" -f "$tmp_containerfile" . > /dev/null 2>&1
        rm "$tmp_containerfile"
        success "Builder image created: $BUILDER_IMAGE"
    else
        info "Using existing builder image: $BUILDER_IMAGE"
    fi
}

# Function to run container with cache volume
run_with_cache() {
    local cmd="$1"
    # First fix permissions as root
    $CONTAINER_CMD run --rm --platform linux/amd64 \
        -v "$(pwd):/build" \
        -v "${PACMAN_CACHE_VOLUME}:/var/cache/pacman/pkg" \
        -w /build \
        "$BUILDER_IMAGE" \
        bash -c "chown -R builder:builder /build"

    # Then run the actual command as builder user
    $CONTAINER_CMD run --rm --platform linux/amd64 \
        -v "$(pwd):/build" \
        -v "${PACMAN_CACHE_VOLUME}:/var/cache/pacman/pkg" \
        -w /build \
        --user builder \
        "$BUILDER_IMAGE" \
        bash -c "$cmd"
}

# Function to test PKGBUILD validity
test_pkgbuild() {
    local dir="$1"
    local pkg_name=$(basename "$dir")
    cd "$dir"

    info "Testing PKGBUILD in $pkg_name"

    if [[ "$USE_CONTAINER" == "true" ]]; then
        info "Using container ($CONTAINER_CMD) to test PKGBUILD"
        if run_with_cache 'namcap PKGBUILD'; then
            success "PKGBUILD validation passed for $pkg_name"
            return 0
        else
            warning "PKGBUILD validation had warnings for $pkg_name (may be okay)"
            return 0
        fi
    else
        if command -v namcap &> /dev/null; then
            namcap PKGBUILD || warning "PKGBUILD validation had warnings (may be okay)"
        else
            warning "namcap not found - skipping PKGBUILD validation"
        fi
    fi
}

# Function to generate .SRCINFO
generate_srcinfo() {
    local dir="$1"
    local pkg_name=$(basename "$dir")
    cd "$dir"

    info "Generating .SRCINFO in $pkg_name"

    if [[ "$USE_CONTAINER" == "true" ]]; then
        info "Using container ($CONTAINER_CMD) to generate .SRCINFO"
        run_with_cache 'makepkg --printsrcinfo' > .SRCINFO
    else
        makepkg --printsrcinfo > .SRCINFO
    fi
}

# Function to commit and push AUR package
commit_and_push() {
    local dir="$1"
    local version="$2"
    local pkg_name=$(basename "$dir")

    cd "$dir"

    info "Committing changes for $pkg_name"
    git add PKGBUILD .SRCINFO
    git commit -m "Update to version ${version}"

    info "Pushing to AUR for $pkg_name"
    git push

    success "$pkg_name updated to version ${version}"
}

# Main script
main() {
    info "Starting AUR deployment process"

    # Detect if makepkg is available
    if ! command -v makepkg &> /dev/null; then
        warning "makepkg not found - attempting to use container runtime"
        CONTAINER_CMD=$(detect_container_runtime)
        if [[ -z "$CONTAINER_CMD" ]]; then
            error "Neither makepkg nor container runtime (podman/docker) found!"
            error "Please install podman or docker to continue."
            exit 1
        fi
        USE_CONTAINER=true
        success "Using $CONTAINER_CMD for makepkg operations"

        # Set up container infrastructure
        ensure_pacman_cache_volume
        ensure_builder_image
    else
        info "Using native makepkg"
    fi

    # Check if directories exist
    if [[ ! -d "$AUR_SOURCE_DIR" ]]; then
        error "AUR source directory not found: $AUR_SOURCE_DIR"
        exit 1
    fi

    if [[ ! -d "$AUR_BIN_DIR" ]]; then
        error "AUR binary directory not found: $AUR_BIN_DIR"
        exit 1
    fi

    # Get current version
    VERSION=$(get_version)
    info "Detected version: $VERSION"

    # Confirm with user
    echo ""
    warning "This will update both AUR packages to version $VERSION"
    read -p "Do you want to continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Deployment cancelled"
        exit 0
    fi

    # Create temporary directory for downloads
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    info "Using temporary directory: $TMP_DIR"

    # ============================================================
    # Update term39 (source package)
    # ============================================================
    info "Updating term39 (source package)..."

    SOURCE_URL="https://github.com/alejandroqh/term39/archive/v${VERSION}.tar.gz"
    SOURCE_FILE="${TMP_DIR}/term39-${VERSION}.tar.gz"

    SOURCE_SHA=$(download_and_hash "$SOURCE_URL" "$SOURCE_FILE")
    if [[ $? -ne 0 ]]; then
        error "Failed to download source tarball. Make sure the GitHub release exists."
        exit 1
    fi

    success "Source SHA256: $SOURCE_SHA" >&2

    # Update PKGBUILD
    update_pkgbuild_version "${AUR_SOURCE_DIR}/PKGBUILD" "$VERSION"
    update_checksum "${AUR_SOURCE_DIR}/PKGBUILD" "sha256sums" "$SOURCE_SHA"

    # Generate .SRCINFO (skipping validation for speed)
    # test_pkgbuild "$AUR_SOURCE_DIR"
    generate_srcinfo "$AUR_SOURCE_DIR"

    # ============================================================
    # Update term39-bin (binary package)
    # ============================================================
    info "Updating term39-bin (binary package)..."

    # x86_64 binary
    BIN_X64_URL="https://github.com/alejandroqh/term39/releases/download/v${VERSION}/term39-${VERSION}-linux-64bit-x86-binary.tar.gz"
    BIN_X64_FILE="${TMP_DIR}/term39-x86_64.tar.gz"

    BIN_X64_SHA=$(download_and_hash "$BIN_X64_URL" "$BIN_X64_FILE")
    if [[ $? -ne 0 ]]; then
        error "Failed to download x86_64 binary. Make sure the GitHub release exists."
        exit 1
    fi

    success "x86_64 SHA256: $BIN_X64_SHA" >&2

    # aarch64 binary
    BIN_ARM64_URL="https://github.com/alejandroqh/term39/releases/download/v${VERSION}/term39-${VERSION}-linux-64bit-arm-binary.tar.gz"
    BIN_ARM64_FILE="${TMP_DIR}/term39-arm64.tar.gz"

    BIN_ARM64_SHA=$(download_and_hash "$BIN_ARM64_URL" "$BIN_ARM64_FILE")
    if [[ $? -ne 0 ]]; then
        error "Failed to download aarch64 binary. Make sure the GitHub release exists."
        exit 1
    fi

    success "aarch64 SHA256: $BIN_ARM64_SHA" >&2

    # Update PKGBUILD
    update_pkgbuild_version "${AUR_BIN_DIR}/PKGBUILD" "$VERSION"
    update_checksum "${AUR_BIN_DIR}/PKGBUILD" "sha256sums_x86_64" "$BIN_X64_SHA"
    update_checksum "${AUR_BIN_DIR}/PKGBUILD" "sha256sums_aarch64" "$BIN_ARM64_SHA"

    # Generate .SRCINFO (skipping validation for speed)
    # test_pkgbuild "$AUR_BIN_DIR"
    generate_srcinfo "$AUR_BIN_DIR"

    # ============================================================
    # Show changes and confirm push
    # ============================================================
    echo ""
    info "Changes to be committed:"
    echo ""
    echo "=== term39 (source) ==="
    cd "$AUR_SOURCE_DIR"
    git diff PKGBUILD
    echo ""
    echo "=== term39-bin (binary) ==="
    cd "$AUR_BIN_DIR"
    git diff PKGBUILD
    echo ""

    warning "Ready to commit and push to AUR"
    read -p "Do you want to commit and push? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Changes made but not pushed. You can manually commit from the AUR directories."
        exit 0
    fi

    # Commit and push
    commit_and_push "$AUR_SOURCE_DIR" "$VERSION"
    commit_and_push "$AUR_BIN_DIR" "$VERSION"

    # ============================================================
    # Done!
    # ============================================================
    echo ""
    success "AUR deployment completed successfully!"
    info "Packages updated:"
    echo "  - term39: https://aur.archlinux.org/packages/term39"
    echo "  - term39-bin: https://aur.archlinux.org/packages/term39-bin"
}

# Run main function
main "$@"
