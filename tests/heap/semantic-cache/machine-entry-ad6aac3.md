<!-- uorc:non-authoritative -->
# Exact machine entry module diagnostic cache

machine-entry-ad6aac3.json contains the full unchanged official parsed_module
for RuntimeMachine at public source commit
ad6aac30f6c73e5d9907c630054b30e58f2deb98. The49284-byte source has SHA256
7dc5661651faeac4d267157303971d77601be59d3ab2ae6ce1c6ab2f8ef20e81
and Git blob9b3bedce18d8b81173c99212bda22eb29588a749.

The retained official prismpm-semantic-source reader parsed the exact source
twice with byte-identical output, and its writer restored the source bytes
exactly. Reader commit/binary/source/package identities, imports and all31
definition executable/axiom/production markings accompany the full IR. The
reader directly invokes the official SDK; no notation or declaration model was
reconstructed. This file is a diagnostic cache and never a production input.
No linking, kernel check, export or release qualification is inferred.

Reproduce the parse using the bound official helper:

```sh
prismpm-semantic-source read < src/Uorc/Core/RuntimeMachine.lex.tex
```
