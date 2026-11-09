#!/bin/sh
# Regenerates client/src/Uorc/Codec/*.lex.tex. The generators are writing aids; the .lex.tex files are the source.
set -e
cd "$(dirname "$0")"
C=../src/Uorc/Codec
python3 gen_table.py; python3 fixmul.py $C/Table.lex.tex; python3 finish.py dec $C/Table.lex.tex
python3 gen_dec.py; python3 fixmul.py $C/LzDecode.lex.tex; python3 finish.py dec $C/LzDecode.lex.tex
python3 gen_enc.py; python3 fixmul.py $C/LzEncode.lex.tex; python3 finish.py enc $C/LzEncode.lex.tex
python3 gen_archive.py; python3 fixmul.py $C/Archive.lex.tex
