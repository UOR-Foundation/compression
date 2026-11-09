# Replaces the state type tokens after fixmul.py has rewritten multiplications.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from state import Layout
kind, path = sys.argv[1], sys.argv[2]
L = Layout('Dec', ['mem'], [], {}) if kind == 'dec' else Layout('Enc', ['mem'], [], {})
text = L.finish(open(path).read()).rstrip('\n') + '\n'
open(path, 'w').write(text)
