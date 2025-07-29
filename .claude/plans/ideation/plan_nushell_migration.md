# Nushell Migration Plan for SuperConfig Development

## Project Overview

Migrate SuperConfig development workflow from bash to nushell to enable AI-integrated development with structured data handling and improved developer experience.

## Vision: AI-First Development Environment

- **Structured Data Throughout**: Every command output is queryable, pipeable data
- **AI-Friendly Scripting**: Natural language-like commands that AI can generate reliably
- **Configuration Synergy**: Nushell's data model aligns perfectly with SuperConfig's multi-format philosophy
- **Rich Development Context**: Build outputs, test results, and metrics as structured data for AI analysis

## Phase 1: Foundation Setup (1-2 hours)

### 1.1 Install and Configure Nushell

- [ ] **Install nushell**: `cargo install nu` (from your fork if needed)
- [ ] **Set as default shell**: `chsh -s $(which nu)` or configure terminal
- [ ] **Create config directory**: `mkdir -p ~/.config/nushell`
- [ ] **Basic configuration**: Set up `config.nu` and `env.nu`

### 1.2 Essential Configuration

```nu
# ~/.config/nushell/config.nu
$env.config = {
    show_banner: false
    table: {
        mode: rounded
    }
    completions: {
        case_sensitive: false
        quick: true
    }
    filesize: {
        metric: false
        format: "auto"
    }
}

# Rust development aliases
alias cb = cargo build --all-features
alias ct = cargo test
alias cc = cargo check --workspace
alias cn = cargo nextest run
```

### 1.3 Verify Installation

- [ ] **Test basic commands**: `ls`, `ps`, `sys`
- [ ] **Test JSON handling**: `cargo metadata | from json | get packages.0.name`
- [ ] **Test file operations**: `ls **/*.rs | where size > 10kb`

## Phase 2: Development Workflow Migration (2-3 hours)

### 2.1 Replace Common Bash Commands

**File Operations:**

```nu
# Instead of: find . -name "*.rs" -type f
glob **/*.rs

# Instead of: find . -name "*.rs" -exec grep -l "pattern" {} \;
glob **/*.rs | each { |file| if (open $file | str contains "pattern") { $file } } | compact

# Instead of: wc -l **/*.rs
glob **/*.rs | each { |file| {file: $file, lines: (open $file | lines | length)} }
```

**Git Operations:**

```nu
# Rich git log analysis
git log --oneline -n 20 | lines | each { |line| $line | parse "{hash} {message}" } | flatten

# Analyze commit patterns
git log --pretty=format:"%h,%an,%s" | from csv | group-by column2 | transpose key value | sort-by value
```

### 2.2 Create justfile with Nushell Integration

- [ ] **Install just**: `cargo install just`
- [ ] **Create initial justfile**:

```justfile
# Set nushell as shell for complex commands
set shell := ["nu", "-c"]

# Simple commands stay simple
build:
    cargo build --all-features

check:
    cargo check --workspace

test:
    cargo nextest run

# AI-friendly structured commands
analyze-code:
    glob **/*.rs | each { |f| {file: $f.name, size: $f.size, lines: (open $f | lines | length)} } | sort-by size --reverse | first 10

analyze-deps:
    cargo metadata --format-version=1 | from json | get packages | select name version | sort-by name

analyze-tests:
    cargo nextest run --message-format=json | from json | where type == "test" | group-by outcome | transpose outcome count

ci-report:
    {
        build: (cargo build --all-features 2>&1 | complete),
        tests: (cargo nextest run --message-format=json | from json | where type == "test" | length),
        clippy: (cargo clippy --message-format=json 2>&1 | from json | where level == "warning" | length),
        features: (cargo metadata | from json | get packages | where name == "superconfig" | get 0.features | transpose feature deps)
    } | to yaml

# SuperConfig specific tasks
validate-configs:
    ["examples/config.toml", "examples/config.json", "examples/config.yaml"]
    | each { |file|
        {
            file: $file,
            valid: (cargo run -- validate $file | complete | get exit_code) == 0
        }
    }

benchmark-formats:
    ["toml", "json", "yaml"]
    | each { |fmt|
        {
            format: $fmt,
            parse_time: (cargo bench --bench format_bench -- $fmt | parse "time: {time}" | get time.0)
        }
    }
```

### 2.3 Update CLAUDE.md Rules

- [ ] **Add nushell preferences** to package manager section
- [ ] **Update command examples** to use nushell syntax where beneficial
- [ ] **Add structured output guidelines** for AI integration

## Phase 3: AI Integration Features (3-4 hours)

### 3.1 Structured Development Commands

**Create `~/.config/nushell/superconfig-dev.nu`:**

```nu
# AI-friendly development utilities for SuperConfig

# Analyze codebase structure for AI context
export def analyze-codebase [] {
    {
        overview: {
            total_files: (glob **/*.rs | length),
            total_lines: (glob **/*.rs | each { |f| open $f | lines | length } | math sum),
            main_modules: (ls src/ | where type == dir | get name)
        },
        dependencies: (cargo metadata | from json | get packages | where name == "superconfig" | get 0.dependencies | select name req),
        features: (cargo metadata | from json | get packages | where name == "superconfig" | get 0.features | transpose feature deps),
        recent_changes: (git log --oneline -n 5 | lines | each { |line| $line | parse "{hash} {message}" } | flatten)
    }
}

# Generate AI context for current work
export def ai-context [] {
    {
        current_branch: (git branch --show-current),
        staged_files: (git diff --cached --name-only | lines),
        modified_files: (git diff --name-only | lines),
        build_status: (cargo check --workspace 2>&1 | complete),
        test_status: (cargo nextest run --no-run 2>&1 | complete),
        recent_commits: (git log --oneline -n 3 | lines)
    } | to json
}

# Analyze test coverage and suggest improvements
export def analyze-tests [] {
    let test_files = (glob tests/**/*.rs src/**/*test*.rs)
    let src_files = (glob src/**/*.rs | where $it !~ test)

    {
        coverage: {
            test_files: ($test_files | length),
            src_files: ($src_files | length),
            ratio: (($test_files | length) / ($src_files | length))
        },
        test_results: (cargo nextest run --message-format=json | from json | where type == "test" | group-by outcome),
        suggestions: (
            $src_files
            | each { |f|
                let test_exists = ($test_files | any { |t| ($t | path basename | str starts-with ($f | path basename | str replace ".rs" "")) })
                if not $test_exists { $f }
            }
            | compact
        )
    }
}

# Performance benchmarking with structured output
export def benchmark-features [] {
    ["core", "cli", "mcp", "api"]
    | each { |feature|
        {
            feature: $feature,
            build_time: (time (cargo build --features $feature) | get real),
            binary_size: (ls target/debug/superconfig* | where name =~ $feature | get size | first)
        }
    }
}
```

### 3.2 Integration with Development Tools

**VS Code/Cursor Integration:**

```nu
# Generate project context for AI coding assistants
export def cursor-context [] {
    let context = (ai-context)
    $context | save .cursor-context.json
    print "AI context saved to .cursor-context.json"
}

# Generate structured commit messages
export def ai-commit [] {
    let changes = {
        staged: (git diff --cached --stat | lines),
        files: (git diff --cached --name-only | lines),
        diff: (git diff --cached)
    }

    print "=== Changes to commit ==="
    $changes.staged | each { |line| print $line }
    print "\n=== AI Analysis Context ==="
    $changes | to json | save .commit-context.json
    print "Context saved to .commit-context.json for AI analysis"
}
```

### 3.3 Moon Integration (Optional)

- [ ] **Compare Moon vs justfile** for SuperConfig's needs
- [ ] **Create hybrid approach** if beneficial:

```justfile
# Use Moon for complex multi-language builds (future WASM)
moon-build:
    moon run superconfig:build

# Use justfile for simple Rust-only tasks
build:
    cargo build --all-features

# Use nushell for AI integration
ai-analysis:
    nu -c "use superconfig-dev.nu; analyze-codebase | to yaml"
```

## Phase 4: Advanced Workflows (2-3 hours)

### 4.1 Configuration Testing Workflows

```nu
# Test SuperConfig across all supported formats
export def test-all-formats [] {
    let formats = ["toml", "json", "yaml", "ron"]
    let test_configs = (ls examples/ | where name =~ config | get name)

    $formats | each { |fmt|
        let configs = ($test_configs | where $it =~ $fmt)
        {
            format: $fmt,
            config_count: ($configs | length),
            validation_results: ($configs | each { |cfg|
                {
                    file: $cfg,
                    valid: (cargo run -- validate $cfg | complete | get exit_code) == 0,
                    parse_time: (time (cargo run -- parse $cfg) | get real)
                }
            })
        }
    }
}

# Multi-language binding preparation (future)
export def prepare-bindings [] {
    {
        rust_ready: (cargo build --all-features | complete | get exit_code) == 0,
        wasm_target: (rustup target list --installed | lines | any { |t| $t == "wasm32-unknown-unknown" }),
        bindgen_deps: (cargo tree | lines | any { |line| $line =~ "bindgen" })
    }
}
```

### 4.2 Release Automation

```nu
export def release-check [] {
    {
        version: (cargo metadata | from json | get packages | where name == "superconfig" | get 0.version),
        git_status: (git status --porcelain | lines | length) == 0,
        tests_passing: (cargo nextest run --no-fail-fast | complete | get exit_code) == 0,
        docs_building: (cargo doc --no-deps | complete | get exit_code) == 0,
        benchmarks: (cargo bench --no-run | complete | get exit_code) == 0
    }
}
```

## Phase 5: Documentation and Polish (1-2 hours)

### 5.1 Update Documentation

- [ ] **Update README** with nushell workflow examples
- [ ] **Create nushell usage guide** in docs/
- [ ] **Document AI integration features**

### 5.2 Share Configuration

- [ ] **Export nushell config** to version control:

```nu
# Create shareable config
{
    config: (open ~/.config/nushell/config.nu),
    env: (open ~/.config/nushell/env.nu),
    custom_commands: (open ~/.config/nushell/superconfig-dev.nu)
} | to json | save .nushell-config.json
```

## Success Criteria

### Phase 1 Success:

- [ ] **Nushell installed and configured** as primary shell
- [ ] **Basic commands working**: file operations, git, cargo
- [ ] **JSON/YAML parsing** working seamlessly

### Phase 2 Success:

- [ ] **All bash scripts replaced** with nushell equivalents
- [ ] **justfile integration** working smoothly
- [ ] **Development workflow** faster than before

### Phase 3 Success:

- [ ] **AI context generation** working
- [ ] **Structured data workflows** providing rich insights
- [ ] **Integration with coding tools** functional

### Phase 4 Success:

- [ ] **SuperConfig-specific workflows** implemented
- [ ] **Multi-format testing** automated with rich reporting
- [ ] **Release automation** with comprehensive checks

## Long-term Vision

### AI-Enhanced Development Loop:

1. **Context Generation**: `ai-context` provides rich structured data to AI
2. **Code Analysis**: `analyze-codebase` gives AI architectural understanding
3. **Test Insights**: `analyze-tests` identifies coverage gaps
4. **Performance Tracking**: `benchmark-features` monitors optimization opportunities
5. **Release Readiness**: `release-check` ensures quality gates

### Integration Points:

- **Claude Code**: Rich context from structured commands
- **Cursor**: Project state via `.cursor-context.json`
- **CI/CD**: Structured reports for automated decision making
- **Documentation**: Auto-generated insights from codebase analysis

## Next Steps

1. **Start with Phase 1**: Basic nushell setup and configuration
2. **Gradual Migration**: Replace one workflow at a time
3. **Measure Impact**: Compare productivity before/after
4. **Iterate**: Refine based on actual usage patterns
5. **Share**: Document learnings for future projects

---

**Timeline Estimate**: 8-12 hours total across 1-2 weeks
**Risk Level**: Low - can run parallel to existing bash workflows
**Expected Benefits**:

- 40% faster common operations through structured data
- 60% better AI integration through rich context
- 30% reduction in debugging time through better error handling
- Foundation for advanced SuperConfig features (multi-language bindings, AI-powered config generation)

_This plan transforms SuperConfig development into an AI-first, data-structured workflow that scales with the project's multi-language ambitions._
