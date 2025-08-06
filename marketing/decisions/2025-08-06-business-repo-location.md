[🚪 ← Back to Decisions Overview](../DECISIONS.md)

# Business Documentation Repository Location Decision

**Decision Date**: 2025-08-06\
**Status**: ✅ Decided\
**Meeting Attendees**: Nayeem Syed, Sonnet 4\
**Priority**: 🔴 Urgent (affects document organization strategy)

## 🎯 Action Items from This Decision

### 🔴 This Week (Urgent)

- [ ] Create `deepbrain/business` repository
- [ ] Move `/marketing/` folder from superconfig repo to business repo
- [ ] Update documentation links and references
- [ ] Continue business decisions in dedicated business repository

---

## Decision Context

**Current State**: Business and marketing decisions stored in `/superconfig/marketing/`

**Problem**: Business strategy documents mixed with product-specific code repository

**Need**: Clean separation of business strategy from product development

---

## Decision Analysis

### Issues with Current Location

- **Product coupling**: Business decisions tied to SuperConfig repository
- **Repository restructuring risk**: Business docs affected by product repo changes
- **Cross-product decisions**: Company decisions shouldn't live in single product repo
- **Team clarity**: Mixed business and technical concerns in same repository

### Options Considered

**Option 1: Keep in SuperConfig**

- ❌ Business decisions tied to single product
- ❌ Affected by product repository restructuring
- ❌ Poor separation of concerns

**Option 2: Wait for Main DeepBrain Infrastructure Repo**

- ❌ Delays clean organization
- ❌ Infrastructure repo timeline unclear
- ❌ Continues current mixing of concerns

**Option 3: Create Dedicated Business Repository**

- ✅ Clean separation of business vs product development
- ✅ Survives product repository restructuring
- ✅ Central location for cross-product business decisions
- ✅ Clear ownership and access control
- ✅ Independent evolution of business strategy

---

## Final Decision

**Decision**: Create separate `deepbrain/business` repository for all business and marketing decisions

**Implementation Strategy:**

1. **Create Repository**: `github.com/deepbrain/business` (or similar naming)
2. **Migrate Content**: Move entire `/marketing/` folder from superconfig
3. **Update Links**: Fix cross-references and documentation links
4. **Establish Process**: All future business decisions go in business repo

**Repository Structure:**

```
deepbrain/business/
├── decisions/                    # Strategic business decisions
│   ├── DECISIONS.md             # Main tracker (moved from /marketing/)
│   ├── 2025-08-06-*.md          # Individual decision documents
│   └── ...
├── strategy/                    # Business strategy documents
├── legal/                       # Legal documents and compliance
├── branding/                    # Brand guidelines and assets
├── market-analysis/             # Market research and analysis
└── README.md                    # Repository overview and navigation
```

**Benefits:**

- **Separation of Concerns**: Business strategy separate from product code
- **Repository Independence**: Unaffected by product repo restructuring
- **Cross-Product Decisions**: Central location for company-wide decisions
- **Team Clarity**: Clear distinction between business and technical work
- **Access Control**: Can set different permissions for business vs technical repos

**Migration Process:**

1. Create new repository with appropriate structure
2. Move marketing folder maintaining git history if possible
3. Update all internal links and references
4. Add redirect/notice in original location
5. Update workflows to use new repository

---

## Future Organization

### Business Repository Scope

- ✅ Strategic decisions and business planning
- ✅ Marketing strategy and go-to-market planning
- ✅ Legal documents and compliance
- ✅ Brand guidelines and messaging
- ✅ Cross-product decisions and company direction

### Product Repository Scope

- ✅ Technical decisions specific to that product
- ✅ Architecture and implementation decisions
- ✅ Product-specific documentation and examples
- ✅ Development workflows and tooling

This creates clean boundaries between business strategy and product development while maintaining effective collaboration.

---

_Document created: 2025-08-06_\
_Status: Decided - Create deepbrain/business repository_\
_Next action: Repository creation and content migration_
