#!/usr/bin/env python3
"""Patch init.py v2 — using string splitting to avoid nested quote issues."""
import re

PATH = "plugins/mcu_init/python/init.py"
with open(PATH, "r") as f:
    text = f.read()

def find_func(name):
    """Return (start_idx, end_idx_exclusive, body)."""
    pattern = r'^def ' + re.escape(name) + r'\([^\n]*\n'
    m = re.search(pattern, text, re.MULTILINE)
    if not m:
        return None
    start = m.start()
    next_def = re.search(r'^(def |class |# ──)', text[m.end():], re.MULTILINE)
    end = m.end() + next_def.start() if next_def else len(text)
    return start, end, text[start:end]

# ── Read replacement text from external files ──
import os
script_dir = os.path.dirname(os.path.abspath(__file__))

def read_patch(name):
    p = os.path.join(script_dir, f"patch_{name}.pyfrag")
    if os.path.exists(p):
        with open(p) as f:
            return f.read()
    return None

# ── Patch mode: use pre-prepared fragments ──
patches = {}
for name in ['makefile_content', 'main_c_content', 'hal_templates', 'create_template', 'action_init']:
    body = read_patch(name)
    if body:
        patches[name] = body
        print(f"[OK] Loaded patch_{name}.pyfrag ({len(body)} chars)")

# Apply patches
for func_name, new_body in patches.items():
    if func_name == 'hal_templates':
        # Insert before create_template
        r = find_func('create_template')
        if r:
            text = text[:r[0]] + new_body + "\n\n" + text[r[0]:]
            print(f"[OK] Inserted hal_templates before create_template")
        continue

    r = find_func(func_name)
    if not r:
        print(f"[FAIL] {func_name} not found")
        continue
    s, e, old = r
    text = text[:s] + new_body + text[e:]
    print(f"[OK] Replaced {func_name}")

with open(PATH, "w") as f:
    f.write(text)
print(f"\nDone. {len(text)} chars")
