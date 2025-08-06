# Zero Warnings Policy

## Critical Rule: Absolute Zero Warnings

- **ZERO WARNINGS** are acceptable in any code
- **NO SHORTCUTS** allowed to silence warnings
- **NEVER use underscore prefixes** like `_variable` to silence unused variable warnings
- **NEVER use `#[allow(dead_code)]`** or similar macros to hide warnings
- **ALWAYS fix the root cause** of warnings, not suppress them
- **ALWAYS use Moon commands** for building and testing

## Warning Resolution Strategy

1. **Identify the actual issue** causing the warning
2. **Fix the underlying problem** (use variables properly, remove unused code, etc.)
3. **Verify fix with clean build** using `cargo clean` if needed
4. **Test with Moon commands** to ensure zero warnings in CI/CD pipeline

## Examples of FORBIDDEN shortcuts:

- ❌ `let _code = ...` to silence unused variable warning
- ❌ `#[allow(unused_variables)]` annotations
- ❌ `#[allow(dead_code)]` annotations
- ❌ Any `#[allow(...)]` attributes to suppress warnings

## Examples of CORRECT fixes:

- ✅ Actually use the variable in meaningful code
- ✅ Remove unused variables/code entirely
- ✅ Refactor code to eliminate the warning source
- ✅ Clean rebuild to ensure warnings are truly gone

This policy ensures code quality and prevents technical debt accumulation.
