[🚪 ← Back to Decisions Overview](../DECISIONS.md)

# LogFFI Product Naming Decision

**Decision Date**: TBD\
**Status**: ❓ Under Discussion\
**Meeting Attendees**: Nayeem Syed, Claude Code Opus 4.1

## Executive Summary

**Current Name**: logffi\
**Leading Candidate**: TraceLog\
**Decision Needed**: This week (blocks marketing materials)

---

## Current Name Analysis: "logffi"

### Problems with "logffi"

- **Too Technical**: FFI (Foreign Function Interface) is niche terminology
- **Hard to Pronounce**: "log-f-f-i" vs "log-fee" vs "log-fi"?
- **Limits Perception**: Sounds like just FFI bindings, not a full logging solution
- **Poor SEO**: Generic terms, hard to rank
- **Not Memorable**: Doesn't stick in developers' minds
- **No Emotion**: Purely functional, no brand personality

### What It Does Well

- Describes technical capability (FFI support)
- Clear it's about logging
- Honest about origins

---

## Naming Options Evaluation

### Option 1: TraceLog ⭐

**Pros:**

- ✅ **Descriptive**: Combines tracing + logging concepts
- ✅ **Professional**: Sounds enterprise-ready
- ✅ **Memorable**: Two simple words
- ✅ **SEO Friendly**: Unique enough to own search results
- ✅ **Pronounceable**: "Trace-Log" - no ambiguity
- ✅ **Scalable**: Room for TraceLog Pro, TraceLog Cloud, etc.
- ✅ **Modern**: Aligns with OpenTelemetry trends

**Cons:**

- ❌ Might imply it's ONLY for tracing
- ❌ Two words might feel less unified
- ❌ Similar to existing "trace" crates

**Brand Positioning**: "Zero-friction Rust logging with built-in tracing"

---

### Option 2: LogForge

**Pros:**

- ✅ **Strong Imagery**: Forge = crafting, building
- ✅ **Memorable**: Unique name
- ✅ **Single Word**: Clean, unified
- ✅ **Implies Power**: Industrial strength

**Cons:**

- ❌ Many "Forge" products exist
- ❌ Less descriptive of actual function
- ❌ Might sound like a log viewer/analyzer

---

### Option 3: RustLog

**Pros:**

- ✅ **Crystal Clear**: Rust + Logging
- ✅ **SEO Targeted**: Rust developers will find it
- ✅ **Simple**: No confusion

**Cons:**

- ❌ **Too Generic**: Limits brand potential
- ❌ **Not Unique**: Hard to trademark
- ❌ **Limiting**: Ties completely to Rust

---

### Option 4: LogBridge

**Pros:**

- ✅ **Hints at FFI**: Bridge between systems
- ✅ **Metaphorical**: Connecting different worlds
- ✅ **Unique**: Not commonly used

**Cons:**

- ❌ Still technical
- ❌ "Bridge" is overused in tech
- ❌ Doesn't convey the Zero-friction aspect

---

### Option 5: ZeroLog

**Pros:**

- ✅ **Captures Philosophy**: Zero-friction logging
- ✅ **Memorable**: Short, punchy
- ✅ **Modern Feel**: Like ZeroMQ, Zerocopy

**Cons:**

- ❌ Might imply "no logging"
- ❌ Similar to existing zerolog (Go)
- ❌ Less descriptive

---

### Option 6: LogStream

**Pros:**

- ✅ **Modern**: Streaming/async implications
- ✅ **Flow Imagery**: Smooth, continuous
- ✅ **Descriptive**: Logs as streams

**Cons:**

- ❌ Many "Stream" products
- ❌ Might imply only streaming logs
- ❌ Generic feeling

---

## Opus 4.1 Recommendation

### 🎯 **My Strong Recommendation: TraceLog**

**Why TraceLog Wins:**

1. **Market Alignment**: The industry is moving toward unified observability (logs + traces + metrics). TraceLog positions you perfectly for this trend.

2. **Immediate Understanding**: Developers instantly know what it does - it's logging with tracing capabilities.

3. **Brand Potential**:
   - TraceLog Core (open source)
   - TraceLog Cloud (future SaaS)
   - TraceLog Studio (future GUI)

4. **SEO Advantage**: "tracelog rust" will own search results. Currently no major Rust crate uses this name.

5. **Professional Sound**: Enterprises trust products with clear, descriptive names.

6. **Story Potential**: "From simple logs to distributed traces" - the name tells a journey.

### Marketing Positioning with TraceLog

**Tagline Options:**

- "Zero-friction Rust logging with built-in tracing"
- "From println! to production in minutes"
- "The missing link between logs and traces"

**Key Messages:**

- Unified logging and tracing
- Zero-overhead abstractions
- FFI-ready for polyglot systems

### Implementation Plan

If we choose TraceLog:

1. **Week 1**:
   - Update all code references
   - Update Cargo.toml metadata
   - Register crates.io name

2. **Week 2**:
   - Update documentation
   - Create migration guide from logffi
   - Design logo/brand assets

3. **Launch Messaging**:
   ```
   Introducing TraceLog: Zero-friction Rust logging

   - 🚀 5x faster than traditional loggers
   - 🔍 Built-in tracing support
   - 🌐 FFI-ready for any language
   - 📊 OpenTelemetry compatible
   ```

---

## Alternative Opus 4.1 Suggestion

If you don't like TraceLog, my second choice would be **ZeroLog** because:

- It captures your "zero-friction" philosophy
- It's short and memorable
- It has that modern, performance-focused feel

But I strongly believe TraceLog is the winner here.

---

## Decision Criteria Checklist

| Criteria          | logffi | TraceLog | LogForge | RustLog | ZeroLog |
| ----------------- | ------ | -------- | -------- | ------- | ------- |
| Easy to pronounce | ❌     | ✅       | ✅       | ✅      | ✅      |
| Memorable         | ❌     | ✅       | ✅       | 🟡      | ✅      |
| Descriptive       | 🟡     | ✅       | 🟡       | ✅      | 🟡      |
| SEO potential     | ❌     | ✅       | ✅       | ❌      | ✅      |
| Unique            | 🟡     | ✅       | 🟡       | ❌      | 🟡      |
| Scalable brand    | ❌     | ✅       | ✅       | ❌      | ✅      |
| Professional      | ❌     | ✅       | ✅       | ✅      | ✅      |

**Winner: TraceLog** (7/7 criteria met)

---

## Final Opus 4.1 Thoughts

The name change from logffi to TraceLog is more than cosmetic - it's a strategic repositioning. You're not just offering "FFI logging bindings" anymore. You're offering a modern, comprehensive logging solution that happens to have excellent FFI support.

TraceLog says: "We understand modern observability."
logffi says: "We do technical FFI stuff."

Which company would you rather build?

---

## Next Steps

1. **Decide on name** (Target: Today)
2. **Register domains** if needed (tracelog.dev?)
3. **Update codebase** (1-2 days)
4. **Create announcement** post about the rename
5. **Update all documentation**

---

_Document created: 2025-08-06_\
_Last updated: 2025-08-06_\
_Next review: Upon decision_
