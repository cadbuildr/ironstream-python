//! Python bindings for the IronStream CAD geometry kernel.
//!
//! The surface mirrors IronStream's OCCT-shaped prelude, in the spirit of
//! pyOCCT: types live both flat under `ironstream` and under OCCT-style
//! submodules (`ironstream.gp`, `ironstream.prim`, `ironstream.algo`,
//! `ironstream.builder`, `ironstream.topods`, `ironstream.mesh`,
//! `ironstream.io`).

use pyo3::prelude::*;
use pyo3::types::PyBytes;

// The kernel crate (renamed via Cargo to avoid clashing with our own lib name).
use kernel::prelude as kp;
use kp::{Ax1, BBox, Compound, Face, MeshParams, Pnt, Solid, Trsf, TriMesh, Vertex, Wire};

// ---------------------------------------------------------------------------
// gp — points, axes, transforms
// ---------------------------------------------------------------------------

#[pyclass(name = "Pnt", module = "ironstream.gp")]
#[derive(Clone)]
struct PyPnt {
    inner: Pnt,
}

#[pymethods]
impl PyPnt {
    #[new]
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { inner: Pnt::new(x, y, z) }
    }
    #[staticmethod]
    fn origin() -> Self {
        Self { inner: Pnt::origin() }
    }
    #[getter]
    fn x(&self) -> f64 {
        self.inner.x
    }
    #[getter]
    fn y(&self) -> f64 {
        self.inner.y
    }
    #[getter]
    fn z(&self) -> f64 {
        self.inner.z
    }
    fn dot(&self, o: &PyPnt) -> f64 {
        self.inner.dot(o.inner)
    }
    fn cross(&self, o: &PyPnt) -> PyPnt {
        PyPnt { inner: self.inner.cross(o.inner) }
    }
    fn norm(&self) -> f64 {
        self.inner.norm()
    }
    fn distance(&self, o: &PyPnt) -> f64 {
        self.inner.distance(o.inner)
    }
    fn as_tuple(&self) -> (f64, f64, f64) {
        (self.inner.x, self.inner.y, self.inner.z)
    }
    fn __repr__(&self) -> String {
        format!("Pnt({}, {}, {})", self.inner.x, self.inner.y, self.inner.z)
    }
}

#[pyclass(name = "Ax1", module = "ironstream.gp")]
#[derive(Clone)]
struct PyAx1 {
    inner: Ax1,
}

#[pymethods]
impl PyAx1 {
    #[new]
    fn new(location: &PyPnt, direction: &PyPnt) -> Self {
        Self { inner: Ax1::new(location.inner, direction.inner) }
    }
    #[staticmethod]
    fn oz() -> Self {
        Self { inner: Ax1::oz() }
    }
    #[getter]
    fn location(&self) -> PyPnt {
        PyPnt { inner: self.inner.location }
    }
    #[getter]
    fn direction(&self) -> PyPnt {
        PyPnt { inner: self.inner.direction }
    }
}

#[pyclass(name = "Trsf", module = "ironstream.gp")]
#[derive(Clone)]
struct PyTrsf {
    inner: Trsf,
}

#[pymethods]
impl PyTrsf {
    #[staticmethod]
    fn translation(v: &PyPnt) -> Self {
        Self { inner: Trsf::translation(v.inner) }
    }
    #[staticmethod]
    fn rotation(axis: &PyAx1, angle: f64) -> Self {
        Self { inner: Trsf::rotation(axis.inner, angle) }
    }
    #[staticmethod]
    fn scale_uniform(f: f64) -> Self {
        Self { inner: Trsf::scale_uniform(f) }
    }
    #[staticmethod]
    fn scale_xyz(fx: f64, fy: f64, fz: f64) -> Self {
        Self { inner: Trsf::scale_xyz(fx, fy, fz) }
    }
    /// Compose: apply `self`, then `outer`.
    fn then(&self, outer: &PyTrsf) -> PyTrsf {
        PyTrsf { inner: self.inner.then(&outer.inner) }
    }
    fn apply_point(&self, p: &PyPnt) -> PyPnt {
        PyPnt { inner: self.inner.apply_point(p.inner) }
    }
}

// ---------------------------------------------------------------------------
// topods — topology handles
// ---------------------------------------------------------------------------

#[pyclass(name = "Vertex", module = "ironstream.topods")]
#[derive(Clone)]
struct PyVertex {
    inner: Vertex,
}

#[pymethods]
impl PyVertex {
    #[new]
    fn new(p: &PyPnt) -> Self {
        Self { inner: Vertex(p.inner) }
    }
    #[getter]
    fn point(&self) -> PyPnt {
        PyPnt { inner: self.inner.0 }
    }
}

#[pyclass(name = "Wire", module = "ironstream.topods")]
#[derive(Clone)]
struct PyWire {
    inner: Wire,
}

#[pyclass(name = "Face", module = "ironstream.topods")]
#[derive(Clone)]
struct PyFace {
    inner: Face,
}

#[pyclass(name = "Compound", module = "ironstream.topods")]
#[derive(Clone)]
struct PyCompound {
    inner: Compound,
}

#[pyclass(name = "Solid", module = "ironstream.topods")]
#[derive(Clone)]
struct PySolid {
    inner: Solid,
}

#[pymethods]
impl PySolid {
    fn volume(&self) -> f64 {
        self.inner.volume()
    }
    fn mesh(&self) -> PyTriMesh {
        PyTriMesh { inner: self.inner.mesh().clone() }
    }
    fn __repr__(&self) -> String {
        format!("Solid(volume={:.4})", self.inner.volume())
    }
}

// ---------------------------------------------------------------------------
// mesh — triangle mesh + bounding box
// ---------------------------------------------------------------------------

#[pyclass(name = "TriMesh", module = "ironstream.mesh")]
#[derive(Clone)]
struct PyTriMesh {
    inner: TriMesh,
}

#[pymethods]
impl PyTriMesh {
    #[getter]
    fn num_vertices(&self) -> usize {
        self.inner.verts.len()
    }
    #[getter]
    fn num_triangles(&self) -> usize {
        self.inner.tris.len()
    }
    fn volume(&self) -> f64 {
        self.inner.volume()
    }
    /// Flat `[x0, y0, z0, x1, y1, z1, ...]` vertex buffer.
    fn vertices_flat(&self) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.inner.verts.len() * 3);
        for p in &self.inner.verts {
            out.push(p.x);
            out.push(p.y);
            out.push(p.z);
        }
        out
    }
    /// Flat `[i0, j0, k0, i1, j1, k1, ...]` triangle index buffer.
    fn triangles_flat(&self) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.inner.tris.len() * 3);
        for t in &self.inner.tris {
            out.extend_from_slice(&[t[0], t[1], t[2]]);
        }
        out
    }
    fn __repr__(&self) -> String {
        format!(
            "TriMesh(vertices={}, triangles={})",
            self.inner.verts.len(),
            self.inner.tris.len()
        )
    }
}

#[pyclass(name = "BBox", module = "ironstream.mesh")]
#[derive(Clone)]
struct PyBBox {
    inner: BBox,
}

#[pymethods]
impl PyBBox {
    fn volume(&self) -> f64 {
        self.inner.volume()
    }
}

// ---------------------------------------------------------------------------
// prim / builder / algo — mesh params + geometry construction
// ---------------------------------------------------------------------------

#[pyclass(name = "MeshParams", module = "ironstream.prim")]
#[derive(Clone)]
struct PyMeshParams {
    circle_segments: usize,
    lat_segments: usize,
}

#[pymethods]
impl PyMeshParams {
    #[new]
    #[pyo3(signature = (circle_segments = 64, lat_segments = 32))]
    fn new(circle_segments: usize, lat_segments: usize) -> Self {
        Self { circle_segments, lat_segments }
    }
    #[getter]
    fn circle_segments(&self) -> usize {
        self.circle_segments
    }
    #[getter]
    fn lat_segments(&self) -> usize {
        self.lat_segments
    }
}

impl PyMeshParams {
    fn to_kernel(&self) -> MeshParams {
        MeshParams { circle_segments: self.circle_segments, lat_segments: self.lat_segments }
    }
}

fn resolve_mp(mp: Option<&PyMeshParams>) -> MeshParams {
    match mp {
        Some(m) => m.to_kernel(),
        None => MeshParams::default(),
    }
}

// --- primitives ---

#[pyfunction]
fn make_box(corner: &PyPnt, dx: f64, dy: f64, dz: f64) -> PySolid {
    PySolid { inner: kp::make_box(corner.inner, dx, dy, dz) }
}

#[pyfunction]
fn make_box_centered_xy(center: &PyPnt, w: f64, h: f64, d: f64) -> PySolid {
    PySolid { inner: kp::make_box_centered_xy(center.inner, w, h, d) }
}

#[pyfunction]
#[pyo3(signature = (r, h, mp = None))]
fn make_cylinder(r: f64, h: f64, mp: Option<&PyMeshParams>) -> PySolid {
    PySolid { inner: kp::make_cylinder(r, h, resolve_mp(mp)) }
}

#[pyfunction]
#[pyo3(signature = (r1, r2, h, mp = None))]
fn make_cone(r1: f64, r2: f64, h: f64, mp: Option<&PyMeshParams>) -> PySolid {
    PySolid { inner: kp::make_cone(r1, r2, h, resolve_mp(mp)) }
}

#[pyfunction]
#[pyo3(signature = (r, mp = None))]
fn make_sphere(r: f64, mp: Option<&PyMeshParams>) -> PySolid {
    PySolid { inner: kp::make_sphere(r, resolve_mp(mp)) }
}

#[pyfunction]
#[pyo3(signature = (major, minor, mp = None))]
fn make_torus(major: f64, minor: f64, mp: Option<&PyMeshParams>) -> PySolid {
    PySolid { inner: kp::make_torus(major, minor, resolve_mp(mp)) }
}

#[pyfunction]
fn make_prism(face: &PyFace, vec: &PyPnt) -> PySolid {
    PySolid { inner: kp::make_prism(&face.inner, vec.inner) }
}

#[pyfunction]
#[pyo3(signature = (wire, axis, angle, mp = None))]
fn make_revol(wire: &PyWire, axis: &PyAx1, angle: f64, mp: Option<&PyMeshParams>) -> PySolid {
    PySolid { inner: kp::make_revol(&wire.inner, axis.inner, angle, resolve_mp(mp)) }
}

// --- builders ---

#[pyfunction]
fn make_polygon(points: Vec<PyPnt>) -> PyWire {
    let pts: Vec<Pnt> = points.iter().map(|p| p.inner).collect();
    PyWire { inner: kp::make_polygon(&pts) }
}

#[pyfunction]
fn make_face(outer: &PyWire) -> PyFace {
    PyFace { inner: kp::make_face(outer.inner.clone()) }
}

#[pyfunction]
fn make_face_with_holes(outer: &PyWire, holes: Vec<PyWire>) -> PyFace {
    let holes: Vec<Wire> = holes.iter().map(|w| w.inner.clone()).collect();
    PyFace { inner: kp::make_face_with_holes(outer.inner.clone(), holes) }
}

#[pyfunction]
fn transform(solid: &PySolid, t: &PyTrsf) -> PySolid {
    PySolid { inner: kp::transform(&solid.inner, &t.inner) }
}

/// Build a solid directly from a triangle soup — flat vertex buffer
/// `[x0,y0,z0, ...]` and flat index buffer `[i0,j0,k0, ...]`. Used by callers
/// (e.g. the castiron loft/sweep pass) that assemble a mesh themselves.
#[pyfunction]
fn solid_from_mesh(vertices_flat: Vec<f64>, triangles_flat: Vec<usize>) -> PySolid {
    let verts: Vec<Pnt> = vertices_flat
        .chunks_exact(3)
        .map(|c| Pnt::new(c[0], c[1], c[2]))
        .collect();
    let tris: Vec<[usize; 3]> = triangles_flat
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let mesh = TriMesh { verts, tris, ..Default::default() };
    PySolid { inner: Solid::from_mesh(mesh) }
}

// --- booleans ---

#[pyfunction]
fn fuse(a: &PySolid, b: &PySolid) -> PySolid {
    PySolid { inner: kp::fuse(&a.inner, &b.inner) }
}

#[pyfunction]
fn cut(a: &PySolid, b: &PySolid) -> PySolid {
    PySolid { inner: kp::cut(&a.inner, &b.inner) }
}

#[pyfunction]
fn common(a: &PySolid, b: &PySolid) -> PySolid {
    PySolid { inner: kp::common(&a.inner, &b.inner) }
}

#[pyfunction]
fn fuse_all(solids: Vec<PySolid>) -> PySolid {
    let solids: Vec<Solid> = solids.iter().map(|s| s.inner.clone()).collect();
    PySolid { inner: kp::fuse_all(&solids) }
}

// --- io ---

#[pyfunction]
fn write_binary_stl<'py>(py: Python<'py>, mesh: &PyTriMesh) -> Bound<'py, PyBytes> {
    PyBytes::new(py, &kp::write_binary_stl(&mesh.inner))
}

#[pyfunction]
fn write_ascii_stl(mesh: &PyTriMesh, name: &str) -> String {
    kp::write_ascii_stl(&mesh.inner, name)
}

#[pyfunction]
fn write_step(mesh: &PyTriMesh, name: &str) -> String {
    kp::write_step(&mesh.inner, name)
}

// ---------------------------------------------------------------------------
// module assembly
// ---------------------------------------------------------------------------

/// Register the classes used across submodules onto a target module.
fn add_common_classes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPnt>()?;
    m.add_class::<PyAx1>()?;
    m.add_class::<PyTrsf>()?;
    m.add_class::<PyVertex>()?;
    m.add_class::<PyWire>()?;
    m.add_class::<PyFace>()?;
    m.add_class::<PyCompound>()?;
    m.add_class::<PySolid>()?;
    m.add_class::<PyTriMesh>()?;
    m.add_class::<PyBBox>()?;
    m.add_class::<PyMeshParams>()?;
    Ok(())
}

fn make_submodule<'py>(
    parent: &Bound<'py, PyModule>,
    name: &str,
) -> PyResult<Bound<'py, PyModule>> {
    let py = parent.py();
    let sub = PyModule::new(py, name)?;
    parent.add_submodule(&sub)?;
    // Register in sys.modules so `import ironstream.<name>` works.
    py.import("sys")?
        .getattr("modules")?
        .set_item(format!("ironstream.{name}"), &sub)?;
    Ok(sub)
}

#[pymodule]
fn ironstream(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Flat top-level surface (mirrors the Rust prelude).
    add_common_classes(m)?;
    m.add("Vec3", m.getattr("Pnt")?)?;
    m.add("Point3", m.getattr("Pnt")?)?;

    macro_rules! add_fns {
        ($target:expr, $($f:ident),+ $(,)?) => {
            $( $target.add_function(wrap_pyfunction!($f, $target)?)?; )+
        };
    }

    add_fns!(
        m,
        make_box,
        make_box_centered_xy,
        make_cylinder,
        make_cone,
        make_sphere,
        make_torus,
        make_prism,
        make_revol,
        make_polygon,
        make_face,
        make_face_with_holes,
        transform,
        solid_from_mesh,
        fuse,
        cut,
        common,
        fuse_all,
        write_binary_stl,
        write_ascii_stl,
        write_step,
    );

    // OCCT-style submodules (pyOCCT feel): the same symbols, namespaced.
    let gp = make_submodule(m, "gp")?;
    gp.add_class::<PyPnt>()?;
    gp.add_class::<PyAx1>()?;
    gp.add_class::<PyTrsf>()?;
    gp.add("Vec3", gp.getattr("Pnt")?)?;
    gp.add("Point3", gp.getattr("Pnt")?)?;

    let topods = make_submodule(m, "topods")?;
    topods.add_class::<PyVertex>()?;
    topods.add_class::<PyWire>()?;
    topods.add_class::<PyFace>()?;
    topods.add_class::<PyCompound>()?;
    topods.add_class::<PySolid>()?;

    let mesh = make_submodule(m, "mesh")?;
    mesh.add_class::<PyTriMesh>()?;
    mesh.add_class::<PyBBox>()?;

    let prim = make_submodule(m, "prim")?;
    prim.add_class::<PyMeshParams>()?;
    add_fns!(
        &prim,
        make_box,
        make_box_centered_xy,
        make_cylinder,
        make_cone,
        make_sphere,
        make_torus,
        make_prism,
        make_revol,
    );

    let builder = make_submodule(m, "builder")?;
    add_fns!(&builder, make_polygon, make_face, make_face_with_holes, transform, solid_from_mesh);

    let algo = make_submodule(m, "algo")?;
    add_fns!(&algo, fuse, cut, common, fuse_all);

    let io = make_submodule(m, "io")?;
    add_fns!(&io, write_binary_stl, write_ascii_stl, write_step);

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
