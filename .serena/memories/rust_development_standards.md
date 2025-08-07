# Rust Development Standards

## Edition and Version Requirements

- **ALWAYS use Rust edition 2024** - the latest edition with modern language features
- **ALWAYS use the latest stable Rust version** (currently 1.88.0+)
- **ALWAYS use the latest versions of dependencies** - check crates.io for current versions
- **ALWAYS consult the latest documentation** when implementing features

## Documentation Sources Priority

1. **Official crates.io documentation** - for latest API changes
2. **GitHub repository documentation** - for cutting-edge features
3. **Rust Book and Reference** - for language features and best practices
4. **Context7 MCP server** - for up-to-date library documentation

## Version Update Strategy

- Before adding any dependency, check the latest version available
- Use semantic versioning appropriately (^major.minor.patch)
- Regularly audit and update dependencies to latest stable versions
- Test thoroughly after version updates

## Modern Rust Practices

- Use latest language features available in edition 2024
- Follow current Rust API guidelines and conventions
- Leverage modern async/await patterns where applicable
- Use latest procedural macro capabilities
- Apply current error handling best practices

## Quality Assurance

- Always test examples and code after creation/modification
- Ensure all dependencies compile with latest Rust version
- Verify compatibility with latest toolchain features
- Use latest cargo features and commands
