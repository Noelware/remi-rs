#!/bin/bash

# 🐻‍❄️🧶 remi-rs: Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers
# Copyright (c) 2022-2023 Noelware, LLC. <team@noelware.org>
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

set -e

BASH_SRC=${BASH_SOURCE[0]}
while [ -L "$BASH_SRC" ]; do
    target=$(readlink "$BASH_SRC")
    if [[ $target == /* ]]; then
        BASH_SRC=$target
    else
        dir=$(dirname "$BASH_SRC")
        BASH_SRC=$dir/$target
    fi
done

SCRIPTS_DIR=$(cd -P "$(dirname "$BASH_SRC")" >/dev/null 2>&1 && pwd)
ROOT_DIR="${SCRIPTS_DIR}/.."

if ! [ -f "$ROOT_DIR/.remi-version" ]; then
    echo "[remi::scripts] You must be in the root directory to use this script."
    exit 1
fi

version=$(cat "$ROOT_DIR/.remi-version")
crates=(
    "remi_core"
    "remi_support/fs"
    "remi_support/s3"
    "remi_support/gridfs"
    "remi"
)

if [ -z "${CRATES_IO_TOKEN}" ]; then
    echo "[remi::scripts] Missing \`CRATES_IO_TOKEN\` environment variable"
    exit 1
fi

! [ -d "$ROOT_DIR/dist" ] && mkdir $ROOT_DIR/dist
! [ -d "$ROOT_DIR/dist/remi_support" ] && mkdir $ROOT_DIR/dist/remi_support

remi::publish() {
    echo "Now publishing crates with $version!"

    for crate in "${crates[@]}"; do
        echo "===> [$crate] Setting up publishing environment!"
        ! [ -d "$ROOT_DIR/dist/${crate}" ] && cp -R "$ROOT_DIR/${crate}" "$ROOT_DIR/dist/${crate}"

        sed -i "s#version = \"0.0.0-devel.0\"#version = \"$version\"#" $ROOT_DIR/dist/${crate}/Cargo.toml
        echo "===> [$crate] Running \`cargo build\`..."

        (cd $ROOT_DIR/dist/${crate} && cargo build)
        if [ "$?" != "0" ]; then
            echo "===> [$crate] Crate has failed to be built, exiting..."
            exit 1
        fi

        if [ -z "${REMI_DRY_RUN:-}" ]; then
            echo "===> [$crate] Publishing to crates.io!"
            (cd $ROOT_DIR/dist/${crate} && cargo build --token=${CRATES_IO_TOKEN})
        fi
    done
}

remi::publish
