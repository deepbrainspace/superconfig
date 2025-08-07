# Nix Development Environment & Package Distribution Strategy

[🚪 ← Back to Decisions Overview](../DECISIONS.md)

## Decision Summary

**Status**: **✗** **ABORTED** - Use Makefile + proto instead for simpler development setup\
**Priority**: 🟡 **Important** (2 weeks)\
**Date**: 2025-08-06\
**Category**: 🏗️ Infrastructure

## Action Items

### ✅ Completed/Aborted

- [x] ~~Add Nix flake configuration to superconfig monorepo for testing~~ **✗** **ABORTED** - Implemented Makefile + proto instead
- [x] ~~Configure existing tools (cargo-examples, moon) via Nix instead of manual installation~~ **✗** **ABORTED** - Used proto for tool management
- [x] ~~Test Nix + Moon integration workflow across team members~~ **✗** **ABORTED** - Makefile approach proved simpler
- [x] ~~Document Nix setup process for future git templates~~ **✗** **ABORTED** - Documented Makefile approach instead
- [x] ~~Enable Nix experimental features in development documentation~~ **✗** **ABORTED** - No longer needed

### 🟢 This Month (Planning)

- [x] ~~Create git template with Nix + Moon for future DeepBrain product monorepos~~ **✗** **ABORTED** - Will use Makefile template
- [x] ~~Research package distribution options (GitHub Releases vs wrapper strategy vs custom registry)~~ **✗** **ABORTED** - Deferred to future

---

## Abort Rationale

### **Decision to Abandon Nix (2025-08-07)**

After initial exploration and flake configuration implementation, the team decided to abandon Nix in favor of a simpler Makefile + proto approach.

#### **Key Issues with Nix**

1. **Complexity Overhead**: Managing flake.lock files and SHA updates added unnecessary complexity
2. **Learning Curve**: Team members needed significant time investment to understand Nix concepts
3. **Simpler Alternative Available**: Proto + Makefile achieved same goals with less complexity
4. **Development Speed**: Makefile approach was faster to implement and understand

#### **Successful Alternative: Makefile + Proto**

- **Proto**: Universal tool version manager handles rust, moon, direnv installation
- **Makefile**: Simple `make init` command for development setup
- **Same Benefits**: Consistent tool versions, easy onboarding, cross-platform support
- **Less Complexity**: No flake management, no learning curve, standard tooling

#### **Implementation Success**

The Makefile + proto approach successfully delivered:

- ✅ Consistent development environments
- ✅ Easy setup (`make init`)
- ✅ Cross-platform compatibility (Mac/Linux/WSL)
- ✅ Tool version management via proto
- ✅ Zero learning curve for team members

---

## Context & Problem

DeepBrain is building multiple monorepo-based products (LogFusion, MetaRust, SuperConfig, DeepBrain Core) with the following challenges:

1. **Inconsistent development environments** between local development, CI, and new team members
2. **Manual tool installation** (cargo-examples, moon, etc.) prone to version drift
3. **Cross-platform compatibility** needed for Mac/Linux/Windows (WSL)
4. **Future package distribution** for custom DeepBrain tools to external users
5. **Template-based project setup** for rapid new product development

## Decision: Adopt Nix for Development Environment Management

### **Primary Use Cases**

1. **Development Environment Consistency**
   - Identical tool versions across all machines and CI
   - Automatic environment activation with direnv
   - No manual installation of development tools

2. **Git Template Integration**
   - Standardized setup for all DeepBrain product monorepos
   - Mix-and-match templates for different project types
   - Moon + Nix integration as standard stack

3. **Future Package Distribution**
   - Custom binary distribution for DeepBrain tools
   - Cross-platform automatic architecture detection
   - Professional distribution infrastructure

### **Technical Architecture**

#### **Phase 1: Development Environment (Immediate)**

```nix
# flake.nix - Standard DeepBrain development environment
{
  description = "DeepBrain development environment";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  
  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              # Rust toolchain from rust-toolchain.toml
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              
              # Development tools - consistent versions everywhere
              cargo-examples  # Run all examples
              # moon         # Task runner (when available in nixpkgs)
              git
              
              # System dependencies
              pkg-config
              openssl
            ];
            
            env = {
              RUST_BACKTRACE = "1";
              RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            };
          };
        });
    };
}
```

```bash
# .envrc - Automatic environment activation
use flake
```

#### **Moon + Nix Integration**

```yaml
# .moon/tasks.yml - Moon uses tools provided by Nix
tasks:
  examples:
    command: 'cargo examples'  # Available via Nix
    inputs: ['examples/**/*', 'src/**/*', 'Cargo.toml']
    options:
      cache: false
      outputStyle: "stream"

  examples-release:
    command: 'cargo examples -- --release'
    inputs: ['examples/**/*', 'src/**/*', 'Cargo.toml']
    options:
      cache: false
```

**Key Insight**: Moon orchestrates tasks, Nix provides consistent tooling - they complement each other perfectly.

#### **Cross-Platform Support**

| Platform            | Support Level | Notes                             |
| ------------------- | ------------- | --------------------------------- |
| Linux x86_64        | ✅ Native     | Full support, fastest performance |
| Linux ARM64         | ✅ Native     | Full support for ARM servers      |
| macOS Intel         | ✅ Native     | Stable, widely used               |
| macOS Apple Silicon | ✅ Native     | M1/M2 support, good performance   |
| Windows             | ✅ WSL2       | Requires WSL2, works perfectly    |

### **Phase 2: Package Distribution (Future)**

#### **Multi-Strategy Distribution**

1. **GitHub Releases** (Immediate, unlimited, free)
   - Build binaries in CI
   - Upload to GitHub releases
   - Nix fetches from releases

2. **Wrapper Strategy** (Maximum ecosystem reach)
   ```bash
   # Single Rust binary, multiple installation methods
   cargo install logfusion-cli      # From crates.io
   pip install logfusion-cli        # Python wrapper from PyPI  
   npm install -g logfusion-cli     # npm wrapper from npmjs
   nix profile install github:deepbrain/tools#logfusion-cli  # Nix
   ```

3. **Custom Package Registry** (Professional, later)
   - Custom domain: `download.deepbrain.space`
   - Cloudflare Workers + R2 backend
   - Multiple package format support
   - Professional web interface

#### **Custom Registry Architecture (Future)**

```
Users → download.deepbrain.space → Cloudflare Worker → R2 Backend
                ↓
        Multiple package formats:
        - Nix binaries
        - npm tarballs  
        - Raw binaries
        - Python wheels
```

**Cost Structure**:

- Cloudflare R2: 10GB free storage
- Cloudflare Workers: Generous free tier
- GitHub Releases: Unlimited storage/bandwidth
- npm/PyPI: Free hosting for open source

### **Template Strategy**

#### **Git Template Structure**

```
deepbrain-template/
├── flake.nix                 # Nix environment
├── .envrc                    # Direnv integration  
├── .moon/
│   ├── workspace.yml         # Moon configuration
│   └── tasks.yml            # Standard tasks
├── Cargo.toml               # Rust workspace
├── README.template.md       # Gets filled out
└── .github/workflows/       # Standard CI/CD
```

#### **Template Usage**

```bash
# Create new DeepBrain project
nix flake init --template github:deepbrain/templates#rust-monorepo
cd my-new-project
# Environment automatically activates, all tools available
moon run :examples           # Works immediately
```

## Benefits Analysis

### ✅ **Development Experience**

- **Zero setup time**: New contributors get perfect environment in seconds
- **Consistent everywhere**: Same tools/versions in dev, CI, and production
- **Cross-platform**: Works on Mac/Linux/Windows (WSL) identically
- **Automatic**: direnv integration means tools just work when you `cd`

### ✅ **Strategic Alignment**

- **Multiple monorepos**: Each product gets identical development setup
- **Template scaling**: Easy to create specialized templates for different product types
- **Professional distribution**: Can evolve into sophisticated package distribution
- **Cost effective**: Leverages free tiers, scales with usage

### ✅ **Technical Benefits**

- **Reproducible builds**: Same input → same output, guaranteed
- **Hermetic environments**: No global state pollution
- **Fast environment switching**: Instant activation, no container overhead
- **Binary caching**: Pre-built packages download in seconds

### ✅ **Future-Proofing**

- **Package distribution ready**: Foundation for distributing DeepBrain tools
- **Multi-language support**: Can extend to Python, Node.js, Go wrappers
- **Professional branding**: Can evolve into `download.deepbrain.space`

## Risks & Mitigations

### ❌ **Learning Curve**

**Risk**: Team members need to learn Nix\
**Mitigation**: Start with simple flake, provide clear documentation, gradual adoption

### ❌ **Nix Availability**

**Risk**: Not everyone has Nix installed\
**Mitigation**: Fallback instructions for manual tool installation, Nix installer in docs

### ❌ **Package Availability**

**Risk**: Some tools not packaged in nixpkgs\
**Mitigation**: Package popular tools ourselves, contribute back to nixpkgs

## Implementation Plan

### **Week 1-2: Testing & Integration**

1. Add `flake.nix` and `.envrc` to current superconfig monorepo
2. Configure existing tools (cargo-examples) via Nix
3. Test workflow with team members
4. Enable Nix experimental features in documentation

### **Week 3-4: Template Creation**

1. Extract working patterns into reusable git template
2. Document setup process
3. Test template with new project creation
4. Integrate with existing Moon configuration

### **Month 1-2: Future Planning**

1. Research package distribution options
2. Evaluate custom registry vs wrapper strategy
3. Plan integration with product launch timeline

## Integration with Existing Decisions

### **Complements Previous Decisions**

- **Moon Monorepo** ([Infrastructure Decisions](2025-08-06-infrastructure.md)): Nix provides tools, Moon orchestrates tasks
- **Repository Architecture** ([Repository Architecture](2025-08-06-repository-architecture.md)): Each product repo gets identical Nix setup
- **Development Focus** ([Development Focus](2025-08-06-development-focus.md)): Supports rapid development and testing workflow

### **Enables Future Decisions**

- **Package distribution strategy** for custom DeepBrain tools
- **Professional developer experience** for DeepBrain products
- **Multi-language ecosystem reach** through wrapper strategy

## Success Metrics

### **Short Term (2 weeks)**

- [ ] Nix environment works for all team members
- [ ] cargo-examples and Moon work via Nix
- [ ] Faster onboarding for new contributors

### **Medium Term (1 month)**

- [ ] Git template ready for new product repos
- [ ] Documentation complete for Nix workflow
- [ ] Package distribution strategy researched

### **Long Term (3+ months)**

- [ ] All DeepBrain product repos use standardized Nix setup
- [ ] Custom tool distribution via chosen method
- [ ] Professional package registry (if needed)

## Alternative Considered: Docker Development Containers

**Why Nix Instead of Dev Containers:**

- ✅ **Faster**: No container startup time, instant environment activation
- ✅ **Lighter**: No Docker overhead, just environment variables
- ✅ **Better CI integration**: Same tools in dev and CI without container complexity
- ✅ **Cross-platform**: Native performance on all platforms

**Dev Containers Still Available**: Teams can layer Dev Containers on top of Nix if desired.

---

**Final Decision**: Adopt Nix for development environment management with future package distribution capabilities. Start with simple flake.nix in current monorepo, evolve into standardized git templates for all DeepBrain products.

---

_Decision Document Version: 1.0_\
_Last Updated: 2025-08-06_\
_Status: ✅ Approved_\
_Priority: 🟡 Important (2 weeks)_
