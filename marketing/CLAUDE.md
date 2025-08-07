# DeepBrain Marketing Decision Management

## Context

This folder manages strategic decisions for DeepBrain Technologies Ltd, a developer tools company progressing from Rust-specific tools to broader AI products.

## File Structure

- `DECISIONS.md` - Main decision tracker with action items
- `STRATEGY.md` - Strategic vision and market progression
- `decisions/` - Individual decision documents

## Decision Workflow

1. **Create**: New decisions get individual `.md` files in `decisions/`
2. **Track**: Add to main `DECISIONS.md` overview table
3. **Link**: Action items link back to source decisions using `[[Decision Topic]](file.md)` format
4. **Status**: ❓ Under Discussion → ✅ Decided

## Key Decisions Made

- **Company**: DeepBrain Technologies Ltd → Inc.
- **Products**: LogFusion, RustToolkit, SuperConfig, DeepBrain Core
- **Strategy**: Hybrid descriptive naming + DeepBrain brand
- **Market**: Rust devs → All devs → AI users

## Action Item Format

```markdown
- [ ] Task description [[Decision Topic]](decisions/file.md)
- [ ] Pending task [Decision Type - TBD]
```

## Navigation

- All decision docs have `[🚪 ← Back to Decisions Overview](../DECISIONS.md)`
- Each decision doc includes action items section at top
- Main tracker shows priority: 🔴 Urgent | 🟡 Important | 🟢 Planning

## When Working Here

1. Update decision statuses in both individual docs AND main tracker
2. Add action items to both decision docs and main DECISIONS.md
3. Use proper linking format for traceability
4. Mark completed items with [x] and strikethrough
5. **ALWAYS reply/suggest/comment based on thoroughly researched information - never make claims about domain availability, technical constraints, or market data without verification**

## Token Efficiency

- **Read selectively**: Full File should be read only once to get full context unless updated.
- **Edit precisely**: Target specific sections, avoid full rewrites
- **Batch operations**: Multiple edits in single MultiEdit calls
- **Stay focused**: Write Concise and accurate information.
