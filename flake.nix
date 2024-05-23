# 🐻‍❄️🧶 remi-rs: Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers
# Copyright (c) 2022-2024 Noelware, LLC. <team@noelware.org>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
{
  description = "🐻‍❄️🧶 Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustflags =
        if pkgs.stdenv.isLinux
        then ''-C link-arg=-fuse-ld=mold -C target-cpu=native $RUSTFLAGS''
        else "$RUSTFLAGS";

      libraries = with pkgs; [openssl];
    in {
      devShells.default = pkgs.mkShell {
        nativeBuildInputs =
          [pkgs.pkg-config]
          ++ (with pkgs; lib.optional stdenv.isLinux [mold lldb])
          ++ (with pkgs; lib.optional stdenv.isDarwin [darwin.apple_sdk.frameworks.CoreFoundation]);

        buildInputs = [
          pkgs.cargo-nextest
          pkgs.cargo-machete
          pkgs.cargo-expand
          pkgs.cargo-deny

          pkgs.openssl
          rust
        ];

        shellHook = ''
          export RUSTFLAGS="${rustflags}"
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH"
        '';
      };
    });
}
