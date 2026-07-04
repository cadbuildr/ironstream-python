# ironstream (Python bindings)

PyO3 bindings for the [IronStream](https://github.com/cadbuildr/ironstream) CAD
geometry kernel — a from-scratch, zero-dependency Rust port of OpenCascade.

This is one of three consumption paths off the IronStream kernel (the other two
— native Rust and a WASM/TypeScript export — live elsewhere). It mirrors the
kernel's OCCT-shaped prelude, in the spirit of
[pyOCCT](https://github.com/trelau/pyOCCT): symbols are available both flat and
under OCCT-style submodules.

```python
import ironstream as ist

box = ist.make_box(ist.Pnt(0, 0, 0), 20, 20, 20)
hole = ist.make_cylinder(5, 20)
part = ist.cut(box, hole)
print(part.volume())

with open("part.stl", "wb") as f:
    f.write(ist.write_binary_stl(part.mesh()))
```

pyOCCT-style namespaces are also available:

```python
from ironstream.gp import Pnt
from ironstream.prim import make_box
from ironstream.algo import cut
```

## Layout

| Namespace              | Contents                                                            |
| ---------------------- | ------------------------------------------------------------------- |
| `ironstream.gp`        | `Pnt` (`Vec3`, `Point3`), `Ax1`, `Trsf`                             |
| `ironstream.topods`    | `Vertex`, `Wire`, `Face`, `Solid`, `Compound`                       |
| `ironstream.mesh`      | `TriMesh`, `BBox`                                                    |
| `ironstream.prim`      | `make_box/cylinder/cone/sphere/torus/prism/revol`, `MeshParams`     |
| `ironstream.builder`   | `make_polygon`, `make_face`, `make_face_with_holes`, `transform`    |
| `ironstream.algo`      | `fuse`, `cut`, `common`, `fuse_all`                                  |
| `ironstream.io`        | `write_binary_stl`, `write_ascii_stl`, `write_step`                 |

## Development

Requires a Rust toolchain and [maturin](https://www.maturin.rs/). The kernel is
pulled in as a local path dependency (`../ironstream`), so clone both repos as
siblings.

```bash
uv venv
maturin develop            # builds the kernel + binding, installs into the venv
python -c "import ironstream; print(ironstream.make_box(ironstream.Pnt(0,0,0),1,1,1).volume())"
```

Build a distributable wheel with `maturin build --release`.

## Pyodide / WebAssembly

The binding also builds as a **Pyodide wheel** (~140 KB) that runs the whole
kernel in the browser — this powers the live demo on the IronStream project
page:

```bash
chmod +x scripts/build_pyodide_wheel.sh
./scripts/build_pyodide_wheel.sh    # -> dist/cadbuildr_ironstream-*-emscripten_*_wasm32.whl
```

The script pins the full toolchain (pyodide xbuildenv 0.29.4, emscripten 4.0.9,
rust nightly-2025-02-01 + pyodide's wasm-eh sysroot) and documents why each pin
exists. Load it in a page with:

```js
const { loadPyodide } = await import("https://cdn.jsdelivr.net/pyodide/v0.29.4/full/pyodide.mjs");
const py = await loadPyodide({ indexURL: "https://cdn.jsdelivr.net/pyodide/v0.29.4/full/" });
await py.loadPackage("<url of the .whl>");
py.runPython(`import ironstream as ist; print(ist.make_sphere(10).volume())`);
```
