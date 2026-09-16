#!/usr/bin/env bash
# Print the cargo arguments that redirect the twelve revm crates at a checkout
# of this fork, reading the crate table from scripts/mega/crates.txt.
#
# Usage: cargo check --workspace $(scripts/mega/patch-args.sh /abs/path/to/revm)
#
# Output is two whitespace-separated tokens per crate, so unquoted command
# substitution splits it into argv exactly as cargo expects:
#
#   --config
#   patch.crates-io.revm.path="/abs/path/to/revm/crates/revm"
#
# All twelve crates are patched together. Patching only `revm` pulls the fork's
# transitive crates through path dependencies while other workspace members
# still resolve the crates.io ones, and the build ends up with two copies.

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
table="$script_dir/crates.txt"

if [ "$#" -ne 1 ]; then
    echo "usage: $(basename "$0") <fork-root>" >&2
    exit 2
fi

fork_root=$1

case $fork_root in
    /*) ;;
    *)
        echo "$(basename "$0"): fork root must be an absolute path: $fork_root" >&2
        exit 2
        ;;
esac

# The caller splits our output on whitespace, so a path containing whitespace
# would silently produce a wrong argv; a quote or backslash would break the
# TOML value.
case $fork_root in
    *[[:space:]]*|*\"*|*\\*)
        echo "$(basename "$0"): fork root must not contain whitespace, quotes or backslashes: $fork_root" >&2
        exit 2
        ;;
esac

# Strip a trailing slash so the printed paths do not contain a double slash.
fork_root=${fork_root%/}

if [ ! -d "$fork_root" ]; then
    echo "$(basename "$0"): fork root does not exist: $fork_root" >&2
    exit 2
fi

if [ ! -f "$table" ]; then
    echo "$(basename "$0"): crate table not found: $table" >&2
    exit 2
fi

count=0
# `|| [ -n "$crate" ]` keeps a final record that lacks its newline.
while read -r crate version dir || [ -n "$crate" ]; do
    case $crate in
        ''|\#*) continue ;;
    esac
    if [ -z "$version" ] || [ -z "$dir" ]; then
        echo "$(basename "$0"): malformed record in $table: $crate" >&2
        exit 2
    fi
    if [ ! -d "$fork_root/$dir" ]; then
        echo "$(basename "$0"): missing crate directory: $fork_root/$dir" >&2
        exit 2
    fi
    printf -- '--config\npatch.crates-io.%s.path="%s/%s"\n' "$crate" "$fork_root" "$dir"
    count=$((count + 1))
done < "$table"

if [ "$count" -eq 0 ]; then
    echo "$(basename "$0"): crate table is empty: $table" >&2
    exit 2
fi
