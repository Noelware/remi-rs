# 🐻‍❄️🧶 remi-rs: Asynchronous Rust crate to handle communication between applications and object storage providers
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
{pkgs}:
with pkgs; let
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
  flags =
    if stdenv.isLinux
    then ''-C link-arg=-fuse-ld=mold -C target-cpu=native''
    else "";
in
  mkShell {
    LD_LIBRARY_PATH = lib.makeLibraryPath [openssl];
    nativeBuildInputs =
      [pkg-config]
      ++ (lib.optional stdenv.isLinux [mold lldb])
      ++ (lib.optional stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
        CoreFoundation
        Security
      ]));

    buildInputs = [
      cargo-nextest
      cargo-machete
      cargo-expand
      cargo-deny

      toolchain
      openssl
      git
    ];

    shellHook = ''
      export RUSTFLAGS="${flags} $RUSTFLAGS"
    '';
  }
