use serde::Serialize;
use solidcraft_geom::{Aabb3, Mesh, Vec3};

use crate::Result;
use crate::body::Body;
use crate::topo::{TopoCounts, merged_topology};

/// Measured properties of a body.
#[derive(Clone, Debug, Serialize)]
pub struct BodyMeasure {
    pub volume: f64,
    pub area: f64,
    pub centroid: Vec3,
    pub bbox: Aabb3,
    /// Raw kernel topology counts (closed curves and surfaces may be split in pieces).
    pub faces: usize,
    pub edges: usize,
    pub vertices: usize,
    pub shells: usize,
    /// Counts after merging split pieces of the same analytic surface/curve (comparable with
    /// systems that use periodic faces and edges).
    pub merged: TopoCounts,
}

/// Per-face sums: area, 6 × signed volume and 24 × first moments (divergence theorem, origin 0).
fn face_sums(m: &Mesh, nf: usize) -> Vec<(f64, f64, Vec3)> {
    let mut out = vec![(0.0, 0.0, Vec3::ZERO); nf];
    for (t, f) in m.triangles.iter().zip(&m.tri_face) {
        let (Some([a, b, c]), Some(slot)) = (m.tri(t), out.get_mut(*f as usize)) else { continue };
        let d = a.dot(b.cross(c));
        slot.0 += (b - a).cross(c - a).len() * 0.5;
        slot.1 += d;
        slot.2 += (a + b + c) * d;
    }
    out
}

/// Smallest radius of curvature along the mesh's edge polylines (straight edges ignored).
fn min_edge_radius(m: &Mesh) -> Option<f64> {
    let mut best: Option<f64> = None;
    for e in &m.edges {
        for w in e.windows(3) {
            let (Some(&a), Some(&b), Some(&c)) = (w.first(), w.get(1), w.get(2)) else { continue };
            let (ab, bc, ca) = (b - a, c - b, a - c);
            let cross = ab.cross(bc).len();
            if !(cross > 1e-12 * ab.len() * bc.len()) {
                continue;
            }
            let r = ab.len() * bc.len() * ca.len() / (2.0 * cross);
            if r.is_finite() && r > 0.0 && best.is_none_or(|x| r < x) {
                best = Some(r);
            }
        }
    }
    best
}

/// Measure a body from a fine tessellation (chord tolerance 5e-5 of the body size, finer for
/// small radii down to 5e-6). Faces whose fine tessellation disagrees with a coarse one (a
/// meshing failure) use the coarse result.
pub fn measure(b: &Body) -> Result<BodyMeasure> {
    if let Some((pos, tris)) = b.triangle_mesh() {
        // Exact for a mesh body; topology counts are facets, sides and vertices.
        let m = b.tessellate(1.0)?;
        let mm = m.measure();
        let edges = b.mesh.as_ref().map(|m| m.edge_count()).unwrap_or(0);
        let merged =
            TopoCounts { faces: tris.len(), edges, vertices: pos.len(), face_types: [("mesh".to_string(), tris.len())].into_iter().collect() };
        crate::diag_trace::emit_fields(
            "measure_total",
            "mesh",
            &[("area", mm.area), ("volume", mm.volume), ("cx", mm.centroid.x), ("cy", mm.centroid.y), ("cz", mm.centroid.z)],
            (tris.len(), edges, pos.len()),
        );
        return Ok(BodyMeasure {
            volume: mm.volume,
            area: mm.area,
            centroid: mm.centroid,
            bbox: m.bounds(),
            faces: tris.len(),
            edges,
            vertices: pos.len(),
            shells: 1,
            merged,
        });
    }
    let size = b.size();
    crate::diag_trace::emit_fields("measure_begin", "", &[("size", size)], (b.face_count(), b.edge_count(), b.vertex_count()));
    let coarse = b.tessellate_with((size * 1e-3).max(1e-3), true)?;
    crate::diag_trace::emit_fields(
        "measure_mesh",
        "coarse",
        &[("tris", coarse.triangles.len() as f64), ("verts", coarse.positions.len() as f64)],
        (0, 0, 0),
    );
    // Chord error relative to a radius sets the volume error: keep it near 3e-4 of the
    // smallest radius (thin tubes, small fillets).
    let tight = min_edge_radius(&coarse).map(|r| r * 3e-4).unwrap_or(f64::INFINITY);
    let fine_tol = (size * 5e-5).min(tight).max(size * 5e-6).max(1e-4);
    let fine = b.tessellate_with(fine_tol, true)?;
    let medium = b.tessellate_with((fine_tol * 5.0).max(2e-4), true)?;
    crate::diag_trace::emit_fields(
        "measure_tol",
        "",
        &[("size", size), ("tight", tight), ("fine_tol", fine_tol), ("medium_tol", (fine_tol * 5.0).max(2e-4))],
        (0, 0, 0),
    );
    crate::diag_trace::emit_fields(
        "measure_mesh",
        "fine",
        &[("tris", fine.triangles.len() as f64), ("verts", fine.positions.len() as f64)],
        (0, 0, 0),
    );
    crate::diag_trace::emit_fields(
        "measure_mesh",
        "medium",
        &[("tris", medium.triangles.len() as f64), ("verts", medium.positions.len() as f64)],
        (0, 0, 0),
    );
    let nf = b.face_count();
    let (sf, sm, sc) = (face_sums(&fine, nf), face_sums(&medium, nf), face_sums(&coarse, nf));
    let close = |x: f64, y: f64| (x - y).abs() <= 0.01 * x.abs().max(y.abs()) + 1e-9;
    let (mut area, mut v6, mut mom) = (0.0, 0.0, Vec3::ZERO);
    for (i, ((f, m), c)) in sf.iter().zip(&sm).zip(&sc).enumerate() {
        // Trust the finest tessellation that agrees with the next coarser one.
        let (cfm, cmc) = (close(f.0, m.0), close(m.0, c.0));
        let (pick, which) = if cfm {
            (f, "fine")
        } else if cmc {
            (m, "medium")
        } else {
            (f, "coarse")
        };
        crate::diag_trace::emit_fields(
            "measure_face",
            which,
            &[
                ("idx", i as f64),
                ("area_f", f.0),
                ("area_m", m.0),
                ("area_c", c.0),
                ("v6_f", f.1),
                ("v6_m", m.1),
                ("v6_c", c.1),
                ("close_fm", f.0 - m.0),
                ("close_mc", m.0 - c.0),
            ],
            (0, 0, 0),
        );
        area += pick.0;
        v6 += pick.1;
        mom += pick.2;
    }
    let volume = v6 / 6.0;
    let centroid = if v6.abs() > 1e-12 { mom / (4.0 * v6) } else { fine.bounds().center() };
    crate::diag_trace::emit_fields(
        "measure_total",
        "",
        &[("area", area), ("v6", v6), ("volume", volume), ("cx", centroid.x), ("cy", centroid.y), ("cz", centroid.z)],
        (b.face_count(), b.edge_count(), b.vertex_count()),
    );
    Ok(BodyMeasure {
        volume,
        area,
        centroid,
        bbox: coarse.bounds().union(&fine.bounds()),
        faces: b.face_count(),
        edges: b.edge_count(),
        vertices: b.vertex_count(),
        shells: b.shell_count(),
        merged: merged_topology(b, &b.tessellate((size * 1e-3).max(1e-3))?)?,
    })
}
