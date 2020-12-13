use glam::{vec2, Vec2};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec2,
    pub curve: QuadCurve,
    pub thickness: f32,
}

fn clamp(a: f32) -> f32 {
    if a < 0. {
        0.
    } else if a > 1. {
        1.
    } else {
        a
    }
}

pub fn bounding_box_frame(mi: Vec2, ma: Vec2, width: f32) -> (Vec2, Vec2) {
    let frame = vec2(width, width);
    (mi - frame, ma + frame)
}

/// oriented are betwee two vectors (by definition det(|v1^T \n v2^T|))
pub fn wedge(v1: Vec2, v2: Vec2) -> f32 {
    v1.x * v2.y - v1.y * v2.x
}

#[derive(Default, Debug)]
pub struct BezierPath {
    pub last: Option<Vec2>,
    pub control: Option<Vec2>,
    pub curves: Vec<QuadCurve>,
}

impl BezierPath {
    pub fn clear(&mut self) {
        self.last = None;
        self.control = None;
        self.curves = vec![];
    }

    pub fn stroke(&mut self, point: Vec2) {
        if let (Some(last), Some(control)) = (self.last, self.control) {
            self.curves.push(QuadCurve {
                a: last,
                control,
                c: point,
            });
            self.last = Some(point);
            self.control = None;
        } else if self.last.is_none() {
            self.last = Some(point);
        } else if self.control.is_none() {
            self.control = Some(point);
        };
    }

    pub fn undo(&mut self) {
        if let Some(curve) = self.curves.pop() {
            self.last = Some(curve.a);
            self.control = Some(curve.control);
        }
    }

    pub fn vertices(&self, width: f32) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = vec![];
        let mut indices = vec![];
        for curve in self.curves.iter() {
            let (vrts, ids) = curve.vertices(width);
            let indices_size = vertices.len() as u16;
            vertices.extend(vrts);
            for i in ids.iter() {
                indices.push(indices_size + i);
            }
        }
        (vertices, indices)
    }
}

pub fn rot(point: Vec2, cosb: f32, sinb: f32) -> Vec2 {
    vec2(
        cosb * point.x - sinb * point.y,
        sinb * point.x + cosb * point.y,
    )
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct QuadCurve {
    pub a: Vec2,
    pub control: Vec2,
    pub c: Vec2,
}

impl QuadCurve {
    pub fn new(a: Vec2, control: Vec2, c: Vec2) -> QuadCurve {
        QuadCurve { a, control, c }
    }

    pub fn vertices(&self, width: f32) -> (Vec<Vertex>, Vec<u16>) {
        let (a, b, c, d) = self.optimal_bb(width);
        let indices = vec![0, 1, 2, 0, 2, 3];
        (
            vec![
                Vertex {
                    position: a,
                    curve: self.clone(),
                    thickness: width,
                },
                Vertex {
                    position: b,
                    curve: self.clone(),
                    thickness: width,
                },
                Vertex {
                    position: c,
                    curve: self.clone(),
                    thickness: width,
                },
                Vertex {
                    position: d,
                    curve: self.clone(),
                    thickness: width,
                },
            ],
            indices,
        )
    }
    /// basically https://www.iquilezles.org/www/articles/bezierbbox/bezierbbox.htm
    /// with extra rotation
    fn optimal_bb(&self, width: f32) -> (Vec2, Vec2, Vec2, Vec2) {
        let dir = self.c - self.a;
        let ndir = dir.normalize();
        let ox = vec2(1., 0.);
        let sinb = wedge(ndir, ox);
        let cosb = (self.c - self.a).normalize().dot(ox);
        // align with Ox axis
        let p0 = self.a;
        let p0p1 = self.control - self.a;
        let p1 = p0 + rot(p0p1, cosb, sinb);
        let p2 = p0 + vec2(dir.length(), 0.);

        // calculation inflection point in this coordinates
        let mut mi = p0.min(p2);
        let mut ma = p0.max(p2);
        if p1.x < mi.x || p1.x > ma.x || p1.y < mi.y || p1.y > ma.y {
            let t = (p0 - p1) / (p0 - 2. * p1 + p2);
            let t = vec2(clamp(t.x), clamp(t.y));
            let s = vec2(1., 1.) - t;
            let q = s * s * p0 + 2.0 * s * t * p1 + t * t * p2;
            mi = mi.min(q);
            ma = ma.max(q);
        }
        let (mi, ma) = bounding_box_frame(mi, ma, width);
        // now align back with bezier
        let ma_rotated = p0 + rot(ma - p0, cosb, -sinb);
        let mi_rotated = p0 + rot(mi - p0, cosb, -sinb);
        let offset = ndir.dot(mi_rotated - ma_rotated) * ndir;
        let b = ma_rotated + offset;
        let d = mi_rotated - offset;
        (mi_rotated, b, ma_rotated, d)
    }

    /// bounding box with edges parallel to Ox Oy
    pub fn bounding_box(&self) -> (Vec2, Vec2) {
        let p0 = self.a;
        let p1 = self.control;
        let p2 = self.c;
        let mut mi = p0.min(p2);
        let mut ma = p0.max(p2);
        if p1.x < mi.x || p1.x > ma.x || p1.y < mi.y || p1.y > ma.y {
            let t = (p0 - p1) / (p0 - 2. * p1 + p2);
            let t = vec2(clamp(t.x), clamp(t.y));
            let s = vec2(1., 1.) - t;
            let q = s * s * p0 + 2.0 * s * t * p1 + t * t * p2;
            mi = mi.min(q);
            ma = ma.max(q);
        }
        (mi, ma)
    }

    pub fn scale(&self, scale: f32) -> QuadCurve {
        QuadCurve {
            a: self.a * scale,
            control: self.control * scale,
            c: self.c * scale,
        }
    }

    pub fn split(&self) -> (QuadCurve, QuadCurve) {
        let q0 = (self.a + self.control) / 2.;
        let q1 = (self.control + self.c) / 2.;
        let r0 = (q0 + q1) / 2.;
        (
            QuadCurve::new(self.a, q0, r0),
            QuadCurve::new(r0, q1, self.c),
        )
    }
}
