[🚪 ← Back to Decisions Overview](../DECISIONS.md)

# Company Naming & LogFFI Renaming Decision

**Decision Date**: 2025-08-06\
**Status**: ✅ Decided\
**Meeting Attendees**: Nayeem Syed, Claude Code Opus 4.1\
**Priority**: 🔴 URGENT (blocks incorporation & product launch)

## 🎯 Action Items from This Decision

### 🔴 This Week (Urgent)

- [ ] Register crates.io names (logfusion, metarust)
- [ ] Reserve domains (logfusion.dev, metarust.com if available)
- [ ] Begin codebase rename: logffi → logfusion

### 🟡 Next 2 Weeks (Important)

- [ ] Complete codebase updates (Cargo.toml, docs, README)
- [ ] Prepare product rename announcement posts

---

## Part 1: Company Naming Decision

### Context

- Existing DeepBrain AI company exists (different AI focus, no direct competition)
- Need to incorporate UK Ltd immediately, then Delaware C-Corp later

### Options Analysis

#### Option 1: DeepBrain Ltd → DeepBrain Inc.

**Pros:**

- ✅ Clean, simple progression
- ✅ Strong brand consistency
- ✅ Domain matches exactly (deepbrain.dev)

**Cons:**

- ❌ Potential confusion with other DeepBrain AI
- ❌ May need disambiguation in marketing

#### Option 2: DeepBrain.Space Ltd → DeepBrain.Space Inc.

**Pros:**

- ✅ Unique, no conflicts
- ✅ .space TLD reinforces uniqueness
- ✅ Could use for creative campaigns

**Cons:**

- ❌ Awkward in conversation ("DeepBrain dot Space Limited")
- ❌ Looks unusual on legal documents
- ❌ Email addresses become long

#### Option 3: DeepBrain Technologies Ltd → Inc. (RECOMMENDED)

**Pros:**

- ✅ Differentiates from other DeepBrain
- ✅ Common, trusted pattern
- ✅ Room for product lines
- ✅ Professional for B2B

**Cons:**

- ❌ Slightly longer
- ❌ "Technologies" is generic

### ✅ DECISION MADE: **DeepBrain Technologies Ltd**

**Final Decision**: DeepBrain Technologies Ltd → DeepBrain Technologies Inc.

**Reasoning:**

1. Avoids conflict with dissolved "DeepBrain Ltd" on UK Companies House
2. "Technologies" signals broader scope than just AI
3. Natural progression to "DeepBrain Technologies Inc." in Delaware
4. Can still use "DeepBrain" in marketing materials
5. No risk with Companies House due to previous dissolution

---

## Part 2: LogFFI Alternative Names

### Requirements Based on Your Insights

1. **Must emphasize**: define_errors! macro (killer feature)
2. **Should convey**: tracing/thiserror compatibility
3. **De-emphasize**: FFI aspects (niche use case)
4. **Must be available**: On crates.io

### Brainstormed Names (Verified Availability)

#### 🎯 Top Tier - Best Options

**1. ErrorForge** ✅ AVAILABLE

- **Why it wins**: Directly emphasizes your killer feature (define_errors!)
- **Brand story**: "Forge perfect Rust errors with zero boilerplate"
- **SEO potential**: Owns "rust error handling" searches
- **Memorable**: Strong verb + clear purpose
- **Growth potential**: ErrorForge Pro, ErrorForge Cloud

**2. LogCore** ✅ AVAILABLE

- **Why it works**: Positions as foundational logging
- **Professional**: Sounds like essential infrastructure
- **Clear**: Core logging functionality
- **Flexible**: LogCore, LogCore Pro, LogCore Studio

**3. TraceFusion** ✅ AVAILABLE

- **Why it works**: Fuses tracing + logging perfectly
- **Modern**: Fusion implies advanced tech
- **Memorable**: Two concepts merged
- **Unique**: No conflicts on crates.io

**4. DefineKit** ✅ AVAILABLE

- **Why it works**: References define_errors! directly
- **Developer-friendly**: -Kit suffix familiar (UIKit, ARKit)
- **Clear purpose**: Kit for defining errors/logs
- **Expandable**: Can add more define_* macros

#### Second Tier - Good Alternatives

**5. LogBridge** ✅ AVAILABLE

- Bridges different logging systems
- Professional architectural metaphor
- Clear conceptual model

**6. ErrorKit** ✅ AVAILABLE

- Focus on error handling
- Simple, memorable
- Professional suffix

**7. RustLog** ✅ AVAILABLE

- Crystal clear purpose
- Great for SEO
- Maybe too generic

**8. UniLog** ✅ AVAILABLE (Note: not "uniflog")

- Unified logging
- Short, punchy
- Easy to remember

**9. LogFusion** ✅ AVAILABLE

- Merging logging libraries
- Action-oriented
- Modern feel

**10. TraceKit** ✅ AVAILABLE

- Complete tracing toolkit
- Professional sound
- Clear purpose

#### Names to Avoid (Already Taken)

- ❌ **ErrLog** - Similar "errlog" exists
- ❌ **LogKit** - Already taken (found in search)
- ❌ **TraceLog** - "tracelogging" exists (too similar)
- ❌ **LogR/RLog** - You mentioned not available

### Availability Matrix & Analysis

| Name           | crates.io | .dev domain | SEO Potential | Memorability | Clarity |
| -------------- | --------- | ----------- | ------------- | ------------ | ------- |
| **ErrorForge** | ✅        | Check       | High          | High         | High    |
| **LogFusion**  | ✅        | Check       | Medium        | High         | Medium  |
| **TraceKit**   | ✅        | Check       | High          | High         | High    |
| **ErrLog**     | ✅        | Check       | Medium        | Very High    | High    |
| **LogBridge**  | ✅        | Check       | Medium        | Medium       | High    |

### ✅ DECISION MADE: **LogFusion**

**Final Decision**: logffi → LogFusion

**Why LogFusion Won:**

- **Perfect metaphor**: Fuses tracing + thiserror + FFI + custom tools
- **Memorable**: Easy to say, spell, and remember
- **Action-oriented**: "Fusion" implies combining powerful elements
- **Modern feel**: Sounds advanced and sophisticated
- **Clear value prop**: "Fusing all your Rust logging needs"
- **Available**: Confirmed available on crates.io

**Marketing Positioning:**

- **Tagline**: "Fusing Rust logging into one powerful solution"
- **Key message**: Combines tracing, thiserror, FFI, and custom error handling
- **Value prop**: One dependency instead of three

---

## Part 3: Updated Product Portfolio

Based on all discussions and your strategic insights:

### Rust-Specific Products (Accept limited TAM, community credibility builders)

1. **LogFusion** (formerly logffi) - ✅ DECIDED
   - **Primary value**: Fuses multiple logging systems
   - **Key features**: define_errors! macro, tracing/thiserror unity, FFI support
   - **Marketing**: "One dependency instead of three"
   - **Market**: ~500K Rust developers
   - **Lifespan**: Rust ecosystem only

2. **RustToolkit** (keep current name) - ✅ DECIDED
   - **Rationale**: crates.io name available, possible .com domain
   - **Clear positioning**: Rust-specific macro utilities
   - **Market**: Rust developers needing macro tools
   - **Lifespan**: Rust ecosystem only

### Cross-Language Products (Broader market potential)

3. **SuperConfig** (keep as-is)
   - **Already strong name**: No change needed
   - **Multi-language**: Your bridge to wider dev community
   - **Market**: ~30M developers globally

4. **DeepBrain Core** (AI product)
   - **Flagship product**: Broadest market appeal
   - **Beyond developers**: Business users, researchers
   - **Market**: Anyone using AI

### ✅ FINAL DECIDED PRODUCT SUITE

```
DeepBrain Technologies Ltd
├── LogFusion - "Fusing Rust logging into one"
├── RustToolkit - "Rust metaprogramming unleashed"  
├── SuperConfig - "Configuration that scales"
└── DeepBrain Core - "AI with perfect memory"
```

### Why This Portfolio Works:

1. **Progressive Market Expansion**:
   - Start: Rust community (ErrorForge + RustToolkit)
   - Expand: All developers (SuperConfig)
   - Scale: Everyone (DeepBrain Core)

2. **Clear Product Identity**:
   - Each name describes function
   - No forced naming conventions
   - Professional, memorable

3. **Strategic Flexibility**:
   - Can pivot individual products
   - Can acquire without rebranding
   - Can spin off if needed

---

## Implementation Plan

### Week 1: Decisions & Registration

1. **Day 1**: Decide on company name
2. **Day 2**: Register "DeepBrain Technologies Ltd" in UK
3. **Day 3**: Check ErrorForge availability thoroughly
4. **Day 4**: Reserve crates.io names
5. **Day 5**: Register any needed domains

### Week 2: Rebranding

1. Update codebases with new names
2. Prepare migration guides
3. Update documentation
4. Create announcement posts

---

## ✅ Decisions Finalized

- [x] **Company name**: DeepBrain Technologies Ltd → DeepBrain Technologies Inc.
- [x] **LogFFI → LogFusion**: Perfect metaphor for fusing logging systems
- [x] **Keep RustToolkit**: Current name stays (crates.io available)
- [x] **Keep SuperConfig**: No change needed
- [ ] **Register domains**: logfusion.dev, metarust.com (if available)

---

_Document created: 2025-08-06_\
_Last updated: 2025-08-06_\
_Next review: URGENT - Blocks incorporation_
