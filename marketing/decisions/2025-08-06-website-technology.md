[🚪 ← Back to Decisions Overview](../DECISIONS.md)

# Website Technology Stack Decision

**Decision Date**: 2025-08-06\
**Status**: 🟢 Decided\
**Meeting Attendees**: Nayeem Syed, Claude Code Opus 4.1

## Executive Summary

**Decision**: Next.js + Fumadocs for unified website and documentation\
**Rationale**: Best balance of professional design, AI-friendly maintenance, and flexibility

---

## Detailed Evaluation

### Technology Comparison Matrix

| Feature                    | Docusaurus               | Next.js + Nextra       | Next.js + Fumadocs         | Next.js + Repopress  |
| -------------------------- | ------------------------ | ---------------------- | -------------------------- | -------------------- |
| **MDX Support**            | ✅ Native                | ✅ Native              | ✅ Native                  | ✅ Native            |
| **Professional Look**      | ⭐⭐⭐ Good defaults     | ⭐⭐⭐⭐ Vercel design | ⭐⭐⭐⭐⭐ Modern, clean   | ⭐⭐⭐ Simple, clean |
| **Rust Docs Integration**  | 🟡 Manual sync           | 🟡 Manual sync         | ✅ API routes possible     | 🟡 Manual sync       |
| **Cloudflare Deployment**  | ✅ Static export         | ✅ Static export       | ✅ Static export           | ✅ Static export     |
| **CI/CD Complexity**       | ⭐⭐⭐⭐⭐ Simple        | ⭐⭐⭐⭐ Good          | ⭐⭐⭐⭐ Good              | ⭐⭐⭐⭐ Good        |
| **Blog Support**           | ✅ Built-in              | ✅ Built-in            | ✅ Via Next.js             | ❌ Docs only         |
| **Marketing Pages**        | 🟡 Limited               | ✅ Full flexibility    | ✅ Full flexibility        | ❌ Not designed for  |
| **AI Search**              | 🟡 Algolia plugin        | ✅ Custom integration  | ✅ Built-in AI search      | 🟡 Basic search      |
| **Theme UI**               | ✅ Good defaults         | ✅ Excellent           | ✅ Beautiful OOB           | ✅ Clean             |
| **AI Agent Maintenance**   | ⭐⭐⭐⭐ Well-documented | ⭐⭐⭐ Complex config  | ⭐⭐⭐⭐⭐ Clear structure | ⭐⭐⭐ Simple        |
| **Content Sync Pipeline**  | 🟡 Plugin needed         | ✅ API routes          | ✅ Built-in remote         | 🟡 Manual            |
| **Version Management**     | ✅ Excellent             | 🟡 Manual              | ✅ Good                    | ❌ Basic             |
| **Interactive Components** | 🟡 React only            | ✅ Full Next.js        | ✅ Full Next.js            | ✅ Full Next.js      |
| **Multi-product Docs**     | ✅ Multi-instance        | ✅ Supported           | ✅ Workspaces              | 🟡 Single focus      |
| **Bundle Size**            | ⭐⭐⭐ Moderate          | ⭐⭐⭐⭐ Optimized     | ⭐⭐⭐⭐⭐ Lightweight     | ⭐⭐⭐⭐ Small       |
| **Community/Support**      | ⭐⭐⭐⭐⭐ Meta-backed   | ⭐⭐⭐⭐ Vercel-backed | ⭐⭐⭐ Growing fast        | ⭐⭐ Small           |
| **Setup Complexity**       | ⭐⭐⭐⭐⭐ Minimal       | ⭐⭐⭐ Config heavy    | ⭐⭐⭐⭐ Straightforward   | ⭐⭐⭐⭐ Simple      |

---

## Why Next.js + Fumadocs Won

### Key Decision Factors

1. **Best for AI Agents**: Clear file structure, explicit configuration for AI-powered maintenance
2. **Modern Professional UI**: Looks like Linear/Vercel docs out of the box
3. **Rust Docs Synchronization**: Built-in support for remote content sources, can build pipeline to pull from docs.rs
4. **AI-Powered Search**: Native support for OpenAI/Anthropic search integration
5. **Unified Experience**: Single site, single deployment for marketing and docs
6. **Full Flexibility**: Complete Next.js power for marketing pages, blog, interactive demos
7. **Performance**: Lightweight, fast builds, excellent SEO

### Implementation Architecture

```
deepbrain.space (Next.js + Fumadocs)
├── /                     # Marketing landing
├── /products/           # Product showcase pages
├── /docs/              # Documentation hub
│   ├── /tracelog       # TraceLog documentation
│   ├── /metarust       # RustToolkit documentation
│   └── /superconfig    # SuperConfig documentation
├── /blog/              # Technical blog
├── /playground/        # Interactive demos
└── /api/               # API routes (including Rust docs sync)
```

---

## Rust Documentation Sync Strategy

### Automated Pipeline Design

```typescript
// app/api/sync-rust-docs/route.ts
export async function POST() {
  // 1. Fetch from docs.rs API
  const docsData = await fetchDocsRS();
  
  // 2. Transform to MDX format
  const mdxContent = transformToMDX(docsData);
  
  // 3. Write to Fumadocs content layer
  await writeToContentLayer(mdxContent);
  
  // 4. Trigger rebuild if needed
  await triggerStaticRegeneration();
}
```

### Benefits

- No manual documentation duplication
- Single source of truth
- Automatic updates via CI/CD
- Consistent formatting across platforms

---

## Implementation Plan

### Phase 1: Setup (Week 1)

- Initialize Next.js 14 with App Router
- Install and configure Fumadocs
- Set up Cloudflare Pages deployment
- Configure Moon tasks for builds

### Phase 2: Core Structure (Week 2)

- Create marketing landing pages
- Set up documentation structure
- Implement blog functionality
- Configure AI search

### Phase 3: Content Migration (Week 3)

- Move existing documentation
- Set up Rust docs sync
- Create product pages
- Add interactive demos

### Phase 4: Polish (Week 4)

- Optimize performance
- Implement analytics
- Add SEO optimizations
- Launch beta version

---

## Technical Specifications

### Stack Details

- **Framework**: Next.js 14 (App Router)
- **Documentation**: Fumadocs v13+
- **Styling**: Tailwind CSS
- **Deployment**: Cloudflare Pages
- **Build System**: Moon
- **Search**: Built-in AI search (OpenAI/Anthropic)
- **Analytics**: Plausible or Fathom
- **CMS**: MDX files in repo

### Performance Targets

- Lighthouse Score: 95+
- First Contentful Paint: <1s
- Time to Interactive: <2s
- Bundle Size: <200KB initial

---

## Alternative Considered

### Why Not Docusaurus?

- Limited flexibility for marketing pages
- Less modern UI out of the box
- Harder to integrate custom functionality
- Would need separate site for non-doc content

### Why Not Nextra?

- Configuration complexity
- Less clear structure for AI agents
- More opinionated, less flexible
- Heavier initial setup

### Why Not Separate Sites?

- Maintenance overhead
- Inconsistent user experience
- Multiple deployments
- Fragmented analytics

---

## Risk Mitigation

### Potential Challenges

1. **Learning Curve**: Fumadocs is newer
   - Mitigation: Good documentation, active community

2. **SEO Migration**: From existing sites
   - Mitigation: Proper redirects, sitemap updates

3. **Build Times**: With multiple products
   - Mitigation: Incremental Static Regeneration

---

## Success Criteria

- Professional appearance matching top developer tools
- Sub-2 second page loads
- Working AI-powered search
- Automated Rust docs synchronization
- Easy content updates via MDX
- Mobile-responsive design
- Dark/light theme support

---

## Meeting Notes

**Key Discussion Points:**

- Need for unified marketing and documentation
- Importance of AI-friendly maintenance
- Professional appearance requirements
- Rust documentation synchronization needs
- Performance and SEO considerations

**Decision Process:**

- Evaluated 4 main options
- Created detailed comparison matrix
- Considered AI agent maintenance as key factor
- Fumadocs emerged as best balance

---

_Document created: 2025-08-06_\
_Last updated: 2025-08-06_\
_Next review: After initial implementation_
