# Release Process

This document describes the release process for the SuperConfig monorepo.

## Manual Release Workflow

Releases are triggered manually through GitHub Actions with built-in approval gates and safety checks.

### How to Release

1. **Go to GitHub Actions**: Navigate to the "Actions" tab in the GitHub repository
2. **Select Release Workflow**: Click on "Release" in the left sidebar
3. **Run Workflow**: Click "Run workflow" button on the right
4. **Configure Release**:
   - **Package**: Select which package to release (currently only `superconfig`)
   - **Version Level**: Choose the version bump type:
     - `patch`: Bug fixes (0.1.0 → 0.1.1)
     - `minor`: New features (0.1.0 → 0.2.0)
     - `major`: Breaking changes (0.1.0 → 1.0.0)
     - `prerelease`: Pre-release version (0.1.0 → 0.1.0-rc.20241123123045)
   - **Dry Run**: Default is `true` - always test first!
   - **Force Publish**: Override version existence checks

### Release Workflow Steps

#### 1. Preparation Phase

- Extracts current version from `Cargo.toml`
- Calculates new version based on selected bump level
- Checks if the new version already exists on crates.io
- Validates release conditions

#### 2. Build and Test Phase

- Runs complete test suite with `cargo test --all-features`
- Performs code formatting checks with `cargo fmt`
- Runs clippy lints with `cargo clippy --all-targets --all-features`
- Builds release version with `cargo build --release --all-features`
- Performs security audit with `cargo audit`

#### 3. Release Phase (Environment: `release`)

- **Requires manual approval** for production releases
- Updates version in `Cargo.toml` and `Cargo.lock`
- Generates changelog from git commits
- Creates release commit and git tag
- Publishes to crates.io (if not dry run)
- Creates GitHub release with generated changelog

#### 4. Notification Phase

- Provides release summary in GitHub Actions UI
- Shows version change, dry run status, and final result

### Safety Features

- **Dry Run Default**: All releases default to dry run mode
- **Environment Protection**: Production releases require manual approval
- **Version Existence Check**: Prevents accidental republishing
- **Complete Test Suite**: All tests must pass before release
- **Security Audit**: Automatic vulnerability scanning
- **Force Override**: Option to override safety checks when needed

### GitHub Environment Setup

To enable approval gates, create a GitHub environment named `release`:

1. Go to **Settings** → **Environments**
2. Click **New environment**
3. Name it `release`
4. Add **Required reviewers** (yourself or team members)
5. Set **Wait timer** if desired (optional delay before deployment)

### Required Secrets

Add these secrets in **Settings** → **Secrets and variables** → **Actions**:

- `CARGO_REGISTRY_TOKEN`: Your crates.io API token for publishing
- `GITHUB_TOKEN`: Automatically provided by GitHub Actions

### Example Release Commands

```bash
# Test a patch release (dry run)
# Go to Actions → Release → Run workflow
# Package: superconfig
# Version Level: patch
# Dry Run: true

# Actual minor release (after testing)
# Go to Actions → Release → Run workflow  
# Package: superconfig
# Version Level: minor
# Dry Run: false
```

### Conventional Commits

While not automated yet, following conventional commit format helps with changelog generation:

- `feat:` New features (minor version bump)
- `fix:` Bug fixes (patch version bump)
- `docs:` Documentation changes
- `chore:` Maintenance tasks
- `BREAKING CHANGE:` or `!` suffix for major version bumps

### Troubleshooting

**Release fails with "Version already exists"**:

- Check if the version was already published to crates.io
- Use different version level or wait for next release cycle
- Use `force_publish: true` only if certain the version should be overridden

**Tests fail during release**:

- Fix the failing tests on the main branch first
- Never skip tests or use force flags
- Re-run the release workflow after fixes are merged

**Approval required but no reviewers**:

- Set up the `release` environment with required reviewers
- Or temporarily remove environment protection for testing

**Cargo publish fails**:

- Verify `CARGO_REGISTRY_TOKEN` is set correctly
- Check crates.io status and rate limits
- Ensure all dependencies are published and available

### Future Enhancements

- [ ] Automatic conventional commit analysis
- [ ] Multi-language package support
- [ ] Enhanced changelog generation with categorization
- [ ] Integration with cargo-release for advanced features
- [ ] Automated dependency updates
- [ ] Release branch strategy for hotfixes
