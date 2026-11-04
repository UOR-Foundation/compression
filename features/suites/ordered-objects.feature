# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: ordered-objects

  @UC-OBJ-01 @open
  Scenario: Decoded output preserves the exact ordered object boundary vector, including zero objects and distinct zero-length objects.
    Given ObjectSplit lengths with binary data, internal empty objects, partial input, excess bytes and empty transport chunks
    When the demultiplexer consumes only demanded input and emits bounded byte spans until one-shot EOF
    Then ordered bytes and empty boundaries are preserved; early EOF and trailing bytes fail; emitted pieces remain staged rather than implicitly published
