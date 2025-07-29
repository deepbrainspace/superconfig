# WasmBridge vs SuperWASM - Naming Decision

**Created**: July 27, 2025\
**Author**: Claude Opus 4\
**Status**: Naming Strategy
**Purpose**: Decide on WASM tool naming and repository strategy

## 🎯 The Monorepo Stars Question

You're absolutely right - if it's in the SuperConfig monorepo, how would it get independent stars?

### Options for Repository Structure:

#### Option 1: Separate GitHub Repo (If going standalone)

```
github.com/deepbrainspace/superconfig     (Main monorepo)
github.com/deepbrainspace/wasmbridge      (Separate repo)
github.com/deepbrainspace/supercli        (Separate repo)
```

#### Option 2: Keep in Monorepo (RECOMMENDED)

```
github.com/deepbrainspace/superconfig
├── crates/superconfig/      ← Published to crates.io
├── crates/wasmbridge/       ← Published to crates.io separately
└── crates/supercli/         ← Published to crates.io separately
```

**Why monorepo is still better**:

- Unified development and testing
- Share dependencies and CI/CD
- Can still publish to crates.io independently
- Track "success" by crates.io downloads, not GitHub stars

## 🏷️ Naming Analysis: WasmBridge vs SuperWASM

### "WasmBridge"

**Pros**:

- ✅ Descriptive - clearly says what it does
- ✅ Professional/neutral - not tied to Super* brand
- ✅ Could stand alone if needed
- ✅ Good SEO (wasm + bridge are searchable)

**Cons**:

- ❌ No brand connection to SuperConfig
- ❌ Generic sounding

### "SuperWASM"

**Pros**:

- ✅ Brand consistency with SuperConfig
- ✅ Memorable and short
- ✅ Domain available (superwasm.dev)
- ✅ Clear "family" connection

**Cons**:

- ❌ Less descriptive of function
- ❌ Might oversell (is it really "super"?)
- ❌ Locks into Super* branding

### Other Options Considered:

- **"WasmLink"** - Too similar to existing tools
- **"RustBridge"** - Too Rust-specific
- **"UniversalWASM"** - Too grandiose
- **"WasmGen"** - Too generic

## 🎯 Recommendation: Go with "WasmBridge"

### Why WasmBridge Wins:

1. **Clear Functionality**: Immediately tells developers what it does
2. **Professional Appeal**: Sounds like a serious tool, not marketing fluff
3. **SEO Advantage**: "wasm bridge", "rust wasm bridge" are searchable terms
4. **Future Flexibility**: Can stand alone without Super* baggage
5. **No Overselling**: Honest, descriptive name that delivers on promise

### Implementation Strategy:

#### Phase 1: Part of SuperConfig Story

- Package name: `wasmbridge` on crates.io
- Docs at: superconfig.dev/wasmbridge
- Positioned as: "The technology behind SuperConfig's universal language support"
- Tagline: "Bridge your Rust code to any language"

#### Phase 2: Measure Success (3-6 months)

Success metrics:

- Crates.io downloads
- Community asking for standalone features
- Other projects wanting to use it
- Blog posts about it specifically

#### Phase 3: Potential Spinoff (If justified)

Only if:

- 10K+ monthly downloads on crates.io
- Clear demand for standalone tool
- Community contributions coming in
- Makes business sense

## 📦 Technical Architecture for Monorepo

```toml
# /Cargo.toml (workspace root)
[workspace]
members = [
  "crates/superconfig",
  "crates/wasmbridge",
  "crates/supercli",
  "crates/superconfig-cli",
]

# Each can be published independently:
# cargo publish -p superconfig
# cargo publish -p superwasm
# cargo publish -p supercli
```

## 🎬 Marketing Approach

### The Story Arc:

1. **Launch**: "SuperConfig - Universal config management"
2. **Week 3**: "Powered by WasmBridge - our open source WASM tooling"
3. **Month 2**: "WasmBridge - Bridge any Rust library to any language"
4. **Future**: Potential standalone product if demand exists

### Messaging Examples:

> "SuperConfig achieves universal language support through WasmBridge"

> "WasmBridge: Open source tooling for Rust→WASM→Everything"

> "We built WasmBridge to solve our own problem. Now you can use it too."

## 💡 The Real Insight

**Don't optimize for GitHub stars** - optimize for:

1. **Crates.io downloads** (actual usage)
2. **npm downloads** (for bindings)
3. **Revenue** (enterprise adoption)
4. **Community** (contributors, not just stargazers)

## 🎯 Final Decision

1. **Name it**: WasmBridge (not SuperWASM)
2. **Keep it**: In the monorepo for now
3. **Position it**: As professional open source tooling that powers SuperConfig
4. **Measure by**: Downloads and usage, not stars
5. **Spin off**: Only if market demands it

**Domain strategy**:

- Don't buy wasmbridge.dev yet
- superconfig.dev/wasmbridge is perfect for now
- Revisit if it needs standalone identity

## 💡 Why WasmBridge is the Right Choice

You convinced me! WasmBridge is:

- **Honest**: It bridges Rust to other languages via WASM
- **Professional**: Sounds like a tool developers would trust
- **Searchable**: Great for SEO and discoverability
- **Flexible**: Not locked into Super* branding if it goes standalone

The "Super" prefix works for SuperConfig (configuration can be "super"), but for a technical tool, descriptive beats catchy.

---

**Remember**: Turborepo, Nx, Rush all keep tools in monorepos but publish separately. Success isn't measured in repository stars.
