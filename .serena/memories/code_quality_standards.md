# Code Quality Standards

## CRITICAL: ZERO WARNINGS POLICY

- **ABSOLUTELY NO WARNINGS ALLOWED** - This is the #1 rule that must NEVER be violated
- **MANDATORY**: Fix ALL compiler warnings before completing ANY task
- **MANDATORY**: Fix ALL clippy warnings before completing ANY task
- **MANDATORY**: Ensure all tests pass with NO warnings
- **PROCESS**: Always use Moon commands for builds and tests
- **PRINCIPLE**: Warnings indicate potential issues and MUST be addressed immediately

## MOON COMMANDS REQUIRED

- **ALWAYS use Moon commands instead of direct cargo commands**
- **Build**: `moon run logffi:build` (NOT `cargo build`)
- **Test**: `moon run logffi:test` (NOT `cargo test`)
- **Lint**: `moon run logffi:lint` (NOT `cargo clippy`)
- **Format**: `moon run logffi:format` (NOT `cargo fmt`)
- **Quality checks**: `moon run --affected :lint` and `moon run --affected :format`

## Warning Fixing Workflow

1. **After ANY code change** - immediately run `moon run logffi:lint`
2. **Fix every single warning** - no exceptions, no "leave it for later"
3. **Test again** - ensure fixes don't break functionality with `moon run logffi:test`
4. **Only then commit** - never commit code with warnings

## Testing Standards

- All tests must pass with ZERO warnings using `moon run logffi:test`
- No ignored tests without explicit justification
- Comprehensive test coverage for new features
- Integration tests for public APIs

## Documentation Standards

- New features require proper documentation placement
- README updates for user-facing features
- Code comments for complex logic
- Examples for new functionality

## REMINDER: ZERO WARNINGS + MOON COMMANDS ARE NON-NEGOTIABLE

These are the most critical requirements that must be followed for ALL development work.
