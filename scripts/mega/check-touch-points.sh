#!/usr/bin/env bash
# check-touch-points.sh BASE HEAD [UPSTREAM]
#
# Every upstream file a change modifies or deletes must have a row in the
# "Upstream touch points" table of MEGAETH-FORK.md, so that a rebase onto a
# new upstream base knows what to re-apply. Files the fork owns (workflows,
# scripts/mega, the fork documents and the `megaeth` modules) are not upstream
# files and are skipped; files the change adds are new, not touch points.
set -euo pipefail
base=$1
head=$2
# Optional: the upstream base revision (the tag in scripts/mega/base.txt). A
# modified file whose content at $head equals upstream's is a restoration, not
# a touch point, and needs no row.
upstream=${3:-}
doc=MEGAETH-FORK.md
# Only the table section counts: a path mentioned elsewhere in the document
# (a rule, an example) must not satisfy the check.
table=$(awk '/^## Upstream touch points/{f=1; next} /^## /{f=0} f' "$doc")
fail=0
while IFS= read -r file; do
  case "$file" in
    .github/*|scripts/mega/*|MEGAETH-FORK.md|REVIEW.md) continue ;;
    */megaeth.rs|*/megaeth/*) continue ;;
  esac
  if [ -n "$upstream" ] && git cat-file -e "$upstream:$file" 2>/dev/null \
     && [ "$(git rev-parse "$upstream:$file")" = "$(git rev-parse "$head:$file" 2>/dev/null)" ]; then
    echo "::notice file=$file::restored to its upstream content; no touch-point row needed"
    continue
  fi
  if ! grep -Fq "\`$file\`" <<< "$table"; then
    echo "::error file=$file::modified upstream file has no row in the touch-point table of $doc"
    fail=1
  fi
done < <(git diff --name-only --diff-filter=MD "$base...$head")
if [ "$fail" -eq 0 ]; then
  echo "every modified upstream file has a touch-point row"
fi
exit "$fail"
