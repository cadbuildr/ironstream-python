#!/usr/bin/env bash
# Build the ironstream Python binding as a Pyodide (WebAssembly) wheel.
#
# The result runs in the browser: pyodide.loadPackage(<wheel url>) then
# `import ironstream`. Used by the live demo on the IronStream project page
# (ironstream repo, docs/).
#
# Toolchain notes (each pin matters — discovered the hard way):
#  * pyodide xbuildenv 0.29.4  -> targets Pyodide 0.29.x, CPython 3.13, emscripten 4.0.9
#  * host python 3.13 venv     -> must match the xbuildenv's CPython minor
#  * rust nightly-2025-02-01   -> pyodide-build passes -Z flags (link-native-libraries,
#                                 emscripten-wasm-eh); newer nightlies removed
#                                 -Zemscripten-wasm-eh, stable rejects -Z entirely
#  * wasm-eh sysroot overlay   -> pyodide ships a prebuilt std with wasm exceptions for
#                                 exactly (emcc 4.0.9, nightly-2025-02-01); it must
#                                 REPLACE the rustup-installed target dir, not merge
#  * emscripten.py patch       -> emcc 4.0.9 rejects rust legacy-mangled wasm exports
#                                 ('$'/'.' chars) with "invalid export name"; the names
#                                 are harmless (JS glue uses bracket access), so the
#                                 validator is patched to skip them (fixed upstream later)
#  * kernel bsp.rs             -> run_csg() has a #[cfg(target_family = "wasm")] inline
#                                 path because wasm has no threads
set -euo pipefail

PYODIDE_XBUILDENV=0.29.4
EMSCRIPTEN_VERSION=4.0.9
RUST_TOOLCHAIN=nightly-2025-02-01
VENV=/tmp/pyodide-venv313
EMSDK="${EMSDK_DIR:-$HOME/emsdk}"
REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# --- host venv + pyodide-build ---------------------------------------------
if [ ! -x "$VENV/bin/pyodide" ]; then
  uv venv --python 3.13 "$VENV"
  uv pip install --python "$VENV/bin/python" pyodide-build
fi
export PATH="$VENV/bin:$PATH"
pyodide xbuildenv install "$PYODIDE_XBUILDENV" 2>/dev/null || true

# --- emsdk ------------------------------------------------------------------
if [ ! -d "$EMSDK" ]; then
  git clone https://github.com/emscripten-core/emsdk.git "$EMSDK"
fi
(cd "$EMSDK" && ./emsdk install "$EMSCRIPTEN_VERSION" && ./emsdk activate "$EMSCRIPTEN_VERSION")
# shellcheck disable=SC1091
source "$EMSDK/emsdk_env.sh"

# patch emcc's export-name validator (rust legacy mangling contains '$' and '.')
python3 - "$EMSDK/upstream/emscripten/tools/emscripten.py" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = "    if not n.isidentifier():\n      exit_with_error(f'invalid export name: {n}')"
new = ("    if not n.replace('$', '_').isidentifier():"
       "  # tolerate rust legacy-mangled exports; JS glue uses bracket access\n      continue")
if old in s:
    open(p, 'w').write(s.replace(old, new))
    print("emscripten.py: patched export-name validator")
elif "tolerate rust legacy-mangled exports" in s:
    print("emscripten.py: already patched")
else:
    sys.exit("emscripten.py: expected validator code not found — emscripten version changed?")
PY

# --- rust toolchain + pyodide's wasm-eh std sysroot -------------------------
rustup toolchain install "$RUST_TOOLCHAIN" --profile minimal
rustup target add --toolchain "$RUST_TOOLCHAIN" wasm32-unknown-emscripten
SYSROOT_URL="https://github.com/pyodide/rust-emscripten-wasm-eh-sysroot/releases/download/emcc-${EMSCRIPTEN_VERSION}_${RUST_TOOLCHAIN}/emcc-${EMSCRIPTEN_VERSION}_${RUST_TOOLCHAIN}.tar.bz2"
TC_LIB="$(rustc +"$RUST_TOOLCHAIN" --print sysroot)/lib/rustlib"
if [ ! -f "$TC_LIB/wasm32-unknown-emscripten/.pyodide-wasm-eh" ]; then
  curl -sL -o /tmp/rust-em-sysroot.tar.bz2 "$SYSROOT_URL"
  rm -rf "$TC_LIB/wasm32-unknown-emscripten"
  tar xjf /tmp/rust-em-sysroot.tar.bz2 -C "$TC_LIB"
  touch "$TC_LIB/wasm32-unknown-emscripten/.pyodide-wasm-eh"
fi

# --- build ------------------------------------------------------------------
cd "$REPO_DIR"
rm -rf dist
export RUSTFLAGS="-C link-arg=-sSIDE_MODULE=2 -Z link-native-libraries=yes -Z emscripten-wasm-eh"
RUSTUP_TOOLCHAIN="$RUST_TOOLCHAIN" pyodide build

echo
echo "wheel(s):"
ls -la dist/
