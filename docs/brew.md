# Homebrew Distribution

Nostos is available via Homebrew for macOS and Linux.

## Installation

```bash
brew tap pegesund/nostos
brew install nostos
```

## How It Works

### User Installation
1. `brew tap` adds the pegesund/nostos tap (https://github.com/pegesund/homebrew-nostos)
2. `brew install nostos` downloads the prebuilt binary for your platform
3. The binary has stdlib embedded - no additional files needed

### Release Automation

The Homebrew formula is **automatically updated** when you push a version tag:

```bash
# Create and push a release tag
git tag -a v0.3.0 -m "Release v0.3.0"
git push origin v0.3.0
```

This triggers the following workflow:

1. **Tag pushed** (`v*`) → `.github/workflows/release.yml` starts
2. **Build** → Compiles binaries for:
   - macOS ARM64 (M1/M2/M3)
   - macOS Intel (x86_64)
   - Linux x86_64
   - Windows x86_64
3. **Release** → Creates GitHub release with binaries and SHA256 checksums
4. **Homebrew update** → Triggers `update-formula` workflow in `pegesund/homebrew-nostos`
5. **Formula updated** → New version and checksums committed automatically

### What Does NOT Trigger Updates

- Pushing to `main` branch
- Pushing to feature branches
- Creating draft releases

Only pushing a `v*` tag triggers the full release + Homebrew update.

## Repository Structure

- **Main repo**: `pegesund/nostos` - source code, release workflow
- **Homebrew tap**: `pegesund/homebrew-nostos` - Formula and update workflow

## Secrets Required

The `HOMEBREW_TAP_TOKEN` secret must be set in the main nostos repo:
- Go to: https://github.com/pegesund/nostos/settings/secrets/actions
- This token allows the release workflow to trigger the formula update

## Manual Formula Update

If auto-update fails, manually update `homebrew-nostos/Formula/nostos.rb`:

1. Update `version "X.Y.Z"`
2. Update SHA256 checksums (from release page)
3. Commit and push to homebrew-nostos repo

## Testing Installation

```bash
# Install
brew tap pegesund/nostos
brew install nostos

# Verify
nostos --version
echo 'main() = 42' | nostos /dev/stdin
```
