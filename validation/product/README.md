<!-- uorc:non-authoritative -->
# Product component diagnostic

The source inventory and trace owners are in `src/Uorc/Product/Scenarios.lex.tex`.
Run the component check with an installed official compiler and Lean toolchain:

```sh
python3 scripts/product-control-validation.py --lexlean lexlean --lean lean
```

For an explicitly separate newer-compiler language-1.1 compatibility diagnostic:

```sh
python3 scripts/product-control-validation.py --lexlean lexlean --lean lean --diagnostic-compatibility
```

The latter records the two pre-existing executable-annotation omissions in its
temporary copy. It changes no repository source, lock, proof policy or behavior.
Every run executes the full component inventory, all four planted defects, and the
restored control. The harness and gate contain no product implementation.

`evidence.json` is a component execution receipt, not pinned-SDK, native, browser,
compression-effectiveness or release evidence. `just vv` in the exact locked SDK
remains the full acceptance boundary. See the linked source and
`docs/governance/product-api-view-evidence.md` for integration scope.

`host-contract.json` is generated from `HostContract.contract`; regenerate it with
`python3 scripts/product-host-projection.py --write`. The component gate checks
that actual projection before executing the source controls.

`root-probe/result.json` is the actual official diagnostic export manifest for
the extracted `Api.sameResource` root; `root-probe/evidence.json` records artifact
replay, the rejected single-prefix request and five native controls. Its scope
is separate from the complete Workspace/View and pinned-SDK acceptance.
