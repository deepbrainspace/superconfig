# Moon Affected Detection Bug Report

## Issue
Moon's affected detection behavior is inconsistent between local and CI environments, requiring workarounds to achieve consistent behavior.

## Bug Description
Moon hardcodes different behavior for affected detection based on environment detection (`is_ci()`):

- **Local environment**: Uses `local: true`, only checks uncommitted changes via `git status --porcelain`
- **CI environment**: Uses `local: false`, compares against base branch properly

This means the same command `moon query projects --affected` produces different results locally vs CI, even with identical workspace configuration.

## Root Cause
In `/crates/app/src/queries/touched_files.rs:215-228`, Moon hardcodes:

```rust
pub async fn load_touched_files(vcs: &BoxedVcs) -> miette::Result<FxHashSet<WorkspaceRelativePathBuf>> {
    let ci = is_ci();
    
    query_touched_files_with_stdin(
        vcs,
        &QueryTouchedFilesOptions {
            default_branch: ci,    // true in CI, false locally
            local: !ci,           // false in CI, true locally
            // ...
        },
    )
}
```

This ignores workspace VCS configuration and forces different behavior based on environment.

## Current Workaround
We set `CI=true` and `MOON_BASE=origin/main` in project-level `env` section:

```yaml
# crates/superconfig/moon.yml
env:
  CI: 'true'
  MOON_BASE: 'origin/main'
```

This forces Moon to use CI-like behavior locally.

## Proposed Solution
Moon should respect workspace configuration for affected detection behavior instead of hardcoding based on environment detection. Suggested improvements:

1. **Add workspace-level config option**:
   ```yaml
   # .moon/workspace.yml
   vcs:
     manager: 'git'
     defaultBranch: 'main'
     remoteCandidates: ['origin']
     affectedMode: 'branch'  # 'local' | 'branch' | 'auto'
   ```

2. **Respect MOON_BASE/MOON_HEAD environment variables** consistently across all environments

3. **Add task option to override affected detection mode** per project

## Impact
- Inconsistent developer experience between local and CI
- Requires workarounds that pollute project configuration
- Makes affected detection unpredictable

## Related Documentation
- [Moon CI documentation](https://moonrepo.dev/docs/guides/ci)
- [Moon affected detection](https://moonrepo.dev/docs/commands/ci)

## Status
- **Reported**: 2025-07-25
- **Workaround implemented**: Project-level env variables
- **Upstream report needed**: Should be reported to moonrepo/moon GitHub repository