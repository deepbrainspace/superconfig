# FFI Tool Approach: Enhance Existing vs Build New

**Created**: July 27, 2025  
**Author**: Claude Opus 4  
**Status**: Technical Strategy Analysis  
**Decision**: Evaluate enhancing napi-rs vs wasm-pack vs new tool

## 🎯 The Core Question

Should we:
1. **Enhance napi-rs** to support multi-language FFI?
2. **Enhance wasm-pack** to support FFI alongside WASM?
3. **Build new tool** that orchestrates existing tools?

## 📊 Option 1: Enhance napi-rs

### Current napi-rs
- **Purpose**: Rust → Node.js bindings
- **Maintainer**: Active (Vercel team)
- **Used by**: swc, rspack, lightningcss
- **Scope**: Node.js only

### Enhancement Vision
```bash
# Current
napi build --platform

# Enhanced
napi build --platform --target python  # New!
napi build --platform --target ruby    # New!
```

### Pros
✅ Already handles Node.js perfectly  
✅ Active maintenance and community  
✅ Could contribute back to ecosystem  
✅ Proven architecture  

### Cons
❌ Major scope change for project  
❌ Maintainers might reject PR  
❌ Node.js-centric design decisions  
❌ Would need significant refactoring  

## 📊 Option 2: Enhance wasm-pack

### Current wasm-pack
- **Purpose**: Rust → WASM → JS
- **Maintainer**: Rust WASM team
- **Scope**: WASM + JavaScript

### Enhancement Vision
```bash
# Current
wasm-pack build --target nodejs

# Enhanced
wasm-pack build --target nodejs --ffi  # Use FFI instead!
wasm-pack build --target python --ffi  # New!
```

### Pros
✅ Already handles packaging/publishing  
✅ Knows about npm ecosystem  
✅ Could add FFI as alternative to WASM  

### Cons
❌ Conceptually weird (wasm-pack without WASM?)  
❌ Would confuse existing users  
❌ WASM-first architecture  
❌ Probably won't be accepted upstream  

## 📊 Option 3: New Orchestrator Tool

### Vision: "uniffi" or "polyglot" or "omni"
```bash
# Uses existing tools under the hood
omni build --target node   # Calls napi-rs
omni build --target python # Calls maturin/PyO3
omni build --target ruby   # Calls magnus
```

### Architecture
```
omni/
├── Orchestrates
│   ├── napi-rs (for Node.js)
│   ├── maturin (for Python)
│   ├── magnus (for Ruby)
│   └── wasm-pack (for web/WASM fallback)
└── Provides
    ├── Unified config (via SuperConfig!)
    ├── Cross-platform builds
    └── Publishing automation
```

### Pros
✅ Uses best tool for each language  
✅ No need to reinvent wheels  
✅ Can start simple, grow over time  
✅ Clear value proposition  
✅ Respects existing ecosystems  

### Cons
❌ Another tool to maintain  
❌ Dependency on multiple tools  
❌ Need to learn each underlying tool  

## 🎯 Recommendation: Hybrid Approach

### Phase 1: Direct Integration (Week 1-2)
**Just use napi-rs directly in SuperConfig**
```toml
# crates/superconfig/Cargo.toml
[features]
nodejs = ["napi", "napi-derive"]

[dependencies]
napi = { version = "2", optional = true }
napi-derive = { version = "2", optional = true }
```

**Benefits**:
- Ship Node.js bindings NOW
- Learn the patterns
- No new tools needed

### Phase 2: Evaluate Need (Week 3-4)
After shipping Node.js bindings:
- How painful was it?
- What patterns emerged?
- Is automation worth it?

### Phase 3: Build If Needed (Month 2+)
If we need multiple languages:
- Build thin orchestrator
- Focus on YOUR specific needs
- Open source if useful

## 💡 The Real Insight

**Don't build tools until you feel the pain**

1. **Use napi-rs directly** for Node.js NOW
2. **Use maturin/PyO3** for Python when needed
3. **Only build tool** if pattern is clear

## 🚀 Immediate Action Plan

### This Week
1. Add napi-rs to superconfig crate
2. Create Node.js bindings
3. Publish to npm
4. Document the process

### Next Week
1. If painful → design tool
2. If easy → just do Python too
3. Let experience guide architecture

## 📝 Example: Direct napi-rs Usage

```rust
// crates/superconfig/src/node.rs
#[cfg(feature = "nodejs")]
use napi_derive::napi;

#[cfg(feature = "nodejs")]
#[napi]
pub struct NodeSuperConfig {
    inner: crate::SuperConfig,
}

#[cfg(feature = "nodejs")]
#[napi]
impl NodeSuperConfig {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { inner: crate::SuperConfig::new() }
    }
    
    #[napi]
    pub fn with_file(&mut self, path: String) -> napi::Result<()> {
        self.inner = self.inner.clone().with_file(path);
        Ok(())
    }
}
```

## 🎬 Revised Naming Strategy

Forget "WasmBridge" - if we build a tool later:
- **"OmniBind"** - Binds to all languages
- **"PolyBind"** - Polymorphic bindings
- **"UniBind"** - Universal bindings
- **"FFI Bridge"** - Descriptive
- **"Rust Bridge"** - Clear purpose

But first: **Just ship with napi-rs!**

---

**Bottom Line**: Start simple. Use napi-rs directly. Build tools only when the pain justifies it.