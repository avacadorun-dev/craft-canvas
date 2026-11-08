//! World geometry stays f64; only local visible coordinates cross into a GUI's f32.
use crate::{CanvasError, CanvasLimits, Result};

/// A finite point in Canvas world coordinates or local viewport points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorldPoint {
    /// Horizontal coordinate.
    pub x: f64,
    /// Vertical coordinate.
    pub y: f64,
}
impl WorldPoint {
    /// Construct a finite coordinate pair.
    pub fn new(x: f64, y: f64) -> Result<Self> {
        if !x.is_finite() || !y.is_finite() {
            return Err(CanvasError::Invalid("nonfinite point".into()));
        }
        Ok(Self { x, y })
    }
}
/// A positive rectangle in world coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldRect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Positive width.
    pub width: f64,
    /// Positive height.
    pub height: f64,
}
impl WorldRect {
    /// Validate a node rectangle using the caller's source-compatible limits.
    pub fn validate(self, limits: &CanvasLimits) -> Result<Self> {
        let values = [self.x, self.y, self.width, self.height];
        if values.iter().any(|v| !v.is_finite())
            || (limits.integral_geometry && values.iter().any(|v| v.fract() != 0.0))
            || self.x.abs() > limits.max_coordinate
            || self.y.abs() > limits.max_coordinate
            || !(limits.min_dimension..=limits.max_dimension).contains(&self.width)
            || !(limits.min_dimension..=limits.max_dimension).contains(&self.height)
        {
            return Err(CanvasError::Invalid(
                "node geometry exceeds configured bounds".into(),
            ));
        }
        Ok(self)
    }
    /// Center point.
    pub fn center(self) -> WorldPoint {
        WorldPoint {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }
    /// Inclusive point containment.
    pub fn contains(self, p: WorldPoint) -> bool {
        p.x >= self.x && p.x <= self.x + self.width && p.y >= self.y && p.y <= self.y + self.height
    }
    /// Inclusive enclosure, used by spatial groups.
    pub fn encloses(self, other: Self) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.x + other.width <= self.x + self.width
            && other.y + other.height <= self.y + self.height
    }
    /// Whether two rectangles have a nonempty intersection.
    pub fn intersects(self, other: Self) -> bool {
        self.x <= other.x + other.width
            && self.x + self.width >= other.x
            && self.y <= other.y + other.height
            && self.y + self.height >= other.y
    }
    /// Bounding rectangle for both inputs.
    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self {
            x,
            y,
            width: (self.x + self.width).max(other.x + other.width) - x,
            height: (self.y + self.height).max(other.y + other.height) - y,
        }
    }
    /// Translate without rounding or mutating a canonical record.
    pub fn translated(self, delta: WorldPoint) -> Self {
        Self {
            x: self.x + delta.x,
            y: self.y + delta.y,
            ..self
        }
    }
}
/// A precision-safe camera expressed by its f64 world center and finite zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    center: WorldPoint,
    zoom: f64,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            center: WorldPoint::default(),
            zoom: 1.0,
        }
    }
}
impl Camera {
    /// Minimum source-compatible magnification.
    pub const MIN_ZOOM: f64 = 0.12;
    /// Maximum source-compatible magnification.
    pub const MAX_ZOOM: f64 = 3.0;
    /// Construct a finite camera. No absolute world coordinate is converted to f32.
    pub fn new(center: WorldPoint, zoom: f64) -> Result<Self> {
        WorldPoint::new(center.x, center.y)?;
        if !zoom.is_finite() || !(Self::MIN_ZOOM..=Self::MAX_ZOOM).contains(&zoom) {
            return Err(CanvasError::Invalid("camera zoom is out of range".into()));
        }
        Ok(Self { center, zoom })
    }
    /// Current world center.
    pub fn center(self) -> WorldPoint {
        self.center
    }
    /// Current magnification.
    pub fn zoom(self) -> f64 {
        self.zoom
    }
    /// World point to local viewport coordinates, with origin subtraction before scaling.
    pub fn world_to_view(self, p: WorldPoint, size: WorldPoint) -> WorldPoint {
        WorldPoint {
            x: (p.x - self.center.x) * self.zoom + size.x / 2.0,
            y: (p.y - self.center.y) * self.zoom + size.y / 2.0,
        }
    }
    /// Local viewport point to f64 world coordinates.
    pub fn view_to_world(self, p: WorldPoint, size: WorldPoint) -> WorldPoint {
        WorldPoint {
            x: self.center.x + (p.x - size.x / 2.0) / self.zoom,
            y: self.center.y + (p.y - size.y / 2.0) / self.zoom,
        }
    }
    /// Pan by local screen displacement; invalid input leaves the camera unchanged.
    pub fn pan(&mut self, delta: WorldPoint) -> Result<()> {
        let center = WorldPoint::new(
            self.center.x - delta.x / self.zoom,
            self.center.y - delta.y / self.zoom,
        )?;
        self.center = center;
        Ok(())
    }
    /// Change zoom around a pointer while preserving its world target.
    pub fn zoom_at(&mut self, factor: f64, pointer: WorldPoint, size: WorldPoint) -> Result<()> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(CanvasError::Invalid("invalid zoom factor".into()));
        }
        let anchor = self.view_to_world(pointer, size);
        let zoom = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        let center = WorldPoint::new(
            anchor.x - (pointer.x - size.x / 2.0) / zoom,
            anchor.y - (pointer.y - size.y / 2.0) / zoom,
        )?;
        self.center = center;
        self.zoom = zoom;
        Ok(())
    }
    /// Fit a nonempty world rectangle into a finite viewport with a screen-space margin.
    pub fn fit(&mut self, bounds: WorldRect, size: WorldPoint, margin: f64) -> Result<()> {
        if ![
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            size.x,
            size.y,
            margin,
        ]
        .iter()
        .all(|v| v.is_finite())
            || bounds.width <= 0.0
            || bounds.height <= 0.0
            || size.x <= margin * 2.0
            || size.y <= margin * 2.0
            || margin < 0.0
        {
            return Err(CanvasError::Invalid("invalid fit geometry".into()));
        }
        self.center = bounds.center();
        self.zoom = ((size.x - margin * 2.0) / bounds.width)
            .min((size.y - margin * 2.0) / bounds.height)
            .clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        Ok(())
    }
}
