#!/bin/bash

# deploy-homebrew.sh
# Manual deployment script for Homebrew formula using prebuilt binaries

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
REPO_URL="https://github.com/alejandroqh/term39"
RELEASE_BASE_URL="${REPO_URL}/releases/download/v${VERSION}"

echo -e "${GREEN}==> Homebrew Deployment for term39 v${VERSION}${NC}"
echo ""

# Helper to fetch sha256 of a URL
calc_sha256() {
  local url="$1"
  echo -e "${YELLOW}  - Fetching SHA256 for ${url}${NC}" >&2

  local tmp
  tmp=$(mktemp)
  if ! curl -fsL "$url" -o "$tmp"; then
    echo -e "${RED}Error: Failed to download ${url}${NC}" >&2
    rm -f "$tmp"
    exit 1
  fi

  shasum -a 256 "$tmp" | awk '{print $1}'
  rm -f "$tmp"
}

echo -e "${YELLOW}Collecting URLs for prebuilt binaries...${NC}"

# macOS binaries
MAC_ARM_URL="${RELEASE_BASE_URL}/term39-${VERSION}-macos-64bit-arm-binary.tar.gz"
MAC_INTEL_URL="${RELEASE_BASE_URL}/term39-${VERSION}-macos-64bit-x86-binary.tar.gz"

# Linux binaries
LINUX_X86_64_URL="${RELEASE_BASE_URL}/term39-${VERSION}-linux-64bit-x86-binary.tar.gz"
LINUX_ARM64_URL="${RELEASE_BASE_URL}/term39-${VERSION}-linux-64bit-arm-binary.tar.gz"

echo -e "${YELLOW}Calculating SHA256 for each binary archive...${NC}"

MAC_ARM_SHA=$(calc_sha256 "${MAC_ARM_URL}")
MAC_INTEL_SHA=$(calc_sha256 "${MAC_INTEL_URL}")
LINUX_X86_64_SHA=$(calc_sha256 "${LINUX_X86_64_URL}")
LINUX_ARM64_SHA=$(calc_sha256 "${LINUX_ARM64_URL}")

echo ""
echo -e "${GREEN}Computed SHA256 hashes:${NC}"
echo "  macOS arm64 : ${MAC_ARM_SHA}"
echo "  macOS intel : ${MAC_INTEL_SHA}"
echo "  linux x86_64: ${LINUX_X86_64_SHA}"
echo "  linux arm64 : ${LINUX_ARM64_SHA}"
echo ""

# Generate formula
FORMULA_FILE="term39.rb"
cat > "${FORMULA_FILE}" << EOF
class Term39 < Formula
  desc "Modern, retro-styled terminal multiplexer with a classic MS-DOS aesthetic"
  homepage "${REPO_URL}"
  version "${VERSION}"
  license "MIT"

  on_macos do
    on_arm do
      url "${MAC_ARM_URL}"
      sha256 "${MAC_ARM_SHA}"
    end

    on_intel do
      url "${MAC_INTEL_URL}"
      sha256 "${MAC_INTEL_SHA}"
    end
  end

  on_linux do
    on_arm do
      url "${LINUX_ARM64_URL}"
      sha256 "${LINUX_ARM64_SHA}"
    end

    on_intel do
      url "${LINUX_X86_64_URL}"
      sha256 "${LINUX_X86_64_SHA}"
    end
  end

  def install
    # Adjust if tarball contains a subfolder
    bin.install "term39"
  end

  test do
    assert_match "term39", shell_output("\#{bin}/term39 --version")
  end
end
EOF

echo -e "${GREEN}==> Formula generated: ${FORMULA_FILE}${NC}"
echo ""
cat "${FORMULA_FILE}"
echo ""

# Deploy to Homebrew tap
echo -e "${GREEN}==> Deploying to Homebrew tap...${NC}"
echo ""

# Check if homebrew tap exists
if [ ! -d "../homebrew-term39" ]; then
    echo -e "${RED}Error: homebrew-term39 directory not found at ../homebrew-term39${NC}"
    echo "Please clone your tap repository first:"
    echo "  git clone https://github.com/alejandroqh/homebrew-term39.git ../homebrew-term39"
    exit 1
fi

# Create Formula directory if it doesn't exist
mkdir -p ../homebrew-term39/Formula

# Copy formula to tap
echo -e "${YELLOW}Copying formula to homebrew tap...${NC}"
cp ${FORMULA_FILE} ../homebrew-term39/Formula/

# Navigate to tap repository
cd ../homebrew-term39

# Git operations
echo -e "${YELLOW}Preparing git commit...${NC}"
git add Formula/${FORMULA_FILE}
git commit -m "Update term39 to v${VERSION}"

# Show what will be pushed
echo ""
echo -e "${GREEN}Ready to push the following commit:${NC}"
git log --oneline -1
echo ""

# Ask for confirmation
read -p "Do you want to push this to the remote repository? (y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Pushing to remote...${NC}"
    git push
    echo ""
    echo -e "${GREEN}==> Deployment complete!${NC}"
    echo ""
    echo "Users can now install with:"
    echo "  brew tap alejandroqh/term39"
    echo "  brew install term39"
    echo ""
    echo "Or install directly:"
    echo "  brew install alejandroqh/term39/term39"
else
    echo -e "${YELLOW}Push cancelled. The commit has been created locally.${NC}"
    echo "You can push it manually later with: git push"
fi
