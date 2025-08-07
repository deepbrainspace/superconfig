# Development Workflow Best Practices

## Git Workflow Rules

### CRITICAL: Always Commit and Push in Small Chunks

**MANDATORY**: Work in small incremental chunks, commit frequently, and push after each working milestone.

**Workflow Pattern**:

1. Complete a small piece of work
2. Test that it works
3. Fix any warnings/errors
4. Commit with descriptive message
5. Push to remote
6. Move to next small piece

**Example Good Practice**:

- Complete one example → test → commit → push
- Fix warnings → test → commit → push
- Add one test file → test → commit → push
- Update documentation → commit → push

**Why This Matters**:

- Prevents losing work
- Makes debugging easier with smaller changesets
- Shows steady progress to collaborators
- Enables easier rollbacks if needed
- Follows professional development practices

## Zero Warnings Policy

**MANDATORY**: Always maintain zero warnings in the codebase.

**Rules**:

- ❌ NEVER use underscore shortcuts (`_variable`) to silence warnings
- ❌ NEVER use `#[allow()]` macros to bypass warnings
- ✅ ALWAYS fix root causes of warnings
- ✅ Use `moon clean --lifetime "0 seconds"` if warnings appear cached
- ✅ Test after every warning fix to ensure they're eliminated

## Testing Integration

**MANDATORY**: Always test examples by running them.

**Rules**:

- Configure examples to run as tests using `test = true` in Cargo.toml
- Use Moon commands: `moon <project>:test` instead of direct cargo
- Verify examples work by actually executing them
- Fix any test failures immediately before moving on

## Command Usage

**MANDATORY**: Use Moon commands instead of direct cargo commands.

**Correct**:

- ✅ `moon logfusion:test`
- ✅ `moon logfusion:build`
- ✅ `moon logfusion:lint`

**Incorrect**:

- ❌ `cargo test`
- ❌ `cargo build`
- ❌ `cargo clippy`

## Incremental Development Approach

1. **Plan** - Create todo list with TodoWrite tool
2. **Implement** - Work on one small piece at a time
3. **Test** - Run tests after each piece
4. **Fix** - Address any warnings/failures immediately
5. **Commit** - Commit working code with descriptive message
6. **Push** - Push to remote to save progress
7. **Repeat** - Move to next small piece

This approach was successfully demonstrated in the LogFusion error introspection enhancement project where 9 tasks were completed incrementally with regular commits and pushes.
