# SuperConfig CLI Library Selection

[🚪 ← Back to Decisions Overview](../DECISIONS.md)

## Decision Summary

**Status**: ❓ **Under Discussion** - CLI library selection for SuperConfig interactive features\
**Priority**: 🟢 **Planning** (month)\
**Date**: 2025-08-07\
**Category**: 🏗️ Infrastructure

## Action Items

### 🟢 This Month (Planning)

- [ ] Evaluate CLI library options for SuperConfig interactive features [[SuperConfig CLI Library Selection]](2025-08-07-superconfig-cli-library-selection.md)
- [ ] Create proof-of-concept with selected library [[SuperConfig CLI Library Selection]](2025-08-07-superconfig-cli-library-selection.md)
- [ ] Design CLI user experience flow [[SuperConfig CLI Library Selection]](2025-08-07-superconfig-cli-library-selection.md)

---

## Context & Problem

SuperConfig is a configuration management toolkit that could benefit from interactive CLI capabilities for:

1. **Configuration Generation** - Interactive prompts to create config files
2. **Configuration Validation** - Visual feedback on config errors and suggestions
3. **Configuration Debugging** - Interactive exploration of merged configuration sources
4. **Project Setup** - Guided setup for new projects using SuperConfig

The question is whether to implement these features and which Rust library to use for the best developer experience.

## Research: Available Rust CLI Libraries

### 1. **inquire** (Recommended)

**Repository**: https://github.com/mikaelmello/inquire\
**Version**: 0.7\
**Maintainer**: Active (updated 2024)

**Strengths**:

- ✅ Modern, clean API design
- ✅ Rich prompt types (text, select, multi-select, confirm, password)
- ✅ Built-in fuzzy search for selections
- ✅ Custom validators and auto-completion
- ✅ Excellent error handling
- ✅ Good documentation and examples

**Example**:

```rust
let format = Select::new("Config format?", vec!["TOML", "YAML", "JSON"]).prompt()?;
let name = Text::new("Project name:").with_default("myapp").prompt()?;
let features = MultiSelect::new("Features:", vec!["logging", "metrics", "auth"]).prompt()?;
```

### 2. **dialoguer** (Popular Choice)

**Repository**: https://github.com/console-rs/dialoguer\
**Version**: 0.11\
**Maintainer**: console-rs team (very active)

**Strengths**:

- ✅ Most widely adopted in Rust ecosystem
- ✅ Mature and stable API
- ✅ Extensive prompt types including progress bars
- ✅ Good Windows/cross-platform support
- ✅ Part of console-rs ecosystem (with `console` crate)

**Considerations**:

- 🔄 Slightly more verbose API compared to inquire
- 🔄 Less modern UX compared to inquire's fuzzy search

### 3. **requestty** (Inquirer.js-like)

**Repository**: https://github.com/Lutetium-Vanadium/requestty\
**Version**: 0.5\
**Maintainer**: Less active (last update mid-2023)

**Strengths**:

- ✅ Familiar API for JavaScript developers (inquirer.js-like)
- ✅ Good prompt chaining capabilities
- ✅ Flexible prompt system

**Concerns**:

- ❌ Less active maintenance
- ❌ Smaller community adoption

### 4. **ratatui** (TUI Framework)

**Repository**: https://github.com/ratatui-org/ratatui\
**Version**: 0.28\
**Maintainer**: Very active community

**Strengths**:

- ✅ Full terminal UI framework (like ncurses)
- ✅ Powerful widgets and layouts
- ✅ Real-time interactive applications
- ✅ Excellent for complex interfaces

**Considerations**:

- 🔄 Overkill for simple prompts
- 🔄 Steeper learning curve
- 🔄 Better for full TUI apps than simple CLI prompts

### 5. **Building Blocks Approach** (console + indicatif)

**Repositories**:

- https://github.com/console-rs/console (styling, input)
- https://github.com/console-rs/indicatif (progress bars)

**Strengths**:

- ✅ Maximum flexibility and control
- ✅ Well-maintained by console-rs team
- ✅ Used by many other tools as foundation
- ✅ Can build exactly what we need

**Considerations**:

- 🔄 More development work required
- 🔄 Need to implement prompt logic ourselves

## Decision Analysis

### Use Cases for SuperConfig CLI

1. **Configuration Generator**:
   ```
   ? Project name: myapp
   ? Config format: TOML
   ? Features: [x] logging [ ] metrics [x] auth
   ? Environment support: [x] dev [x] prod [ ] staging
   ✅ Created myapp.toml with 3 features
   ```

2. **Configuration Validator**:
   ```
   ⚠️  Config validation found issues:
   • database.host is missing (required)
   • server.port: 80 conflicts with auth.port: 80
   ? Auto-fix conflicts? Yes
   ✅ Fixed 2 issues in config.toml
   ```

3. **Configuration Explorer**:
   ```
   📁 Configuration Sources:
   ├─ ~/.config/myapp/config.toml (system)
   ├─ ./myapp.toml (project) ← overrides 2 values
   └─ ENV MYAPP_* (runtime) ← overrides 1 value

   ? Explore section: database >
   ```

### Recommendation: **inquire**

**Rationale**:

1. **Modern UX**: Best-in-class user experience with fuzzy search and clean interface
2. **API Quality**: Most intuitive and ergonomic API for common use cases
3. **Active Development**: Well-maintained with regular updates
4. **Perfect Fit**: Exactly what we need without being overkill
5. **Documentation**: Excellent docs and examples

**Implementation Plan**:

1. Add `inquire` as optional dependency (feature-gated)
2. Create `superconfig::cli` module with helper functions
3. Implement config generation wizard first
4. Add validation and debugging tools later

### Alternative Consideration: No CLI Library

**Decision Point**: Do we actually need interactive CLI features for SuperConfig?

**Arguments Against**:

- SuperConfig is primarily a library, not a CLI tool
- Configuration is often automated/scripted in production
- Adds complexity and dependencies
- Most users prefer programmatic configuration

**Arguments For**:

- Developer experience improvement for onboarding
- Configuration debugging is genuinely useful
- Interactive validation can catch errors early
- Competitive advantage over other config libraries

## Recommendation

**Phase 1**: **Defer CLI features** for now, focus on core library capabilities

- SuperConfig should excel as a library first
- CLI features are nice-to-have, not essential
- Can always add later with feature gates

**Phase 2**: If CLI features become needed, use **inquire**

- Start with configuration validation/debugging tools
- Add generation wizard if users request it
- Keep behind optional feature flag

## Implementation Notes

If we do implement CLI features:

```toml
[dependencies]
inquire = { version = "0.7", optional = true }

[features]
cli = ["inquire"]
```

```rust
#[cfg(feature = "cli")]
pub mod cli {
    use inquire::*;
    
    pub fn generate_config() -> Result<()> {
        let name = Text::new("Project name:").prompt()?;
        let format = Select::new("Format:", vec!["TOML", "YAML", "JSON"]).prompt()?;
        // ... implementation
    }
    
    pub fn debug_config() -> Result<()> {
        // Interactive config exploration
    }
}
```

---

## Related Decisions

- [Development Focus](2025-08-06-development-focus.md) - SuperConfig as core product
- [Repository Architecture](2025-08-06-repository-architecture.md) - Monorepo structure

---

_Decision Status: Under Discussion | Next Review: After core SuperConfig features complete_
