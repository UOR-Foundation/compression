<!-- uorc:non-authoritative -->
# M1 source and tooling review, 2026-10-10

This is a work-status review of issues #7–#18, not a product specification. The
repository LexLean graph is the sole authorized specification. References in
issue text to draft sections or appendices do not supply missing semantic data.

The reviewed `main` revision was
`28deb1c6f5d14b983e2dc9baaba013d3ba2d7e71`. Its `src/` tree contains
`Uorc/Specification.lex.tex` and `Uorc/Registry.lex.tex`: charter, evidence
boundaries, research positioning, and the initial conformance register. PR #71
adds the offline reproduction obligations. This change adds the claim policy.
No profile-02 wire format, opcode definitions, reference semantics, resource
event inventory, or product evaluator appears in those modules.

| Issue | Work supported or remaining prerequisite |
| --- | --- |
| #7: complete register | Generation and drift checks already exist. This change adds exact ledger coverage and source-owned dispositions for the current register. The graph does not contain the 59 exact statements required by this issue; this is not completion of #7. |
| #8: authority bindings | Immutable acquisition and provenance records are feasible with current tools. The graph has no complete artifact inventory or selected versions, role interfaces, licenses, and argument contracts for the requested sources. This change imports no facts or artifacts on their behalf. |
| #9: honesty ledger | Implemented by this change with source-owned classification, generated summaries, closed kernel cases, and executed promotion negatives. No new SDK primitive is required. |
| #10: ULEB128 | The graph has no section-7.1 equivalent, exact rejection/error precedence, domain, boundary vectors, or required theorem statements. Generic varint behavior would not establish this issue's exact acceptance criteria. |
| #11: archive frame | Exact field encoding/order, integrity rules, and C.1–C.6 vectors are absent from the graph. Generated binary transport also remains a separate SDK acceptance concern. |
| #12: graph encoding/static validation | The issue names value types and an opcode range, but the source lacks the complete opcode operand/signature table and wire layout needed for exact validation. |
| #13: reference semantics | Pure mathematical authoring is supported. The complete source definitions and theorem inventory needed to establish the requested semantic/resource model are absent. |
| #14: StepMachine | Depends on complete reference semantics, resource events, release schedule, and indexed/checked generated runtime operations. The locked SDK's capability PR does not yet pass those runtime obligations. |
| #15: store and Scan | Needs exact source-owned store/Scan accounting and refinement statements, plus passing generated-runtime store tests at all requested scales. PR #72's required store probe remains unsuccessful. |
| #16: last-use/heap | Needs the complete operand/capture inventory and allocation/root event model to establish exact release safety and peak accounting. These are not defined in the current graph. |
| #17: resource ledger | The closed event vocabulary, recipes, costs, and scale-point expected results referenced by the issue are absent from the current graph. Inventing charges would create an unauthorized specification. |
| #18: envelope/basis | Depends on source-defined raw framing, event costs, limits, cumulative basis accounting, and the checked generated runtime. Those requirements are not yet complete. |

These findings do not classify all remaining M1 work as unsupported by PrismPM
or LexLean. They distinguish missing product definitions and dependent components
from observed SDK failures. New product definitions belong in reviewed LexLean
changes with their own registered scenarios and acceptance evidence. No local
Markdown brief was used as behavioral authority for this implementation.

The exact SDK selected by `prismpm.lock` is retained. PR #72 remains the separate
record of observed capability failures and does not become passing because
this narrower, independently supported M1 issue passes. Neither M0 nor M1 is
reported complete by this review.
