# DeepBrain Project Decision Tracker

## Decision Overview

| Category             | Decision                                                                                    | Considerations             | Priority | Status |
| -------------------- | ------------------------------------------------------------------------------------------- | -------------------------- | -------- | ------ |
| 🏗️ **Infrastructure** | [Moon monorepo](decisions/2025-08-06-infrastructure.md#monorepo)                            | Unified CI/CD, caching     | ⚪       | ✅     |
| 🏗️ **Infrastructure** | [Next.js + Fumadocs](decisions/2025-08-06-website-technology.md)                            | Website & docs platform    | ⚪       | ✅     |
| 🏗️ **Infrastructure** | [Cloudflare Pages](decisions/2025-08-06-infrastructure.md#hosting)                          | Global CDN, cost-effective | ⚪       | ✅     |
| 🎨 **Branding**      | [DeepBrain](decisions/2025-08-06-branding-strategy.md)                                      | Unified brand, AI-ready    | ⚪       | ✅     |
| 🎨 **Branding**      | [Monorepo migration](decisions/2025-08-06-branding-strategy.md#monorepo-migration-decision) | Move to deepbrain/         | 🟡       | ✅     |
| 🎨 **Branding**      | logffi rename                                                                               | TraceLog proposed          | 🔴       | ❓     |
| 🎨 **Branding**      | meta-rust rename                                                                            | MetaRust proposed          | 🔴       | ❓     |
| 💼 **Business**      | [UK Ltd → Delaware](decisions/2025-08-06-business-formation.md)                             | Start UK, migrate later    | 🔴       | ✅     |
| 💼 **Business**      | [Bootstrap strategy](decisions/2025-08-06-business-formation.md#funding-strategy-decision)  | No VC initially            | ⚪       | ✅     |
| 💼 **Business**      | [Mark A equity](decisions/2025-08-06-business-formation.md#partner-equity-structure-mark-a) | 2-5% performance-based     | ⚪       | ✅     |
| 🌐 **Domain**        | Primary domain                                                                              | deepbrain.space vs .dev    | 🔴       | ❓     |
| 🌐 **Domain**        | superconfig.dev                                                                             | Keep or forward?           | 🔴       | ❓     |
| 📚 **Documentation** | Doc architecture                                                                            | Unified vs federated       | 🟡       | ❓     |
| 📚 **Documentation** | Content pipeline                                                                            | MDX sync from Rust docs    | 🟡       | ❓     |
| 🚀 **GTM**           | Launch timeline                                                                             | After products ready       | 🟢       | ⏸️      |
| 🚀 **GTM**           | Launch strategy                                                                             | Gradual vs all-at-once     | 🟡       | ❓     |

### Legend

**Status**: ✅ Decided | 🔄 Under Review | ❓ Under Discussion | ⏸️ Postponed\
**Priority**: 🔴 Urgent (this week) | 🟡 Important (2 weeks) | 🟢 Planning (month) | ⚪ Complete/No deadline

---

## 🎯 Priority Actions

### 🔴 This Week (Urgent)

- [ ] Incorporate UK Ltd (£12)
- [ ] Decide product names (logffi → TraceLog, meta-rust → MetaRust)
- [ ] Finalize domain strategy

### 🟡 Next 2 Weeks (Important)

- [ ] Execute monorepo migration to deepbrain/
- [ ] Define documentation architecture
- [ ] Determine launch strategy

### 🟢 This Month (Planning)

- [ ] GTM planning (after products ready)
- [ ] Community platform selection
- [ ] Content pipeline implementation

---

## 📁 Detailed Decision Documents

| Date       | Document                                                                   | Summary                                                   |
| ---------- | -------------------------------------------------------------------------- | --------------------------------------------------------- |
| 2025-08-06 | [Business Formation & Funding](decisions/2025-08-06-business-formation.md) | UK Ltd now, bootstrap to revenue, Mark A equity structure |
| 2025-08-06 | [Website Technology Stack](decisions/2025-08-06-website-technology.md)     | Next.js + Fumadocs evaluation and selection               |
| 2025-08-06 | [Branding & Naming Strategy](decisions/2025-08-06-branding-strategy.md)    | DeepBrain brand, product naming, monorepo migration       |
| 2025-08-06 | [Infrastructure Decisions](decisions/2025-08-06-infrastructure.md)         | Moon monorepo, Cloudflare Pages hosting                   |

---

## 📊 Decision History

| Date           | Decisions Made                                                                                                                                                                                                                                                          |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **2025-08-06** | • Business formation strategy (UK → US)<br>• Bootstrap funding approach<br>• Partner equity structure (2-5%)<br>• Website tech (Next.js + Fumadocs)<br>• DeepBrain brand confirmation<br>• Monorepo migration decision<br>• Infrastructure decisions (Moon, Cloudflare) |

---

## Visual Guide

### Status Indicators

- ✅ **Decided** - Decision finalized and documented
- 🔄 **Under Review** - Decision made but needs confirmation
- ❓ **Under Discussion** - Still evaluating options
- ⏸️ **Postponed** - Deferred to later date

### Priority Levels

- 🔴 **Red** - Urgent action needed this week
- 🟡 **Yellow** - Important for next 2 weeks
- 🟢 **Green** - Planning phase, this month
- ⚪ **Grey** - Complete or no specific deadline

### Category Icons

- 🏗️ **Infrastructure** - Technical architecture & tools
- 🎨 **Branding** - Names, logos, identity
- 💼 **Business** - Legal, funding, partnerships
- 🌐 **Domain** - Web presence, URLs
- 📚 **Documentation** - Content strategy
- 🚀 **GTM** - Go-to-market, launch

---

_Last Updated: 2025-08-06 | Version: 2.1_
