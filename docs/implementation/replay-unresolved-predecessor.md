<!-- uorc:non-authoritative -->

# Unresolved predecessor continuation

`Uorc.Search.Replay` is authoritative. Its diagnostic Graph/Rope replay is not the production owned runtime or a released minimum authority.

A predecessor rejected for a non-excluding reason now enters `Phase.unresolvedTrial` with the actual rejected trial and next-word continuation retained. Re-entering that cursor returns the same incomplete outcome, without adding failures twice, incrementing the checker prefix, rerunning the trial, advancing enumeration or issuing coverage. Cancellation retains the rejected trial state and its usage/ledger. An explicit fresh replay is required rather than treating resource refusal as exclusion.

The source includes a parameterized exact continuation law. Independent finite source-JSON diagnostics exercised all ten operational-error constructors, three repeated observations per case, exact failure/counter retention and cancellation. Restoring only the previous drop-to-next-word assignment reproduced the audit's local completion witness, and the restored implementation rejected it.

That witness uses opaque winner/raw records to isolate a local structural transition. It is not evidence of a reachable end-to-end archive or a released false-minimum exploit. Existing Certification already treated incomplete replay as terminal. Ordinary proof, locked-SDK acceptance, canonical membership enforcement and the reference/owned-runtime bridge remain separate obligations.
