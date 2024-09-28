#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dist(self, other: Vec2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn into_tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

#[derive(Debug)]
pub struct Spline {
    pub(crate) points: Vec<Vec2>,
    lengths: Vec<f32>,
    total_length: f32,
    prev: Option<Vec2>,
}

impl Spline {
    pub fn new(points: Vec<Vec2>) -> Self {
        let mut lengths = vec![];
        let mut total_length = 0.0;
        for (i, point) in points.iter().enumerate() {
            if i > 0 {
                let length = point.dist(points[i - 1]);
                lengths.push(length);
                total_length += length;
            }
        }
        Self {
            points,
            lengths,
            total_length,
            prev: None,
        }
    }

    pub fn add_point(&mut self, point: Vec2) {
        if let Some(prev) = self.prev {
            let length = point.dist(prev);
            self.lengths.push(length);
            self.total_length += length;
            self.points.push(point);
        }
        self.prev = Some(point);
    }

    pub fn clear(&mut self) {
        self.points = self.prev.take().map(|p| vec![p]).unwrap_or_default();
        self.total_length = 0.0;
    }

    pub fn get_spline_point(&self, rt: f32) -> Vec2 {
        let l = self.points.len() - 1;
        let d = rt.trunc() as usize;
        let p1 = usize::min(d + 1, l);
        let p2 = usize::min(p1 + 1, l);
        let p3 = usize::min(p2 + 1, l);
        let p0 = p1 - 1;
        let t = rt - d as f32;
        let tt = t * t;
        let ttt = tt * t;
        let q1 = -ttt + 2.0 * tt - t;
        let q2 = 3.0 * ttt - 5.0 * tt + 2.0;
        let q3 = -3.0 * ttt + 4.0 * tt + t;
        let q4 = ttt - tt;

        Vec2 {
            x: (self.points[p0].x * q1
                + self.points[p1].x * q2
                + self.points[p2].x * q3
                + self.points[p3].x * q4)
                / 2.0,
            y: (self.points[p0].y * q1
                + self.points[p1].y * q2
                + self.points[p2].y * q3
                + self.points[p3].y * q4)
                / 2.0,
        }
    }
}
