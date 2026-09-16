#!/usr/bin/env bash
# check-touch-points.sh BASE HEAD
#
# Every upstream file a change modifies or deletes must have a row in the
# "Upstream touch points" table of MEGAETH-FORK.md, so that a rebase onto a
# new upstream base knows what to re-apply. Files the fork owns (workflows,
# scripts/mega, the fork documents, the vendored op-revm crate and the
# `megaeth` modules) are not upstream files and are skipped; files the change
# adds are new, not touch points.
set -euo pipefail
base=$1
head=$2
table=MEGAETH-FORK.md
fail=0
while IFS= read -r file; do
  case "$file" in
    .github/*|scripts/mega/*|MEGAETH-FORK.md|REVIEW.md|crates/op-revm/*) continue ;;
    */megaeth.rs|*/megaeth/*) continue ;;
  esac
  if ! grep -Fq "\`$file\`" "$table"; then
    echo "::error file=$file::modified upstream file has no row in the touch-point table of $table"
    fail=1
  fi
done < <(git diff --name-only --diff-filter=MD "$base...$head")
if [ "$fail" -eq 0 ]; then
  echo "every modified upstream file has a touch-point row"
fi
exit "$fail"
