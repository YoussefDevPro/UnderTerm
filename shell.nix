{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/24.05.tar.gz") { overlays = [ (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")) ]; } }:

let
  # Define the specific nightly toolchain we want to use
  toolchain = pkgs.rust-bin.nightly."2024-07-15";

  # Get the compiler from the toolchain and add our cross-compilation targets
  rust = toolchain.default.override {
    targets = [
      "x86_64-unknown-linux-musl"
      "aarch64-unknown-linux-musl"
    ];
  };

  # Get the matching source code from the same toolchain
  rust_src = toolchain.rust-src;
in
pkgs.mkShell {
  # The build tools and libraries we need
  buildInputs = [
    rust
    rust_src
    pkgs.pkg-config
    pkgs.alsa-lib
    pkgs.openssl # A common dependency, good to have
    pkgs.musl
  ];

  # Set the appropriate linker for each MUSL target
  # This helps cargo find the right C compiler and linker
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER = "aarch64-linux-musl-gcc";
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "musl-gcc";

  # Crucially, set the pkg-config path inside the Nix shell.
  # This was the step that was failing in all previous attempts.
  PKG_CONFIG_PATH = "${pkgs.alsa-lib}/lib/pkgconfig";
}
