<!-- uorc:non-authoritative -->
# uorc factory

The release factory keys every CI and release obligation on its own declared
inputs, so a change re-runs only the obligations that read it. It follows the
PrismPM factory route (PrismPM #566, #579-#581, #586): the same obligation
identity text (`prism-factory/obligation/1`), receipts on a `factory-receipts`
branch, refutation of hermetic failures, retried fetches, and a 16-level weight
that orders remaining work. Here the decisions are a small Node host
(`scripts/factory.mjs`, tests in `scripts/factory.test.mjs`) over the graph in
`graph.json`, not a kernel-checked model: uorc's graph is sixteen rows.

## The graph

| Obligation | Kind | Reads | Carried out by |
| --- | --- | --- | --- |
| `rest-<runner>` (2) | fetch | everything | bootstrap `template-gate` |
| `artifacts` | hermetic | the LexLean project: `src/` and its project files | bootstrap `template-gate` |
| `varint` | hermetic | the import closure of `Uorc.Varint`, `Uorc.VarintContract`; the project files, xtask, scripts | bootstrap `template-gate` |
| `integrity` | hermetic | the import closure of every `Uorc.Integrity` module (SHA-256, the bounded chain and their contract); `standards/`; the project files, xtask, scripts | bootstrap `template-gate` |
| `claims`, `reproducibility` | hermetic | the formal set | bootstrap `template-gate` |
| `offline` | fetch | the formal set | offline-validation |
| `export` | fetch, host | the import closure of `Uorc.Client` (client/src over src), the LexLean project files, the export scripts | `node scripts/client-export.mjs` on a workstation |
| `pages` | fetch | the client set, `crates/uorc-wasm/`, `web/`, the brand files, `scripts/site.mjs`, `scripts/page-acceptance.mjs`, its workflow | pages-client `acceptance` (UC-WEB-01) |
| `gate` | gate | the identities of the seven bootstrap legs and `pages` | proved when all eight are |
| `native-<target>` (2), `wasm` | fetch | the client set (+ `crates/uorc-wasm/`) | release `native`, `wasm` |
| `bench` | fetch | `scripts/bench.mjs`, release.yml, `native-x86_64` | release `bench` |
| `frontier` | hermetic | the client set, `crates/frontier/`, `frontier.toml`, release.yml; needs `native-x86_64` | release `frontier`: decides the closed claims of the release's codec point over every κ-named corpus in `frontier.toml` |
| `publish` | fetch | everything, `@mode`, `@run` | release `publish`, `pages` |

The formal set is the tree without the product client and the release plumbing.
The legs that build xtask (every formal leg but `artifacts`) read the product
crates' Cargo manifests only (`manifest:` inputs): the closure copy gives their
targets empty files, so cargo resolves the workspace without reading their code.
The client set is what `cargo build -p uorc` reads: the workspace manifests and
lock, the toolchain, `.cargo/`, `crates/uorc/` and `generated/rust/`.

A registry or claims edit therefore re-runs `artifacts`, `claims` and
`reproducibility` (they read all of `src/` or the model), not `integrity` or
`varint`; a change to `model/`, `features/` or `CONFORMANCE.md` does not re-run
`artifacts`, `integrity` or `varint`.

## Closure copy

Before a bootstrap leg runs, `factory.mjs materialize` deletes every tracked
file the leg does not declare (PrismPM runs each obligation in a copy of its
closure). An undeclared input fails the leg instead of proving a receipt it
should not. Adding the copy found one: `just model-write` reads `README.md`,
which the old κ scope left out.

## Identity

An obligation's identity is the SHA-256 (hex) of

```text
prism-factory/obligation/1
name <name>
kind hermetic|fetch|gate
environment <runner>
run <command>
input <prefix>            one line per declared input (`!prefix` excludes, `@key` is a context,
                          `closure:ROOTS@DIRS` an import closure, `manifest:PATH` a Cargo manifest only)
needs <name> <identity>   one line per need
closure <sha256 of the closure>
```

where the closure is `file <mode>:<object> <path>` for each tracked entry the
inputs cover, in tree order, then `file context:<value> @<key>` per context. An
entry is covered when the longest inclusion it starts with is longer than the
longest exclusion it starts with.

## Routes

- **proved**: a passing receipt for this identity exists. The job is skipped;
  release stages take their artifact from the run that proved them.
- **refuted**: a hermetic obligation failed on exactly these inputs. It fails in
  seconds instead of an hour later. Re-running the job (attempt 2+) observes it again.
- **blocked**: something it needs is not proved and will not run.
- **observe**: run it.

Each job uploads only a `factory-outcome-*` artifact. `factory.yml` runs from
the default branch after a same-repository run completes and turns outcomes into
receipts, so no pull-request job holds a write token. Receipts for stages with
artifacts expire a day before the artifact does.

## Releasing

1. Bump `[workspace.package] version` in `Cargo.toml` (publish refuses a tag that
   does not name it).
2. Optional: dispatch **release** with `mode: rehearsal` and `ref: main`. It
   builds, benchmarks and packages without publishing.
3. Tag `vX.Y.Z`. The tag run requires the bootstrap gate on that tree, reuses
   every stage the rehearsal (or an earlier attempt) proved, packages, publishes,
   and dispatches the Pages deployment on the default branch.

## Locally

```sh
node scripts/factory.mjs plan              # every obligation's route for HEAD, and the schedule
node scripts/factory.mjs export-check      # is generated/rust exported from the current client closure?
node scripts/client-export.mjs             # exits at once when it is; --force exports anyway
node --test scripts/factory.test.mjs
```

## Release

Merging a workspace version bump to the default branch is the release. After
each receipts run, factory.yml's `release` job runs `factory.mjs release-step`
on the default branch's head:

- **done**: `v<version>` is already tagged.
- **wait**: the bootstrap gate is not proved on the head yet (or a release run
  finished without proving every stage; fix it, then dispatch factory.yml).
- **rehearse**: the gate is proved but `native-*`, `wasm`, `bench` or
  `frontier` is not; it dispatches a release rehearsal of the head.
- **tag**: everything is proved; it creates the annotated tag `v<version>` on
  the head and dispatches release.yml from the default branch with
  `ref=v<version> mode=publish`, which only packages, publishes and deploys
  Pages from the rehearsal's artifacts.

Only a run that is not itself a release starts a rehearsal, so a failing
rehearsal is not retried in a loop. Pushing a `v*` tag by hand still works.

## Merge train

Branch protection on the default branch is strict: a request merges only when
its head contains the default branch and `acceptance / ubuntu-24.04` and
`acceptance / ubuntu-24.04-arm` passed on that head. Every merge therefore
makes every other request stale. The merge train does the update, test and
merge cycle instead of a person or an agent.

To ride it, add the `merge-train` label to a non-draft request into the default
branch. Riders go in the order their label was applied (re-labelling joins at
the back), then by number. A draft waits until it is ready; a stacked request
(based on another branch) waits until it is retargeted. Only same-repository
branches can ride: the train must push to them.

factory.yml's `merge-train` job runs `factory.mjs merge-train` after each
bootstrap run completes, when a request is labelled, and on dispatch. Its first
run creates the label when the repository has none. It looks only at the head
of the train:

- **update**: the head lacks commits of the default branch. It merges the
  default branch into the request (the REST update-branch call), then
  dispatches bootstrap on the request's branch.
- **dispatch**: no bootstrap run has tested the head commit; it dispatches one.
- **wait**: a bootstrap run or a required check on the head is still going. Its
  completion moves the train again.
- **merge**: the head contains the default branch and every required check
  passed. It squash-merges the request (`<title> (#<number>)`, the description
  as the body), dispatches bootstrap on the default branch, and moves on to the
  next rider.
- **drop**: a required check failed, the update conflicts, the merge was
  refused, or bootstrap ran on the head without reporting the checks. It takes
  the label off, comments once with the reason, and moves on. Fix it and label
  it again.

A push or merge made with the workflow token starts no workflow, so the train
dispatches each bootstrap run it needs; the dispatched run's checks satisfy the
required checks on that commit. The default branch's dispatched run yields the
receipts that the release step decides on, so merging a version bump by train
still releases it. The workflow token has no `workflows` permission, so GitHub
refuses it an update or merge that would change `.github/workflows/`; the train
drops such a request with GitHub's message, and it is merged by hand.
