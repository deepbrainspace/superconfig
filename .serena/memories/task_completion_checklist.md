# Task Completion Checklist for SuperConfig

## Mandatory Steps When Completing Any Task

### 1. Code Quality Checks (Required)

```bash
# Run these commands for affected crates after any code changes:
moon run --affected :format-check    # Ensure code is properly formatted
moon run --affected :lint            # Check for linting issues
moon run --affected :test            # Ensure all tests pass
```

### 2. Security and Compliance (Required)

```bash
# Run security checks for affected crates:
moon run --affected :security-audit  # Check for security vulnerabilities
moon run --affected :deny            # Validate dependency policies
```

### 3. Documentation (When Applicable)

- Update rustdoc comments for any new public APIs
- Add examples to doc comments for complex functionality
- Update README.md if adding new major features
- Ensure `cargo doc` builds without warnings

### 4. Testing Requirements

- **Unit tests**: Required for new functions/methods
- **Integration tests**: Required for new major features
- **Coverage**: Aim for high coverage on new code
- **Examples**: Add runnable examples for new APIs

### 5. Git and Commit Standards

- **Conventional commits**: Use proper type(scope): description format
- **Clean commits**: Each commit should be a logical unit of work
- **Branch naming**: Use feat/, fix/, chore/ prefixes
- **Pre-commit hooks**: Will automatically run formatting and linting

### 6. Before Creating Pull Request

```bash
# Ensure repository is clean and up-to-date
git status                           # Check for uncommitted changes
moon run --affected :build-release   # Ensure release builds work
moon run --affected :coverage        # Generate coverage reports
```

### 7. Performance Considerations

- Run benchmarks for performance-critical changes
- Consider memory usage impact
- Validate zero-copy patterns where applicable
- Test with realistic data sizes

## Specific Task Types

### Adding New Features

1. **Design**: Consider FFI compatibility if applicable
2. **Implementation**: Follow zero-copy principles where possible
3. **Testing**: Unit + integration tests + examples
4. **Documentation**: Rustdoc with examples
5. **Benchmarks**: If performance-critical

### Bug Fixes

1. **Reproduce**: Add test that reproduces the bug
2. **Fix**: Implement the minimal fix
3. **Verify**: Ensure the test now passes
4. **Regression**: Check for related edge cases
5. **Documentation**: Update if behavior changes

### Refactoring

1. **Tests first**: Ensure comprehensive test coverage
2. **Small changes**: Keep refactoring changes focused
3. **Backwards compatibility**: Maintain API compatibility where possible
4. **Performance**: Verify no performance regression
5. **Documentation**: Update if API changes

### FFI Changes

1. **Multi-language testing**: Test with Python, Node.js if applicable
2. **Memory management**: Careful attention to ownership
3. **Error handling**: Use logfusion macros for consistent error handling
4. **Documentation**: Update multi-language examples

## Quality Gates (Must Pass)

### Automated Checks (Git Hooks)

- ✅ Code formatting (rustfmt + dprint)
- ✅ Linting (clippy with pedantic warnings)
- ✅ Conventional commit format
- ✅ Clean repository state

### CI Pipeline (Must Pass)

- ✅ Build in release mode
- ✅ All tests passing
- ✅ Security audit
- ✅ Dependency compliance
- ✅ Code coverage reporting

### Manual Verification

- ✅ New functionality works as expected
- ✅ Documentation is accurate and complete
- ✅ Examples run successfully
- ✅ No breaking changes (unless intentional and documented)

## Release Checklist (When Publishing)

### Pre-Release Validation

```bash
moon run CRATE:publish-dry           # Dry-run publish
moon run CRATE:doc                   # Ensure docs build
moon run CRATE:test                  # All tests pass
moon run CRATE:coverage-ci           # Generate coverage report
```

### Version Management

- Update version in Cargo.toml
- Update CHANGELOG.md with new features/fixes
- Create git tag for release
- Ensure conventional commits for proper semantic versioning

### Post-Release

- Verify publication on crates.io
- Update documentation links
- Test installation from crates.io
- Update dependent crates if needed
