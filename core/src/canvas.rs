use std::sync::Arc;

use crate::{Color, Point};

/// A two dimensional canvas to draw on, see [`paint`](crate::views::paint).
#[derive(Clone, Debug, PartialEq)]
pub struct Canvas {
    primitives: Arc<Vec<Primitive>>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    /// Create new [`Canvas`].
    pub fn new() -> Self {
        Self {
            primitives: Default::default(),
        }
    }

    /// Stroke `curve` with given `paint`.
    pub fn stroke(&mut self, curve: Curve, paint: Paint) {
        let primitives = Arc::make_mut(&mut self.primitives);
        primitives.push(Primitive::Stroke(curve, paint));
    }

    /// Fill `curve` with given `paint`.
    pub fn fill(&mut self, curve: Curve, paint: Paint) {
        let primitives = Arc::make_mut(&mut self.primitives);
        primitives.push(Primitive::Fill(curve, paint));
    }

    /// Get the [`Primitive`]s in `self`.
    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }
}

/// A primitive unit in a [`Canvas`].
#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    /// A stroked [`Curve`].
    Stroke(Curve, Paint),

    /// A filled [`Curve`].
    Fill(Curve, Paint),
}

/// Strategy for determining whether a point is inside of a [`Curve`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fill {
    /// Inside the [`Curve`] if the signed winding count of a ray originating at the point is non-zero.
    Winding,

    /// Inside the [`Curve`] if a ray originating at the point crosses the [`Curve`] an odd number
    /// of times.
    EvenOdd,
}

/// Strategy for shading an region on a [`Canvas`].
#[derive(Clone, Debug, PartialEq)]
pub enum Shader {
    /// The region is filled with a color.
    Color(Color),

    /// The region is filled with a linear gradient.
    Gradient(Gradient),
}

/// A gradient.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    /// The start point.
    pub start: Point<f32>,

    /// The end point.
    pub end: Point<f32>,

    /// The colors to interpolate between.
    pub colors: Vec<Color>,
}

/// Strategy for blending two colors.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Blend {
    Clear,
    Src,
    Dst,
    SrcOver,
    DstOver,
    SrcIn,
    DstIn,
    SrcATop,
    DstATop,
}

/// Strategy for joining segments in a [`Curve`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Join {
    /// Draw a sharp corner, limitied by [`Stroke::miter`].
    Miter,

    /// Draw a round corner.
    Round,

    /// Draw a cut-off join.
    Bevel,
}

/// Specifies how endpoints of a [`Curve`] are rendered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cap {
    /// Square ending, exactly at the first/last point.
    Butt,

    /// Round ending centered on the first/last point.
    Round,

    /// Square ending centered on the first/last point.
    Square,
}

/// Specifies how a [`Curve`] should be stroked on a [`Canvas`].
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    /// The width of the line.
    pub width: f32,

    /// The maximum length of a [`Join::Miter`] join.
    pub miter: f32,

    /// How segments should be joined.
    pub join: Join,

    /// How endings should rendered.
    pub cap: Cap,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            miter: 4.0,
            join:  Join::Miter,
            cap:   Cap::Butt,
        }
    }
}

/// Options applied when drawing on a [`Canvas`].
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    /// The [`Shader`] used when drawing.
    pub shader: Shader,

    /// The [`Blend`] mode used when drawing.
    pub blend: Blend,

    /// The [`Stroke`] options used in [`Canvas::stroke`].
    pub stroke: Stroke,

    /// The [`Fill`] mode used in [`Canvas::fill`].
    pub fill: Fill,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            shader: Shader::Color(Color::BLACK),
            blend:  Blend::SrcOver,
            stroke: Stroke::default(),
            fill:   Fill::Winding,
        }
    }
}

/// A collection of geometries, used for drawing on a [`Canvas`].
#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    points: Arc<Vec<Point<f32>>>,
    verbs:  Arc<Vec<CurveVerb>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CurveVerb {
    Move,
    Line,
    Quad,
    Cubic,
    Close,
}

/// A segment in a [`Curve`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CurveSegment {
    /// The beginning of a contour.
    Move(Point<f32>),

    /// A line from the last point.
    Line(Point<f32>),

    /// A quadratic bezier curve from the last point.
    Quad(Point<f32>, Point<f32>),

    /// A cubic bezier curve from the last point.
    Cubic(Point<f32>, Point<f32>, Point<f32>),

    /// The contour is closed.
    Close,
}

impl Default for Curve {
    fn default() -> Self {
        Self::new()
    }
}

impl Curve {
    /// Create new [`Curve`].
    pub fn new() -> Self {
        Self {
            points: Default::default(),
            verbs:  Default::default(),
        }
    }

    /// Check if `self` is empty.
    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }

    /// Check if `self` is closed.
    pub fn is_closed(&self) -> bool {
        self.verbs.last() == Some(&CurveVerb::Close)
    }

    /// Specify the beginning of a new contour.
    pub fn move_to(&mut self, x: f32, y: f32) {
        let points = Arc::make_mut(&mut self.points);
        let verbs = Arc::make_mut(&mut self.verbs);

        points.push(Point::new(x, y));
        verbs.push(CurveVerb::Move);
    }

    /// Add a line from the previous point to `(x, y)`.
    pub fn line_to(&mut self, x: f32, y: f32) {
        let points = Arc::make_mut(&mut self.points);
        let verbs = Arc::make_mut(&mut self.verbs);

        points.push(Point::new(x, y));
        verbs.push(CurveVerb::Line);
    }

    /// Add a quadratic bezier curve from the previous point to `(x1, y1)` with a control point at
    /// `(x0, y0)`.
    pub fn quad_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        let points = Arc::make_mut(&mut self.points);
        let verbs = Arc::make_mut(&mut self.verbs);

        points.push(Point::new(x0, y0));
        points.push(Point::new(x1, y1));
        verbs.push(CurveVerb::Quad);
    }

    /// Add a cubic bezier curve from the previous point to `(x2, y2)` with control points at
    /// `(x0, y0)` and `(x1, y1)`.
    pub fn cubic_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        let points = Arc::make_mut(&mut self.points);
        let verbs = Arc::make_mut(&mut self.verbs);

        points.push(Point::new(x0, y0));
        points.push(Point::new(x1, y1));
        points.push(Point::new(x2, y2));
        verbs.push(CurveVerb::Cubic);
    }

    /// Close the contour.
    pub fn close(&mut self) {
        if !self.is_closed() && !self.is_empty() {
            let verbs = Arc::make_mut(&mut self.verbs);
            verbs.push(CurveVerb::Close);
        }
    }

    /// Add a rectangle.
    pub fn rectangle(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        self.move_to(left, top);
        self.line_to(left, bottom);
        self.line_to(right, bottom);
        self.line_to(right, top);
        self.line_to(left, top);
        self.close();
    }

    /// Add a circle with center at `(x, y)` and radius `r`.
    pub fn circle(&mut self, x: f32, y: f32, r: f32) {
        let k = r * 0.552_284_7;

        self.move_to(x, y - r);
        self.cubic_to(x - k, y - r, x - r, y - k, x - r, y);
        self.cubic_to(x - r, y + k, x - k, y + r, x, y + r);
        self.cubic_to(x + k, y + r, x + r, y + k, x + r, y);
        self.cubic_to(x + r, y - k, x + k, y - r, x, y - r);
        self.close();
    }

    /// Get an [`Iterator`] over the [`CurveSegment`]s in `self`.
    pub fn segments(&self) -> impl Iterator<Item = CurveSegment> {
        const EXPECT: &str = "`Curve` should always uphold its invariants";

        let mut points = self.points.iter();

        self.verbs.iter().map(move |verb| match verb {
            CurveVerb::Move => {
                let p0 = points.next().expect(EXPECT);

                CurveSegment::Move(*p0)
            }

            CurveVerb::Line => {
                let p0 = points.next().expect(EXPECT);

                CurveSegment::Line(*p0)
            }

            CurveVerb::Quad => {
                let p0 = points.next().expect(EXPECT);
                let p1 = points.next().expect(EXPECT);

                CurveSegment::Quad(*p0, *p1)
            }

            CurveVerb::Cubic => {
                let p0 = points.next().expect(EXPECT);
                let p1 = points.next().expect(EXPECT);
                let p2 = points.next().expect(EXPECT);

                CurveSegment::Cubic(*p0, *p1, *p2)
            }

            CurveVerb::Close => CurveSegment::Close,
        })
    }
}
