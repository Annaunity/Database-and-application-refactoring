{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, utils, fenix }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        rust-toolchain = with fenix.packages.${system};
          combine (with complete; [
            rustc
            rust-src
            cargo
            clippy
            rustfmt
            rust-analyzer
            miri
          ]);
      in {
        devShell = (pkgs.mkShell.override { stdenv = pkgs.clangStdenv; }) rec {
          buildInputs = with pkgs; [
            cargo-expand
            cargo-nextest
            clangStdenv
            mold
            pkg-config
            rust-toolchain
            rustPlatform.bindgenHook
            sqlx-cli
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.llvmPackages.clangUseLLVM}/bin/clang";
          RUSTFLAGS = builtins.concatStringsSep " " [
            "-Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold"
            "-Zshare-generics=y"
            "-Zthreads=0"
          ];
          RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = 1;
        };
      }
    );
}
