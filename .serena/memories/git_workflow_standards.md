# Git Workflow Standards

## Incremental Development Approach

- **ALWAYS work in small, incremental pieces** - never implement multiple large features at once
- **ONE small piece at a time** - focus on completing one file, function, or feature before moving to the next
- **ALWAYS add and commit to git after each incremental update** - never accumulate large changesets

## Commit Frequency and Timing

- **After creating each new file** - commit immediately after creation and testing
- **After completing each function or feature** - commit as soon as it's working and tested
- **After fixing each issue** - commit fixes individually rather than batching them
- **After each test passes** - commit working code promptly

## Git Workflow Process

1. **Create/modify one small piece** (single file, function, or feature)
2. **Test the change immediately** - ensure it works before proceeding
3. **Add to git staging** - `git add` the specific files changed
4. **Commit with descriptive message** - clear, conventional commit format
5. **Move to next small piece** - repeat the cycle

## Benefits of Incremental Commits

- **Easier rollback** - can revert individual changes without losing other work
- **Clear progress tracking** - each commit represents a working state
- **Better code review** - smaller, focused changes are easier to review
- **Reduced risk** - smaller changes have less potential for breaking things
- **Better debugging** - can isolate issues to specific commits

## Commit Message Standards

- Use conventional commit format: `type(scope): description`
- Each commit should represent one logical change
- Be specific about what was accomplished
- Reference issue numbers or feature names when applicable

## Example Workflow

```bash
# 1. Create one file
# 2. Test it works
git add crates/logfusion/examples/new_example.rs
git commit -m "feat(logfusion): add new example demonstrating feature X"

# 3. Create next file  
# 4. Test it works
git add crates/logfusion/tests/integration/new_test.rs
git commit -m "test(logfusion): add integration tests for feature X"

# 5. Update documentation
git add crates/logfusion/README.md
git commit -m "docs(logfusion): add feature X documentation to README"
```

## Anti-patterns to Avoid

- ❌ Creating multiple files before committing any
- ❌ Implementing entire features before any commits
- ❌ Accumulating changes across multiple days without commits
- ❌ "Batch commits" that include unrelated changes
- ❌ Working for hours without any git commits
