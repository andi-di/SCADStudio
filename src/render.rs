//! M0 mesh inspection and orthographic drawing. GPU presentation belongs to eframe.
use anyhow::{Result, bail};
use eframe::egui::{self, Color32, Pos2, Rect, Shape};

#[derive(Clone, Default)]
/// Validated triangle positions independent of engine-internal mesh indices.
pub struct Mesh {
    pub triangles: Vec<[[f32; 3]; 3]>,
}

impl Mesh {
    /// Exact exported-edge topology, independent of triangle ordering. Returns
    /// connected components and whether every undirected edge belongs to two faces.
    pub fn topology(&self) -> (usize, bool) {
        use std::collections::{BTreeMap, BTreeSet};
        type Vertex = [u32; 3];
        let key =
            |point: [f32; 3]| -> Vertex { point.map(|v| if v == 0.0 { 0 } else { v.to_bits() }) };
        let mut edges: BTreeMap<(Vertex, Vertex), Vec<usize>> = BTreeMap::new();
        for (id, triangle) in self.triangles.iter().enumerate() {
            let t = triangle.map(key);
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                edges
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_default()
                    .push(id);
            }
        }
        let closed = edges.values().all(|faces| faces.len() == 2);
        let mut adjacency = vec![Vec::new(); self.triangles.len()];
        for faces in edges.values() {
            if let Some(first) = faces.first() {
                for other in &faces[1..] {
                    adjacency[*first].push(*other);
                    adjacency[*other].push(*first);
                }
            }
        }
        let mut visited = BTreeSet::new();
        let mut components = 0;
        for id in 0..self.triangles.len() {
            if visited.contains(&id) {
                continue;
            }
            components += 1;
            let mut pending = vec![id];
            while let Some(face) = pending.pop() {
                if visited.insert(face) {
                    pending.extend(adjacency[face].iter().copied());
                }
            }
        }
        (components, closed)
    }

    /// Loads only the bounded binary STL format requested explicitly from the worker.
    pub fn from_stl(data: &[u8]) -> Result<Self> {
        if data.len() < 84 {
            bail!("truncated STL header");
        }
        let count = u32::from_le_bytes(data[80..84].try_into()?) as usize;
        if count > 1_000_000 || data.len() != 84 + 50 * count {
            bail!("invalid STL length or triangle limit");
        }
        let mut triangles = Vec::with_capacity(count);
        for chunk in data[84..].as_chunks::<50>().0 {
            let mut triangle = [[0.0; 3]; 3];
            for (vertex, point) in triangle.iter_mut().enumerate() {
                for (axis, value) in point.iter_mut().enumerate() {
                    let offset = 12 + vertex * 12 + axis * 4;
                    *value = f32::from_le_bytes(chunk[offset..offset + 4].try_into()?);
                    if !value.is_finite() {
                        bail!("non-finite STL coordinate");
                    }
                }
            }
            triangles.push(triangle);
        }
        Ok(Self { triangles })
    }

    /// Tessellation-dependent axis-aligned mesh bounds, or no bounds for an empty mesh.
    pub fn bounds(&self) -> Option<[[f32; 3]; 2]> {
        let mut bounds = [[f32::INFINITY; 3], [f32::NEG_INFINITY; 3]];
        for point in self.triangles.iter().flatten() {
            for axis in 0..3 {
                bounds[0][axis] = bounds[0][axis].min(point[axis]);
                bounds[1][axis] = bounds[1][axis].max(point[axis]);
            }
        }
        (!self.triangles.is_empty()).then_some(bounds)
    }

    /// Absolute signed-tetrahedron volume; meaningful for consistently oriented closed meshes.
    pub fn volume(&self) -> f64 {
        self.triangles
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|p| p.map(f64::from));
                (a[0] * (b[1] * c[2] - b[2] * c[1])
                    + a[1] * (b[2] * c[0] - b[0] * c[2])
                    + a[2] * (b[0] * c[1] - b[1] * c[0]))
                    / 6.0
            })
            .sum::<f64>()
            .abs()
    }
}

/// Shared projection for native view and offscreen wgpu rendering. M0 uses painter
/// ordering, not a production depth buffer; intersecting surfaces can be ambiguous.
pub fn shapes(
    mesh: &Mesh,
    bounds: [[f32; 3]; 2],
    rect: Rect,
    yaw: f32,
    selected: bool,
) -> Vec<Shape> {
    let center = std::array::from_fn::<_, 3, _>(|axis| (bounds[0][axis] + bounds[1][axis]) * 0.5);
    let size = (0..3)
        .map(|axis| bounds[1][axis] - bounds[0][axis])
        .fold(1.0_f32, f32::max);
    let scale = rect.width().min(rect.height()) * 0.55 / size;
    let project = |point: [f32; 3]| {
        let [x, y, z] = std::array::from_fn(|axis| point[axis] - center[axis]);
        let u = x * yaw.cos() - y * yaw.sin();
        let depth = x * yaw.sin() + y * yaw.cos();
        (
            Pos2::new(
                rect.center().x + u * scale,
                rect.center().y + (depth * 0.45 - z) * scale,
            ),
            depth + z * 0.45,
        )
    };
    let mut triangles: Vec<_> = mesh
        .triangles
        .iter()
        .map(|t| {
            let a: [f32; 3] = std::array::from_fn(|axis| t[1][axis] - t[0][axis]);
            let b: [f32; 3] = std::array::from_fn(|axis| t[2][axis] - t[0][axis]);
            let normal = [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ];
            let length = normal
                .iter()
                .map(|value| value * value)
                .sum::<f32>()
                .sqrt()
                .max(1e-10);
            let shade = 0.45
                + 0.55
                    * ((normal[0] * 0.2 + normal[1] * 0.4 + normal[2] * 0.8) / length)
                        .abs()
                        .min(1.0);
            (t.map(project), shade)
        })
        .collect();
    triangles.sort_by(|a, b| {
        let depth = |t: &[(Pos2, f32); 3]| t.iter().map(|p| p.1).sum::<f32>();
        depth(&a.0).total_cmp(&depth(&b.0))
    });
    triangles
        .into_iter()
        .map(|(t, shade)| {
            let base = if selected {
                [245.0, 180.0, 55.0]
            } else {
                [75.0, 160.0, 190.0]
            };
            let color = Color32::from_rgb(
                (base[0] * shade) as u8,
                (base[1] * shade) as u8,
                (base[2] * shade) as u8,
            );
            let mut triangle = egui::Mesh::default();
            for point in t {
                triangle.colored_vertex(point.0, color);
            }
            triangle.add_triangle(0, 1, 2);
            Shape::mesh(triangle)
        })
        .collect()
}

/// Draws the cached mesh and optional origin highlight, returning a click for candidate picking.
pub fn draw(
    ui: &mut egui::Ui,
    mesh: &Mesh,
    selected: Option<&Mesh>,
    yaw: &mut f32,
) -> (Rect, Option<Pos2>) {
    let (rect, response) =
        ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
    if response.dragged() {
        *yaw += response.drag_delta().x * 0.01;
    }
    ui.painter()
        .rect_filled(rect, 4.0, Color32::from_rgb(24, 29, 38));
    if let Some(bounds) = mesh.bounds() {
        ui.painter().extend(shapes(mesh, bounds, rect, *yaw, false));
        if let Some(selected) = selected {
            ui.painter()
                .extend(shapes(selected, bounds, rect, *yaw, true));
        }
    }
    (
        rect,
        response
            .clicked()
            .then(|| response.interact_pointer_pos())
            .flatten(),
    )
}

/// Screen-projected candidate hit test. Returns candidates, never pretends that
/// overlapping CSG surfaces have one provably unique source.
pub fn hit_test(mesh: &Mesh, bounds: [[f32; 3]; 2], rect: Rect, yaw: f32, point: Pos2) -> bool {
    shapes(mesh, bounds, rect, yaw, false)
        .into_iter()
        .any(|shape| {
            let Shape::Mesh(mesh) = shape else {
                return false;
            };
            let [a, b, c] = [
                mesh.vertices[0].pos,
                mesh.vertices[1].pos,
                mesh.vertices[2].pos,
            ];
            if ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)).abs() < 0.001 {
                return false;
            }
            let mut positive = false;
            let mut negative = false;
            for index in 0..3 {
                let a = mesh.vertices[index].pos;
                let b = mesh.vertices[(index + 1) % 3].pos;
                let cross = (b.x - a.x) * (point.y - a.y) - (b.y - a.y) * (point.x - a.x);
                positive |= cross > 0.001;
                negative |= cross < -0.001;
            }
            !(positive && negative)
        })
}
