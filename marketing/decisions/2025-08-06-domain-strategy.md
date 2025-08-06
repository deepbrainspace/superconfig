[🚪 ← Back to Decisions Overview](../DECISIONS.md)

# Domain Strategy Decision

**Decision Date**: 2025-08-06\
**Status**: ✅ Decided\
**Meeting Attendees**: Nayeem Syed, Claude Code Opus 4.1

## Executive Summary

**Current Situation**: Own deepbrain.space and superconfig.dev\
**Decision Made**: deepbrain.space as primary domain with product-specific domains forwarding\
**Final Strategy**: Unified branding through main site with product pages

---

## Current Domain Portfolio

1. **deepbrain.space** - Purchased, available
2. **superconfig.dev** - Purchased, currently in use
3. **deepbrain.dev** - NOT Available.

---

## Domain Options Analysis

### Option 1: deepbrain.space as Primary

**Pros:**

- ✅ **Already Owned**: No additional cost
- ✅ **Unique TLD**: Stands out, memorable
- ✅ **Brand Alignment**: "space" suggests expansive thinking
- ✅ **Available**: Ready to use immediately

**Cons:**

- ❌ **Trust Issues**: .space less trusted than .com/.dev
- ❌ **Developer Perception**: Not a standard dev TLD
- ❌ **Email Credibility**: name@deepbrain.space looks unusual
- ❌ **Corporate Hesitation**: Enterprises prefer traditional TLDs

**Best for:** Creative branding, consumer products

---

### Option 2: deepbrain.dev as Primary (NOT AVAILABLE)

**Pros:**

- ✅ **Developer Credibility**: .dev is Google-owned, HTTPS-only
- ✅ **Industry Standard**: Used by major tech companies
- ✅ **Trust Factor**: Developers trust .dev domains
- ✅ **SEO Benefits**: Google allegedly favors .dev slightly
- ✅ **Professional Emails**: name@deepbrain.dev looks legitimate

**Cons:**

- ❌ **Additional Cost**: ~$12/year (minimal)
- ❌ **Less Unique**: Many .dev domains
- ❌ **Need to Purchase**: Not immediately available

**Best for:** Developer tools, B2B software, professional presence

---

### Option 3: deepbrain.ai as Primary (NOT AVAILABLE)

**Pros:**

- ✅ **Perfect Alignment**: AI company with .ai domain
- ✅ **Premium Perception**: .ai domains signal AI focus
- ✅ **Memorable**: Short, relevant
- ✅ **Marketing Impact**: Instant understanding of focus

**Cons:**

- ❌ **Very Expensive**: .ai domains cost $100-200/year
- ❌ **Availability Unknown**: Likely taken
- ❌ **Anguilla Dependency**: Small island nation registry

**Best for:** AI-first branding (if pivoting fully to AI)

---

### Option 4: Hybrid Strategy (NOT POSSIBLE)

**Structure:**

- **deepbrain.dev** - Primary domain, main website
- **deepbrain.space** - Specific use cases (see below)
- **docs.deepbrain.dev** - Documentation
- **api.deepbrain.dev** - API endpoints

**Use deepbrain.space for:**

- Marketing campaigns (shorter, unique)
- Community forum
- Blog/content hub
- Redirect to main .dev site

---

## Opus 4.1 Recommendation

### 🎯 **Strong Recommendation: deepbrain.dev as Primary**

**My Reasoning:**

1. **Developer Trust**: You're building developer tools. Developers trust .dev domains. Period.

2. **Email Credibility**: When you email enterprises, nayeem@deepbrain.dev looks professional. nayeem@deepbrain.space raises eyebrows.

3. **Cost is Negligible**: $12/year is nothing. One customer pays for 10 years.

4. **SEO Advantage**: .dev has slight SEO benefits, especially for developer searches.

5. **Keep .space for Creative Uses**:
   - **idea.deepbrain.space** - Innovation lab
   - **labs.deepbrain.space** - Experimental features
   - **2025.deepbrain.space** - Conference/event sites

### Implementation Strategy

**Week 1:**

1. Purchase deepbrain.dev immediately
2. Set up DNS and basic redirect
3. Configure email forwarding

**Week 2:**

1. Deploy website to deepbrain.dev
2. Set up subdomains (docs, api, app)
3. Configure SSL certificates

**Week 3:**

1. Update all marketing materials
2. Set up redirects from .space to .dev
3. Launch with consistent branding

---

## SuperConfig.dev Fate Decision

### What to do with superconfig.dev?

**Option A: Product-Specific Site**

- Keep for SuperConfig product specifically
- Redirect to deepbrain.dev/superconfig
- Maintains product SEO

**Option B: Complete Redirect**

- 301 redirect everything to deepbrain.dev
- Simplest approach
- Loses brand equity

**Option C: Archive/Documentation** (RECOMMENDED)

- Keep live with notice about rebrand
- Link to new DeepBrain site
- Preserve SEO value and backlinks

**My Recommendation**: Option C for first year, then reassess

---

## Decision Criteria Matrix

| Criteria           | .space | .dev | .ai | Hybrid |
| ------------------ | ------ | ---- | --- | ------ |
| Developer Trust    | 🟡     | ✅   | ✅  | ✅     |
| Cost Effectiveness | ✅     | ✅   | ❌  | ✅     |
| Email Credibility  | ❌     | ✅   | ✅  | ✅     |
| SEO Potential      | 🟡     | ✅   | ✅  | ✅     |
| Brand Uniqueness   | ✅     | 🟡   | ✅  | ✅     |
| Flexibility        | 🟡     | ✅   | 🟡  | ✅     |
| Future-Proof       | 🟡     | ✅   | 🟡  | ✅     |

**Winner: deepbrain.space primary** (based on actual availability research)

---

## ✅ DECISION FINALIZED: deepbrain.space as Primary

**Final Decision**: deepbrain.space will serve as the main marketing and branding website for DeepBrain Technologies.

**Rationale**:

- deepbrain.dev and deepbrain.ai are NOT available (confirmed by user research)
- deepbrain.space provides unique brand identity and is already owned
- Cost-effective solution with unified branding strategy

**Domain Architecture**:

- **Primary**: deepbrain.space (main marketing site with dedicated product sections)
- **Product-specific forwards**:
  - superconfig.dev → deepbrain.space/superconfig (already owned)
  - logfusion.dev → deepbrain.space/logfusion (to acquire)
  - metarust.dev → deepbrain.space/metarust (to acquire)
  - DeepBrain Core: uses main site (no separate domain needed)

**Implementation Strategy**:

1. Use deepbrain.space as primary marketing site
2. Create dedicated sections/pages for each product
3. Product-specific domains forward to respective sections
4. Unified branding and SEO strategy

---

## Cost Analysis

**Annual Domain Costs:**

- deepbrain.dev: $12
- deepbrain.space: $12 (already paid)
- superconfig.dev: $12 (already paid)
- **Total: $36/year**

This is literally the cost of one coffee meetup. Don't let $12 block your decision.

---

## 🎯 Action Items from This Decision

### 🔴 This Week (Urgent)

- [ ] Register logfusion.dev and metarust.dev domains
- [ ] Set up domain forwarding configuration
- [ ] Plan deepbrain.space website structure with product sections

### 🟡 Next 2 Weeks (Important)

- [ ] Design unified website with product pages
- [ ] Configure DNS and forwarding rules
- [ ] Update all marketing materials to reference deepbrain.space

---

_Document created: 2025-08-06_\
_Last updated: 2025-08-06_\
_Next review: After website launch_
