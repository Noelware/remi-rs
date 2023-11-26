#!/usr/bin/env bash

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

function remi::crates::service {
    case "$1" in
        azure)
            echo "Microsoft Azure Blob Storage"
            ;;

        gridfs)
            echo "MongoDB GridFS"
            ;;

        gcp)
            echo "Google Cloud Storage"
            ;;

        oci)
            echo "Oracle Cloud Infrastructure Storage"
            ;;

        s3)
            echo "Amazon S3"
            ;;

        fs)
            echo "Local Filesystem"
            ;;

        *)
            echo "===> unknown service [$1]"
            exit 1
            ;;
    esac
}

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

SCRIPT_DIR=$(cd -P "$(dirname $BASH_SRC)/.." >/dev/null 2>&1 && pwd)
tmpl=$(cat "$SCRIPT_DIR/crate-readme.tmpl")
crates_dirs=(
    "gridfs"
    "azure"
    "gcp"
    "oci"
    "s3"
    "fs"
)

echo "===> Rendering crate README templates..."
for crate in "${crates_dirs[@]}"; do
    service=$(remi::crates::service "$crate")
    crate_name="remi-$crate"
    rendered=$(cat "$SCRIPT_DIR/crate-readme.tmpl" | sed -e "s/{{service}}/$service/g" | sed -e "s/{{crate}}/$crate/g")

    ! [ -d "$SCRIPT_DIR/crates/$crate" ] && mkdir -p "$SCRIPT_DIR/crates/$crate"
    echo "$rendered" > "$SCRIPT_DIR/crates/$crate/README.md"
done
