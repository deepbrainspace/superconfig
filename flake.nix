{
  description = "DeepBrain SuperConfig development environment";
  
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
              # Rust toolchain from rust-toolchain.toml (if exists)
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml or rust-bin.stable.latest.default)
              
              # Development tools for cargo templates
              cargo-generate
              
              # Moon task runner - we'll install via binary since not in nixpkgs yet
              # proto - we'll need to install manually for now
              
              # Basic development tools
              git
              curl
              
              # System dependencies
              pkg-config
              openssl
            ];
            
            shellHook = ''
              echo "🚀 DeepBrain SuperConfig development environment activated!"
              echo "Available tools:"
              echo "  • cargo-generate: $(cargo generate --version 2>/dev/null || echo "not found")"
              echo "  • git: $(git --version)"
              
              # Check if moon is available
              if command -v moon >/dev/null 2>&1; then
                echo "  • moon: $(moon --version)"
              else
                echo "  • moon: Installing via proto..."
                # Install proto if not available
                if ! command -v proto >/dev/null 2>&1; then
                  echo "  • Installing proto..."
                  curl -fsSL https://moonrepo.dev/install/proto.sh | bash
                  export PATH="$HOME/.proto/bin:$PATH"
                fi
                # Install moon
                proto install moon
                export PATH="$HOME/.proto/bin:$PATH"
                echo "  • moon: $(moon --version 2>/dev/null || echo "installation in progress")"
              fi
            '';
            
            env = {
              RUST_BACKTRACE = "1";
            };
          };
        });
    };
}