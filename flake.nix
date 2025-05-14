{
  description = "swiftfetch - a fastfetch alternative written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    let
      # First define outputs without system for direct access
      outputsForAllSystems = flake-utils.lib.eachDefaultSystem (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          rustVersion = pkgs.rust-bin.stable.latest.default;
          
          swiftfetch = pkgs.rustPlatform.buildRustPackage rec {
            pname = "swiftfetch";
            version = "0.1.3";
            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [
              rustVersion
              pkg-config
            ];

            preBuild = ''
              export HOME=$(mktemp -d)
            '';

            postInstall = ''
              mkdir -p $out/bin
              mv $out/bin/swiftfetch $out/bin/.swiftfetch-wrapped

              cat > $out/bin/swiftfetch <<EOF
              #!${pkgs.bash}/bin/bash
              CONFIG_DIR="\$HOME/.config/swiftfetch"

              mkdir -p "\$CONFIG_DIR"

              install_if_missing() {
                local file="\$1"
                local src="\$2"
                if [ ! -f "\$CONFIG_DIR/\$file" ]; then
                  cp "\$src" "\$CONFIG_DIR/\$file"
                fi
              }

              install_if_missing "config.toml" "${./config/config.toml}"
              install_if_missing "ascii.txt" "${./config/ascii.txt}"

              exec "$out/bin/.swiftfetch-wrapped" "\$@"
              EOF

              chmod +x $out/bin/swiftfetch
            '';

            meta = with pkgs.lib; {
              description = "A fast and efficient fetch utility written in Rust";
              license = licenses.mit;
              platforms = platforms.linux;
            };
          };
        in
        {
          packages = {
            default = swiftfetch;
            swiftfetch = swiftfetch;
          };

          apps.default = {
            type = "app";
            program = "${swiftfetch}/bin/swiftfetch";
          };

          devShell = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustVersion
              cargo
              rustfmt
              clippy
            ];
            RUST_SRC_PATH = "${rustVersion}/lib/rustlib/src/rust/library";
          };
        }
      );
    in
    outputsForAllSystems // {
      # Add top-level package reference
      swiftfetch = outputsForAllSystems.packages.${builtins.currentSystem}.default;
    };
}