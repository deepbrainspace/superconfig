[🚪 ← Back to Decisions Overview](../DECISIONS.md)

# Meta-Rust Product Naming Decision

**Decision Date**: 2025-08-06\
**Status**: ✅ Decided\
**Meeting Attendees**: Nayeem Syed, Claude Code Opus 4.1

## Executive Summary

**Current Name**: rusttoolkit\
**Leading Candidates**: RustToolkit, RustMacro, MacroForge\
**Decision Needed**: This week (blocks marketing materials)

---

## Current Name Analysis: "rusttoolkit"

### Problems with "rusttoolkit"

- **Hyphenated**: Less professional, harder to brand
- **Too Generic**: "Meta" is overused (Meta/Facebook, metadata, etc.)
- **SEO Challenges**: Competes with many "meta" + "rust" results
- **Unclear Purpose**: Doesn't immediately convey it's for macros/metaprogramming

### What It Does Well

- Clear it's Rust-related
- "Meta" hints at metaprogramming
- Simple and short

---

## Naming Options Evaluation

### Option 1: RustToolkit (CamelCase)

**Pros:**

- ✅ **Minimal Change**: Just remove hyphen
- ✅ **Continuity**: Existing users recognize it
- ✅ **Professional**: CamelCase looks more polished
- ✅ **Broad Scope**: Covers all metaprogramming

**Cons:**

- ❌ Still generic
- ❌ SEO competition remains
- ❌ "Meta" association with Facebook

**Brand Positioning**: "Powerful Rust metaprogramming utilities"

---

### Option 2: MacroForge

**Pros:**

- ✅ **Strong Imagery**: Forging/crafting macros
- ✅ **Memorable**: Unique, stands out
- ✅ **Clear Purpose**: Obviously about macros
- ✅ **Professional**: Sounds enterprise-ready

**Cons:**

- ❌ Complete rebrand needed
- ❌ "Forge" is common in dev tools
- ❌ Limits to just macros (not all metaprogramming)

**Brand Positioning**: "The ultimate Rust macro toolkit"

---

### Option 3: RustMacro

**Pros:**

- ✅ **Crystal Clear**: No ambiguity
- ✅ **SEO Focused**: Rust macro searches find it
- ✅ **Descriptive**: Says exactly what it is

**Cons:**

- ❌ **Too Literal**: No personality
- ❌ **Generic**: Hard to trademark
- ❌ **Limiting**: Only macros, not broader metaprogramming

---

### Option 4: CodeWeaver

**Pros:**

- ✅ **Metaphorical**: Weaving code together
- ✅ **Unique**: Not commonly used
- ✅ **Memorable**: Strong visual

**Cons:**

- ❌ Too abstract
- ❌ Doesn't mention Rust or macros
- ❌ Could be any code tool

---

### Option 5: MetaForge

**Pros:**

- ✅ **Combines Concepts**: Meta + creation
- ✅ **Powerful Sound**: Industrial strength
- ✅ **Broader Scope**: All metaprogramming

**Cons:**

- ❌ Still has "Meta" confusion
- ❌ Many "Forge" products
- ❌ Not Rust-specific

---

### Option 6: ProcMacro (or ProcMacroKit)

**Pros:**

- ✅ **Technical Accuracy**: It's about proc macros
- ✅ **Rust Community Term**: Familiar to users
- ✅ **Clear Purpose**: No confusion

**Cons:**

- ❌ Very technical
- ❌ Limits perception
- ❌ Hard to brand

---

## Opus 4.1 Recommendation

### 🎯 **✅ DECISION MADE: RustToolkit (Keep Current)**

**Final Decision**: Keep "RustToolkit" as product name (no change from rusttoolkit)

**Why RustToolkit won:**

1. **Brand Continuity**: You already have users who know rusttoolkit. Don't lose that recognition.

2. **Scope Advantage**: "Meta" covers ALL metaprogramming - macros, code generation, compile-time computation. MacroForge would limit you.

3. **Minimal Migration**: Just update the styling, not a complete rebrand.

4. **Professional Polish**: RustToolkit looks like a real product. rusttoolkit looks like a GitHub repo.

5. **Future Products**:
   - RustToolkit Core
   - RustToolkit Pro (with IDE integration)
   - RustToolkit Cloud (macro sharing platform?)

### Marketing Positioning with RustToolkit

**Tagline Options:**

- "Powerful Rust metaprogramming made simple"
- "Macros, derives, and beyond"
- "Write code that writes code"

**Key Messages:**

- Comprehensive metaprogramming toolkit
- From simple derives to complex macros
- Battle-tested patterns and utilities

---

## Alternative Opus 4.1 Suggestion

If you want something completely fresh, I'd pick **MacroForge** because:

- It's memorable and unique
- Strong brand potential
- Clear value proposition
- Sounds professional

But I honestly think keeping RustToolkit (just styled better) is the smartest move. You're not fixing a broken name, just polishing it.

---

## Decision Criteria Checklist

| Criteria          | rusttoolkit | RustToolkit | MacroForge | RustMacro | CodeWeaver |
| ----------------- | ----------- | ----------- | ---------- | --------- | ---------- |
| Easy to pronounce | ✅          | ✅          | ✅         | ✅        | ✅         |
| Memorable         | 🟡          | ✅          | ✅         | 🟡        | ✅         |
| Descriptive       | ✅          | ✅          | ✅         | ✅        | ❌         |
| SEO potential     | 🟡          | 🟡          | ✅         | ✅        | ❌         |
| Unique            | ❌          | 🟡          | ✅         | ❌        | ✅         |
| Scalable brand    | 🟡          | ✅          | ✅         | ❌        | 🟡         |
| Professional      | ❌          | ✅          | ✅         | 🟡        | ✅         |
| Continuity        | ✅          | ✅          | ❌         | ❌        | ❌         |

**Winner: RustToolkit** (7/8 criteria met + continuity bonus)

---

## Implementation Plan

If we choose RustToolkit:

1. **Minimal Changes Needed**:
   - Update Cargo.toml name field
   - Update documentation headers
   - Keep crates.io slug as rusttoolkit (for compatibility)

2. **Marketing Materials**:
   - Always use "RustToolkit" in writing
   - Logo uses CamelCase
   - Keep GitHub URL as-is for now

---

## Opus 4.1 Final Thoughts

This is a much easier decision than logffi. You have a decent name that just needs polish. Don't overthink it - RustToolkit is fine. Save your energy for the bigger battles (like domain strategy).

The hyphen → CamelCase change is enough to look professional without confusing existing users.

---

_Document created: 2025-08-06_\
_Last updated: 2025-08-06_\
_Next review: Upon decision_
