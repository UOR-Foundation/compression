#!/usr/bin/env bash
# Content-addressed CI receipts: each slow obligation runs once per content
# identity of its inputs, and a later run with the same identity reuses that
# result instead of proving it again.
#
#   kappa                    print κ, the SHA-256 of the tracked tree minus the
#                            paths no formal leg reads (the product client and
#                            release plumbing below)
#   lookup <obligation>      print kappa=…, proved=true|false and run=… for
#                            $GITHUB_OUTPUT. proved=true when an unexpired
#                            receipt artifact for this obligation and κ exists
#                            from a same-repository run
#   record <obligation> <κ>  write .tmp/receipt/receipt.json for upload
#
# A receipt artifact is uploaded only after its obligation passed, so its
# existence is the proof. Any lookup failure means "not proved": the work runs.
set -euo pipefail

# The product client and release plumbing. Product changes are checked by the
# `rest` leg on every run; none of these paths is read by the formal legs.
EXCLUDE='^(client/|crates/uorc/|crates/uorc-wasm/|generated/|web/|Cargo\.toml$|Cargo\.lock$|README\.md$|scripts/bench\.mjs$|scripts/client-export\.mjs$|scripts/ci-receipt\.sh$|\.github/workflows/release\.yml$)'

kappa() {
  git ls-tree -r --full-tree HEAD | awk -F'\t' -v re="$EXCLUDE" '$2 !~ re' | sha256sum | cut -c1-64
}

case "${1:-}" in
  kappa)
    kappa
    ;;
  lookup)
    obligation=${2:?obligation}
    k=$(kappa)
    echo "kappa=$k"
    run=$(gh api "repos/$GITHUB_REPOSITORY/actions/artifacts?name=receipt-$obligation-$k&per_page=100" \
      --jq '[.artifacts[] | select(.expired | not) | select(.workflow_run.head_repository_id == .workflow_run.repository_id)] | first | .workflow_run.id // empty' \
      2>/dev/null || true)
    if [[ $run =~ ^[0-9]+$ ]]; then
      echo "proved=true"
      echo "run=$run"
      echo "::notice title=$obligation proved by receipt::κ $k passed in run $GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$run; its inputs are unchanged, so it is not run again." >&2
    else
      echo "proved=false"
    fi
    ;;
  record)
    obligation=${2:?obligation}
    k=${3:?kappa}
    mkdir -p .tmp/receipt
    printf '{"schema":"uorc/ci-receipt/1","obligation":"%s","kappa":"%s","sha":"%s","run":"%s","attempt":"%s"}\n' \
      "$obligation" "$k" "$(git rev-parse HEAD)" "${GITHUB_RUN_ID:-}" "${GITHUB_RUN_ATTEMPT:-}" > .tmp/receipt/receipt.json
    ;;
  *)
    echo "usage: $0 kappa | lookup <obligation> | record <obligation> <kappa>" >&2
    exit 64
    ;;
esac
