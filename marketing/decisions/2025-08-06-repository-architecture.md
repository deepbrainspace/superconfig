[🚪 ← Back to Decisions Overview](../DECISIONS.md)

# Repository Architecture & Documentation Strategy Decision

**Decision Date**: 2025-08-06\
**Status**: ✅ Decided\
**Meeting Attendees**: Nayeem Syed, Sonnet 4\
**Priority**: 🟡 Important (affects all future development)

## 🎯 Action Items from This Decision

### 🔴 This Week (Urgent)

- [ ] Create logfusion GitHub repository from logffi migration
- [ ] Plan migration from current superconfig monorepo (preserve existing cookbook/examples structure)
- [ ] Integrate docs.rs automation with `include_str!` approach

### 🟡 Next 2 Weeks (Important)

- [ ] Extract GitHub repository template from proven logfusion repo patterns
- [ ] Set up main deepbrain monorepo structure
- [ ] Build MD → MDX conversion script for website builds
- [ ] Setup Google Analytics 4 for cross-domain tracking (future task)

---

## Executive Summary

**Decision Context**: With separate product websites decided, we need to determine the optimal repository structure that balances:

- GitHub growth metrics (separate repos for stars/community)
- Development efficiency (shared tooling, Moon monorepo benefits)
- Documentation consistency (unified strategy across products)
- Maintenance overhead (templates, shared infrastructure)

**Final Decision**: Hybrid approach with product-specific Moon monorepos + shared infrastructure repo. Implementation will start with logfusion repository as working example, then extract template from proven patterns.

---

## Discussion Evolution

### Phase 1: Monorepo vs Separate Repos for Growth Metrics

**Initial Concern**:

> "One of our key metrics to see product growth/adoption would be the github stars and contributions, but that could get foggy if we have all the products in one repo"

**Analysis of Growth Tracking:**

- **Separate repos enable**: Individual product star tracking, focused communities, clear success metrics
- **Examples**: HashiCorp (terraform 41k stars, vault 30k stars), Vercel (next.js 120k stars, swr 30k stars)
- **Marketing impact**: "Show HN: LogFusion" → lands on focused `github.com/deepbrain/logfusion`

**Conclusion**: Separate repositories are essential for growth tracking and marketing clarity.

### Phase 2: Website Architecture Decision

**Question**: Individual product domains vs pages within deepbrain.space?

**Analysis**:

- **Full separate sites** (logfusion.dev, metarust.dev) provide maximum SEO impact
- **User experience**: Developers expect focused, product-specific sites
- **Successful pattern**: Next.js (nextjs.org), Terraform (terraform.io) have dedicated domains

**Conclusion**: Each product gets its own complete website with comprehensive documentation.

### Phase 3: Repository Structure Refinement

**Challenge**: How to balance separate repos for growth with development efficiency?

**Nayeem's Refined Proposal**:

```
Individual product repos as Moon monorepos:
├── crates/
│   ├── logfusion/          # Main Rust crate
│   ├── logfusion-py/       # Python bindings  
│   ├── logfusion-cli/      # CLI tool
│   └── logfusion-ffi/      # FFI bindings
├── packages/
│   └── logfusion.dev/      # Next.js + Fumadocs website
├── docs/                   # Documentation source (syncs to website + docs.rs)
└── .moon/                  # Moon configuration

Main deepbrain monorepo for shared infrastructure:
├── packages/
│   ├── deepbrain.space/    # Company website
│   ├── design-system/      # Shared React components
│   └── docs-templates/     # Shared documentation templates
├── tools/
│   ├── docs-sync/          # Documentation synchronization tools
│   └── github-workflows/   # Shared CI/CD workflows
└── templates/
    └── product-repo/       # GitHub template for new products
```

---

## Proposed Architecture

### Repository Structure

**1. Main Infrastructure Repository**

```
github.com/deepbrain/deepbrain (current: superconfig)
├── packages/
│   ├── deepbrain.space/           # Company website + product showcase
│   ├── design-system/             # @deepbrain/design-system npm package
│   ├── docs-templates/            # Shared Fumadocs templates  
│   └── shared-workflows/          # Reusable GitHub Actions
├── tools/
│   ├── docs-sync/                 # Rust docs → Fumadocs sync tools
│   ├── repo-setup/                # Scripts for new product setup
│   └── release-automation/        # Cross-repo release coordination
├── templates/
│   └── product-monorepo/          # GitHub template repository
└── .moon/                         # Moon configuration
```

**2. Individual Product Repositories (Moon Monorepos)**

```
github.com/deepbrain/logfusion
├── crates/
│   ├── logfusion/                 # Main Rust crate → crates.io + docs.rs
│   ├── logfusion-py/              # Python bindings → PyPI
│   ├── logfusion-cli/             # CLI tool → crates.io + releases
│   └── logfusion-ffi/             # C/FFI bindings
├── packages/
│   └── logfusion.dev/             # Next.js + Fumadocs → logfusion.dev
├── docs/                          # MDX documentation source
│   ├── getting-started/           # Tutorial content
│   ├── advanced/                  # Advanced guides
│   ├── api/                       # API documentation (synced from Rust docs)
│   └── examples/                  # Code examples
├── examples/                      # Rust example projects
├── tests/                         # Integration tests
└── .moon/                         # Product-specific Moon config
```

### Website Deployment Strategy

| Product            | Domain             | Repository                 | Deployment       |
| ------------------ | ------------------ | -------------------------- | ---------------- |
| **Company**        | deepbrain.space    | `deepbrain/deepbrain`      | Cloudflare Pages |
| **LogFusion**      | logfusion.dev      | `deepbrain/logfusion`      | Cloudflare Pages |
| **MetaRust**       | metarust.dev       | `deepbrain/metarust`       | Cloudflare Pages |
| **SuperConfig**    | superconfig.dev    | `deepbrain/superconfig`    | Cloudflare Pages |
| **DeepBrain Core** | deepbrain.space/ai | `deepbrain/deepbrain-core` | Cloudflare Pages |

---

## Documentation Strategy

### Two-Tier Documentation System

**Tier 1: API Reference (docs.rs)**

- Generated from comprehensive Rust doc comments in `src/lib.rs`
- Automatically published on `cargo publish`
- Focus: Complete API documentation with basic usage examples
- Cross-links to custom site for tutorials

**Tier 2: Comprehensive Guides (Custom Sites)**

- Rich MDX content in `docs/` folder
- Built with Next.js + Fumadocs
- Focus: Getting started, tutorials, use cases, comparisons, community

### Documentation Sync Workflow

**Single Source of Truth Principle**: Documentation lives **in each crate** alongside the code it describes.

**Source Structure (per crate)**:

```
crates/logfusion/
├── src/lib.rs              # Rust code with doc comments → docs.rs
├── cookbook/               # Tutorial content (Markdown)
│   ├── 01-basic-usage.md   # → docs.rs + website
│   └── 02-advanced.md      # → docs.rs + website
├── examples/               # Runnable demos → docs.rs + website
│   ├── basic_demo.rs       # → cargo run --example + docs
│   └── advanced_demo.rs    # → cargo run --example + docs
└── tests/                  # Integration tests → docs.rs coverage

crates/logfusion-py/        # Python bindings crate
├── cookbook/               # Python-specific tutorials
└── examples/               # Python examples

crates/logfusion-cli/       # CLI tool crate
├── cookbook/               # CLI usage guides  
└── examples/               # CLI examples
```

**Automated Sync Process**:

**1. Crate → docs.rs (Automatic)**

```rust
// src/lib.rs - Include external files in doc comments
#![doc = include_str!("../README.md")]

/// # LogFusion Cookbook
/// Complete tutorials and guides.
#[doc = include_str!("../cookbook/README.md")]
pub mod cookbook {
    /// # Basic Logging Tutorial
    #[doc = include_str!("../cookbook/01-basic-logging.md")]
    pub mod basic_logging {}
}

/// # Examples  
/// Runnable code examples.
pub mod examples {
    /// Run: `cargo run --example structured_logging_demo`
    #[doc = include_str!("../examples/structured_logging_demo.rs")]
    pub mod structured_logging_demo {}
}
```

**2. Crates → Website (Build Script)**

```javascript
// packages/logfusion.dev/scripts/sync-docs.js
function syncCrateDocumentation(cratePath) {
    // Convert cookbook/*.md → content/docs/*.mdx
    const cookbookFiles = fs.readdirSync(path.join(cratePath, 'cookbook'));
    cookbookFiles.forEach(file => {
        const mdContent = fs.readFileSync(path.join(cratePath, 'cookbook', file));
        const mdxContent = convertMarkdownToMDX(mdContent);
        fs.writeFileSync(path.join('content/docs', file.replace('.md', '.mdx')), mdxContent);
    });
}

// Sync from multiple crates
['logfusion', 'logfusion-py', 'logfusion-cli'].forEach(crate => {
    syncCrateDocumentation(`../../crates/${crate}`);
});
```

**Website Content Structure**:

```
packages/logfusion.dev/
├── content/docs/           # Generated MDX from all crates
│   ├── rust/              # From crates/logfusion/cookbook/
│   ├── python/            # From crates/logfusion-py/cookbook/
│   ├── cli/               # From crates/logfusion-cli/cookbook/
│   └── examples/          # Aggregated from all examples/
└── scripts/sync-docs.js   # Pulls from all crate folders
```

**Benefits**:

- ✅ **Crate Independence**: Each crate publishable with complete docs
- ✅ **No Duplication**: Single markdown files serve both docs.rs and website
- ✅ **Auto-Sync**: `cargo doc` and website build both get content automatically
- ✅ **Website Aggregation**: Combine Rust + Python + CLI docs in unified experience

### Shared Design System

**@deepbrain/design-system Package**:

- React components for consistent branding
- Fumadocs theme customizations
- Shared layouts and templates
- Published to npm, imported by all product sites

---

## GitHub Template Strategy

### Product Repository Template

**Template Repository**: `github.com/deepbrain/template-product-monorepo`

**Template Contents**:

```
template-product-monorepo/
├── crates/
│   └── {{product-name}}/
│       ├── src/lib.rs              # Template with doc comment examples
│       ├── Cargo.toml              # Pre-configured with DeepBrain metadata
│       └── README.md               # Links to docs.rs and custom site
├── packages/
│   └── {{product-name}}.dev/
│       ├── app/                    # Next.js 14 app directory
│       ├── content/docs/           # Fumadocs content structure
│       ├── components/             # Product-specific components
│       ├── fumadocs.config.ts      # Pre-configured Fumadocs
│       └── package.json            # Dependencies including @deepbrain/design-system
├── crates/
│   └── {{product-name}}/
│       ├── cookbook/               # Tutorial content (stays with crate)
│       │   ├── 01-basic-usage.md   # Template tutorial
│       │   └── README.md           # Template overview
│       ├── examples/               # Runnable examples (stays with crate)
│       │   └── basic_demo.rs       # Template example
│       └── tests/                  # Integration tests (stays with crate)
├── .github/
│   └── workflows/                  # CI/CD workflows (test, build, deploy)
├── .moon/
│   └── workspace.yml               # Moon configuration
└── scripts/
    ├── setup.sh                    # Initialize new product repo
    └── sync-docs.sh                # Documentation sync script
```

**Setup Process**:

1. Create new repo from template
2. Run `scripts/setup.sh {{product-name}}` to replace placeholders
3. Configure Cloudflare Pages deployment
4. Register domain and configure DNS
5. Publish initial version to crates.io

---

## Migration Strategy

### Phase 1: Repository Setup (Week 1-2)

**1. Main Infrastructure Repository**:

- [ ] Rename current `superconfig` repo to `deepbrain`
- [ ] Restructure as main monorepo with Moon
- [ ] Move current superconfig crate to `crates/superconfig/`
- [ ] Create `packages/deepbrain.space/` for company website
- [ ] Set up shared design system in `packages/design-system/`

**2. Create Product Repository Template**:

- [ ] Create `template-product-monorepo` repository
- [ ] Set up template structure with placeholders
- [ ] Create setup scripts and documentation
- [ ] Test template with one product repository

### Phase 2: Product Repository Creation (Week 3-4)

**1. LogFusion Repository**:

- [ ] Create from template: `github.com/deepbrain/logfusion`
- [ ] Migrate current logffi code to `crates/logfusion/`
- [ ] Keep existing `cookbook/`, `examples/`, `tests/` in crate folder
- [ ] Add `include_str!` references in `src/lib.rs` for docs.rs integration
- [ ] Set up website in `packages/logfusion.dev/`
- [ ] Create sync script to convert cookbook MD → website MDX
- [ ] Configure documentation sync
- [ ] Deploy to logfusion.dev

**2. MetaRust Repository**:

- [ ] Create from template: `github.com/deepbrain/metarust`
- [ ] Migrate current meta-rust code
- [ ] Set up website and documentation
- [ ] Deploy to metarust.dev

**3. SuperConfig Repository**:

- [ ] Create from template: `github.com/deepbrain/superconfig`
- [ ] Migrate current superconfig code from main repo
- [ ] Expand existing superconfig.dev site
- [ ] Set up Moon monorepo structure

### Phase 3: Documentation Integration & Tooling (Week 5-6)

**1. Documentation Integration**:

- [ ] Add `include_str!` integration to all crate `src/lib.rs` files
- [ ] Build MD → MDX conversion script for website builds
- [ ] Set up automated sync in CI/CD pipelines
- [ ] Test that `cargo doc` shows complete crate documentation
- [ ] Verify website aggregation across multiple crates

**2. Shared Infrastructure**:

- [ ] Publish `@deepbrain/design-system` to npm
- [ ] Set up shared GitHub Actions workflows
- [ ] Configure analytics and monitoring across all sites
- [ ] Create cross-product linking system

---

## Benefits of This Architecture

### Development Efficiency

- **Moon monorepo per product**: Efficient builds, task orchestration, dependency management
- **Shared infrastructure**: Consistent tooling, design system, CI/CD workflows
- **Template system**: Rapid new product setup, consistent structure

### Growth Tracking

- **Separate GitHub repos**: Clear star metrics, focused communities, individual product success tracking
- **Focused marketing**: Each product has dedicated landing pages and community
- **Independent release cycles**: Products can evolve at their own pace

### Documentation Excellence

- **Single source of truth**: Documentation lives with code, no duplication
- **Automatic docs.rs integration**: `include_str!` pulls content automatically
- **Website aggregation**: Multiple crates combine into unified user experience
- **Crate independence**: Each crate publishable with complete documentation
- **No maintenance overhead**: Write once, sync everywhere

### Maintenance & Scaling

- **Template updates propagate**: Fix template → all future products benefit
- **Shared component updates**: Update design system → all sites get improvements
- **Clear separation of concerns**: Product code vs shared infrastructure
- **Content stays with code**: Developers update docs where they work

---

## Technical Decisions Resolved

1. **MD → MDX conversion complexity**: ✅ **Simple transformation** - Markdown is mostly MDX-compatible, just add frontmatter and syntax highlighting hints

2. **Release coordination**: ✅ **Moon CI/CD** - Use Moon's task orchestration for multi-repo release workflows

3. **Template maintenance**: ✅ **One-time setup** - Template is only for initial repo creation, shared components get pulled via npm dependencies on site rebuild

4. **Cross-repo dependency management**: ✅ **Semantic versioning** - Products pick up tagged versions of shared crates automatically

5. **Documentation versioning**: ✅ **Product-coupled** - Documentation versions with the product release (cookbook/examples stay with crate version)

## Remaining Question

**Analytics consolidation**: How to track user journeys across product sites?

**Potential Solutions**:

- **Google Analytics 4 with cross-domain tracking** - Single GA4 property with domain-level segmentation
- **Unified user identification** - Shared user ID across deepbrain.space, logfusion.dev, metarust.dev, superconfig.dev
- **Custom analytics dashboard** - Aggregate data from all domains into single DeepBrain analytics view
- **UTM parameter strategy** - Track referrals between product sites (logfusion.dev → superconfig.dev)

**Decision**: Start with GA4 cross-domain tracking as it's the simplest to implement and provides good insights into user journeys across the product ecosystem.

## Implementation Approach

**Strategy**: Example-first implementation starting with LogFusion repository, then extract template from proven patterns.

**Rationale**:

- Build working repository first to validate architecture
- Extract template based on real-world usage patterns
- Avoid over-engineering template before understanding actual needs
- LogFusion provides excellent foundation with existing cookbook/, examples/, tests/ structure

---

## Next Steps

1. **Finalize architecture details** with team input
2. **Create GitHub template repository** with complete setup
3. **Begin migration** starting with main deepbrain repository restructure
4. **Build documentation sync tools** for automated maintenance
5. **Test full workflow** with one product before scaling to all

---

_Document created: 2025-08-06_\
_Last updated: 2025-08-06_\
_Next review: After architecture finalization_
