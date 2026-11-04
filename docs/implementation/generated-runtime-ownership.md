# Generated runtime ownership investigation

The official `rust-std` renderer shares the recursive heap index through
`std::rc::Rc`; it does not emit an owned `Box<Index>` tree. The generated
`Arena.clone()` calls found here clone fixed-size inline fields and reference
counted handles, not all entries in the old arena. No recursive deep-copy
problem was found in this carrier.

This is an **unqualified diagnostic rendering**, not an accepted export,
release artifact, certificate-B success, or performance result. Production
qualification remains blocked at the upstream preservation-B regression owned
by LexLean #310 / PR #393. This investigation changes no compiler, lock,
production source, or acceptance check.

## Exact program evidence

The retained `/workspace/heap-complete` diagnostic source snapshot matches all
source, configuration, and lock SHA-256 values in
`/workspace/heap-complete-trace/build-manifest.json`. Its semantic ID is
`9e732560fc14eeedf797410183ad012509036347af962bb56f8f2a334080d875`;
source ID is
`569ca6907a525a5b314a5d37d65af7f4f609e7aa475e4d6c8a85297fe0de402a`.
This is the older, low-bit-first snapshot; it does not certify the subsequent
MSB index refinement or the active sequence/store/builder/cursor changes.

A small adapter, `/tmp/heap-ownership-render.rs`, calls only the official
SDK's `Project::load`, `read_current_lock`, `check_project`, `linked_modules`,
`roots`, `lower_root`, `certificate`, and `calculus::render(Profile::Std)`.
Before rendering each program, it asserts that regenerated certificate-A
`text` is byte-identical to the entire saved official
`preserve-src/LexLeanPreserve/C9e732560fc14eeedf797410183ad0125/Rn.lean`.
The exact equality passed for all five roots: allocate, releaseStep,
HeapTests.suite, RuntimeTable.put, and RuntimeTableTests.suite. Thus this
inspects the actual lowered programs embedded in the retained compiler
artifacts, rather than a manually reconstructed tree or semantic model.
The saved `production/compiler-input.json` is also present; its SHA-256 is
`0e8047cdfc8119cdc5b1242bad073126f935bc7e6992dd57825e0dfada9cc8eb`.

The SDK source is clean at public commit
[`71defb4c663aaec5b9a3aab04cbc6a03e7106227`](https://github.com/UOR-Foundation/LexLean/tree/71defb4c663aaec5b9a3aab04cbc6a03e7106227).
The existing official library used was
`/workspace/LexLean/target/debug/deps/liblexlean-050ca82b96ef2634.rlib`, SHA-256
`8193e898d948976a2f666e5a804d99ae000c67ee3bca9100f70a78a99d14e082`.

Diagnostic output is in `/tmp/heap-ownership-render/`:
`R0` through `R4` each have `.program.json`, `.layout.txt`, and `.rs` files.
`evidence.json` records source hashes, output hashes, the equality assertions,
and compile-only validation. The adapter compilation and replay commands are:

```sh
/workspace/.setup/rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/rustc \
  --edition=2021 /tmp/heap-ownership-render.rs \
  --extern lexlean=/workspace/LexLean/target/debug/deps/liblexlean-050ca82b96ef2634.rlib \
  --extern camino=/workspace/LexLean/target/debug/deps/libcamino-bcb620117a469422.rlib \
  -L dependency=/workspace/LexLean/target/debug/deps \
  -o /tmp/heap-ownership-render-bin
/tmp/heap-ownership-render-bin
```

The adapter is diagnostic only; it does not provide a production build path.
Temporary paths depend on this executor and are not portable dependencies.

## Actual Rust carriers and updates

The official lowering layout identifies `R0` ADTs in this order:
Arena, Index, Entry, Node, ImageKind, Handle, RootRole, RuntimeValue,
ReleaseCursor, Allocation, Delta, HeapError, Children, SequenceInfo.
The actual output `/tmp/heap-ownership-render/R0.rs:797` contains:

```rust
#[derive(Clone)]
pub enum Adt0 { // Arena
    C0(Adt1, u64, bool, u64, u64, u64, Adt8),
}
#[derive(Clone)]
pub enum Adt1 { // Index
    C0,
    C1(Adt2),
    C2(std::rc::Rc<Adt1>, std::rc::Rc<Adt1>),
}
#[derive(Clone)]
pub enum Adt2 { // Entry
    C0(Adt3, u64, u64, u64),
}
```

The comments identifying source types above are explanatory annotations;
the `.rs` artifact itself is unchanged official output.
`Node` is `Adt3` at line 814. Its `sequenceBranch` constructor is
`C2(u64, u64, Adt5, Adt5)` and `metadataBranch` is
`C9(u64, Adt5, Adt5)`; `Adt5` is `Handle`, `C0(u64)`, at lines 838–839.
These nodes contain handles to arena entries rather than recursively owned
Node subtrees. `Bytes` is `Rc<[u8]>` at line 647. ReleaseCursor's pending
list is `List<Adt5>` at lines 872–873; the emitted `List<T>::clone` at lines
666–668 clones its `Rc<Vec<T>>` or `Rc<Node<T>>`, without traversing items.

`allocate` is `f0`, line 911. It starts with `f1(v0.clone())` and includes
further arena clones for field reads. Arena's derived Clone copies its
fixed field count, the top Index variant, and the cursor list handle.
An Index branch clones two Rc handles. An Index leaf clones one Entry,
whose Node contains fixed-size fields, handles, and shared Bytes. It does
not recursively traverse descendants or copy image bytes.

`writeIndex` is `f8`, lines 1623–1643. It recursively updates one side and
passes the other side to `compact` (`f21`):

```rust
if f20(v2) {
    f21(f22(v1.clone()), f8(v5, f23(v1), f24(v2), v3))
} else {
    f21(f8(v5, f22(v1.clone()), f24(v2), v3), f23(v1))
}
```

At lines 1985–1994, `compact` emits
`Adt1::C2(Rc::new(v0), Rc::new(v1))` when nonempty. `left`/`right`
(`f22`/`f23`, lines 1997–2027) extract a child with `(*r141).clone()` or
`(*r143).clone()`. This copies one child enum; its recursive children
remain shared Rc handles. Reboxing that enum can allocate a new wrapper
for the unchanged sibling's root, but does not clone its descendants.
The structural allocation/clone work is proportional to the bounded index
path, with a fixed 64-bit path limit, rather than proportional to all old
arena entries. This is a source/output-shape inference, not a native
allocation, RSS, or elapsed-time measurement.

The releaseStep output `R1.rs:1511` similarly shows `readIndex` cloning
`(*r122)` and `(*r123)` before choosing a branch; these are shallow
header copies. Its loop at lines 1480–1528 follows one path.

## Renderer boundary and remaining work

Independent public fixture evidence agrees: LexLean's accepted
`tests/production-export/production-coverage/packages/rust-std/tree_total/src/lib.rs`
at lines 796–825 uses recursive Rc fields and shallow child clones.
The official renderer implementation makes this precise:
`calculus/rust/lower.rs:1006` detects recursive in-place containment;
`field_ty` at line 1228 emits `Type::Rc`; `store` at line 1238 emits its
internal `Expr::Box`. The internal name `Box` does not mean Rust Box:
`calculus/rust/ast.rs:967` prints it as `std::rc::Rc::new(...)`, while
line 966 prints `Unbox` as `(*handle).clone()`.

No new upstream shared-carrier facility is required for the clone concern:
it already exists in the selected official renderer. The separate source
refinement needed for sequential handle prefix sharing is the MSB-first
index path work owned by the active heap implementation; this older
low-bit-first output establishes carrier ownership, not that refinement's
final qualification. After upstream B is fixed, the exact final source
must pass the full pinned SDK acceptance boundary and its accepted Rust
should be inspected again.

`R0.rs` compiled successfully with the absolute Rust 1.97.1 compiler as
an ordinary library (`--edition=2021 --crate-type lib`); no handwritten
runtime was linked. No generated Rust behavior, native memory allocation,
RSS, or timing benchmark was run here. The semantic `ownedBytes` account
must not be presented as a measurement of Rc/index/list overhead or RSS.
