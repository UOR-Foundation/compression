# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: wire-format

  @UC-WIRE-01 @open
  Scenario: The source-declared unshipped archive and machine profile identities have canonical source-owned layouts, strict incremental parsing, exact body lengths and complete EOF checks, without asserting compatibility with an unavailable older draft.
    Given self-contained and conditional raw/program archives, every opcode, malformed fields and all small transport partitions
    When Frame and GraphEmit serialize and the independent FrameParse and GraphParse paths parse the bytes
    Then accepted bytes have the specified canonical meaning; truncation, trailing bytes, non-shortest numbers, invalid graphs and unsupported identities reach typed failures
