# DeepBrain Project Decision Tracker

## Decision Overview

| Category             | Topic                                                                                                        | Decision                           | Date   | Priority | Status |
| -------------------- | ------------------------------------------------------------------------------------------------------------ | ---------------------------------- | ------ | -------- | ------ |
| 🌐 **Domain**        | [Primary domain & superconfig.dev](decisions/2025-08-06-domain-strategy.md)                                  | deepbrain.space + product forwards | 6.8.25 | 🔴       | ✅     |
| 🚀 **GTM**           | [Marketing start point](decisions/2025-08-06-marketing-start-analysis.md)                                    | After which product? (2/3/4)       | -      | 🟡       | ❓     |
| 📚 **Documentation** | Doc architecture                                                                                             | Unified vs federated               | -      | 🟡       | ❓     |
| 📚 **Documentation** | Content pipeline                                                                                             | MDX sync from Rust docs            | -      | 🟡       | ❓     |
| 🚀 **GTM**           | Launch strategy                                                                                              | Gradual vs all-at-once             | -      | 🟡       | ❓     |
| 🚀 **GTM**           | Launch timeline                                                                                              | After products ready               | -      | 🟢       | ⏸️      |
| 💼 **Business**      | [Company naming](decisions/2025-08-06-company-logffi-naming.md)                                              | DeepBrain Technologies Ltd         | 6.8.25 | 🔴       | ✅     |
| 🎨 **Branding**      | [Product: logffi → LogFusion](decisions/2025-08-06-company-logffi-naming.md#part-2-logffi-alternative-names) | LogFusion                          | 6.8.25 | 🔴       | ✅     |
| 🎨 **Branding**      | [Product: meta-rust](decisions/2025-08-06-metarust-naming.md)                                                | MetaRust (keep current)            | 6.8.25 | 🔴       | ✅     |
| 🎨 **Branding**      | [Product suite strategy](decisions/2025-08-06-product-suite-branding.md)                                     | Hybrid: descriptive + brand        | 6.8.25 | 🔴       | ✅     |
| 💼 **Business**      | [Incorporation strategy](decisions/2025-08-06-business-formation.md)                                         | UK Ltd → Delaware C-Corp           | 6.8.25 | 🔴       | ✅     |
| 💼 **Business**      | [Partner equity](decisions/2025-08-06-business-formation.md#partner-equity-structure-mark-a)                 | Mark A: 2-5% performance           | 6.8.25 | 🔴       | ✅     |
| 🎨 **Branding**      | [Main brand](decisions/2025-08-06-branding-strategy.md)                                                      | DeepBrain                          | 6.8.25 | 🔴       | ✅     |
| 🏗️ **Infrastructure** | [Website technology](decisions/2025-08-06-website-technology.md)                                             | Next.js + Fumadocs                 | 6.8.25 | 🔴       | ✅     |
| 💼 **Business**      | [Funding approach](decisions/2025-08-06-business-formation.md#funding-strategy-decision)                     | Bootstrap to revenue               | 6.8.25 | 🟡       | ✅     |
| 🎨 **Branding**      | [Repository location](decisions/2025-08-06-branding-strategy.md#monorepo-migration-decision)                 | Move to deepbrain/                 | 6.8.25 | 🟡       | ✅     |
| 🏗️ **Infrastructure** | [Hosting platform](decisions/2025-08-06-infrastructure.md#hosting)                                           | Cloudflare Pages                   | 6.8.25 | 🟡       | ✅     |
| 🏗️ **Infrastructure** | [Monorepo structure](decisions/2025-08-06-infrastructure.md#monorepo)                                        | Moon monorepo                      | 6.8.25 | 🟢       | ✅     |

### Legend

| **Status**          | **Priority**            | **Categories**                            |
| ------------------- | ----------------------- | ----------------------------------------- |
| ✅ Decided          | 🔴 Urgent (this week)   | 🏗️ Infrastructure - Technical architecture |
| 🔄 Under Review     | 🟡 Important (2 weeks)  | 🎨 Branding - Names, logos, identity      |
| ❓ Under Discussion | 🟢 Planning (month)     | 💼 Business - Legal, funding              |
| ⏸️ Postponed         | ⚪ Complete/No deadline | 🌐 Domain - Web presence, URLs            |
|                     |                         | 📚 Documentation - Content strategy       |
|                     |                         | 🚀 GTM - Go-to-market, launch             |

---

## 🎯 Action Items

### 🔴 This Week (Urgent)

- [ ] Incorporate DeepBrain Technologies Ltd (£12) [[Business Formation]](decisions/2025-08-06-business-formation.md)
- [x] ~~Decide product names~~ ✅ LogFusion & MetaRust decided [[Company & LogFFI Naming]](decisions/2025-08-06-company-logffi-naming.md)
- [x] ~~Finalize domain strategy~~ ✅ deepbrain.space decided [[Domain Strategy]](decisions/2025-08-06-domain-strategy.md)
- [ ] Register crates.io names (logfusion, metarust) [[Company & LogFFI Naming]](decisions/2025-08-06-company-logffi-naming.md)
- [ ] Register logfusion.dev and metarust.dev domains [[Domain Strategy]](decisions/2025-08-06-domain-strategy.md)
- [ ] Begin codebase rename: logffi → logfusion [[Company & LogFFI Naming]](decisions/2025-08-06-company-logffi-naming.md)

### 🟡 Next 2 Weeks (Important)

- [ ] Complete codebase updates (Cargo.toml, docs, README) [[Company & LogFFI Naming]](decisions/2025-08-06-company-logffi-naming.md)
- [ ] Execute monorepo migration to deepbrain/ [[Branding Strategy]](decisions/2025-08-06-branding-strategy.md)
- [ ] Rename/rebrand git repository from superconfig to deepbrain [[Branding Strategy]](decisions/2025-08-06-branding-strategy.md)
- [ ] Create logos/branding for LogFusion and MetaRust [[Product Suite Branding]](decisions/2025-08-06-product-suite-branding.md)
- [ ] Prepare product rename announcement posts [[Company & LogFFI Naming]](decisions/2025-08-06-company-logffi-naming.md)
- [ ] Define documentation architecture [Documentation Architecture Decision - TBD]
- [ ] Determine launch strategy [[Marketing Start Analysis]](decisions/2025-08-06-marketing-start-analysis.md)

### 🟢 This Month (Planning)

- [ ] GTM planning (after products ready) [[Marketing Start Analysis]](decisions/2025-08-06-marketing-start-analysis.md)
- [ ] Community platform selection [Community Platform Decision - TBD]
- [ ] Content pipeline implementation [Content Pipeline Decision - TBD]
- [ ] Website development (deepbrain.space with Next.js + Fumadocs) [[Website Technology]](decisions/2025-08-06-website-technology.md)
- [ ] Legal setup completion (business banking, accounting) [[Business Formation]](decisions/2025-08-06-business-formation.md)

---

## 📁 Detailed Decision Documents

| Date       | Document                                                                     | Summary                                                             |
| ---------- | ---------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| 2025-08-06 | [Business Formation & Funding](decisions/2025-08-06-business-formation.md)   | UK Ltd now, bootstrap to revenue, Mark A equity structure           |
| 2025-08-06 | [Website Technology Stack](decisions/2025-08-06-website-technology.md)       | Next.js + Fumadocs evaluation and selection                         |
| 2025-08-06 | [Branding & Naming Strategy](decisions/2025-08-06-branding-strategy.md)      | DeepBrain brand, product naming, monorepo migration                 |
| 2025-08-06 | [Infrastructure Decisions](decisions/2025-08-06-infrastructure.md)           | Moon monorepo, Cloudflare Pages hosting                             |
| 2025-08-06 | [Marketing Start Analysis](decisions/2025-08-06-marketing-start-analysis.md) | Soft launch after product 2, full after product 3                   |
| 2025-08-06 | [Company Naming & LogFFI](decisions/2025-08-06-company-logffi-naming.md)     | DeepBrain Technologies Ltd + ErrorForge product name                |
| 2025-08-06 | [Product Suite Branding](decisions/2025-08-06-product-suite-branding.md)     | Hybrid strategy: descriptive names with DeepBrain brand             |
| 2025-08-06 | [Meta-Rust Naming Options](decisions/2025-08-06-metarust-naming.md)          | MetaRust (keep current) for Rust-specific tool                      |
| 2025-08-06 | [Domain Strategy](decisions/2025-08-06-domain-strategy.md)                   | deepbrain.dev primary, .space for creative, archive superconfig.dev |

---

## 📊 Decision History

| Date           | Decisions Made                                                                                                                                                                                                                                                                                         |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **2025-08-06** | • Company: DeepBrain Technologies Ltd<br>• Products: LogFusion (logffi), MetaRust (keep)<br>• Product suite strategy (hybrid naming)<br>• Business formation (UK → US)<br>• Bootstrap funding<br>• Partner equity (2-5%)<br>• Website tech (Next.js + Fumadocs)<br>• Infrastructure (Moon, Cloudflare) |

---

_Last Updated: 2025-08-06 | Version: 2.1_
