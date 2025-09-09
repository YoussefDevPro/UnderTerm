{
  description = "A terminal-based RPG game";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rust-nightly = rust-toolchain.withExtensions [ "rust-src" ];
      in
      with pkgs;
      {
        packages = {
          default = stdenv.mkDerivation {
            name = "under_term";
            src = ./.;
            buildInputs = [
              rust-nightly
              openssl
              pkg-config
              alsa-lib
              udev
            ];
            buildPhase = ''
              cargo build --release --bin under_term
            '';
            installPhase = ''
              mkdir -p $out/bin
              cp target/release/under_term $out/bin
            '';
          };
        };
        devShells.default = mkShell {
          buildInputs = [
            rust-nightly
            openssl
            pkg-config
            alsa-lib
            udev
          ];

          RUST_SRC_PATH = "${rust-nightly}/lib/rustlib/src/rust/library";
        };
      });
}
