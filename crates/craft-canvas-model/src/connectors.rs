//! Shared f64 connector geometry and exact endpoint/label command data.
use crate::{CanvasError, Result, Side, WorkControl, WorldPoint, WorldRect};
/// An exact stable card endpoint; omitted side remains distinct from a new explicit side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectorEndpoint {
    /// Stable board-local card ID.
    pub node_id: String,
    /// Preserved explicit side, if supplied by the canonical edge.
    pub side: Option<Side>,
}
/// Which end of a stable edge a reconnection changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectorEndpointKind {
    /// Source endpoint.
    From,
    /// Destination endpoint.
    To,
}
/// Standard JSON Canvas endpoint marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectorEnd {
    /// No marker.
    None,
    /// Arrow pointing towards the endpoint.
    Arrow,
}
/// A stable node or connector label target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelTarget {
    /// Standard group label; application element labels remain a host dialect concern.
    Node(String),
    /// Connector label.
    Edge(String),
}
/// Native f64 cubic connector, independent of any GUI or SVG string.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicConnector {
    /// Source side midpoint.
    pub from: WorldPoint,
    /// Source tangent control point.
    pub control_from: WorldPoint,
    /// Target tangent control point.
    pub control_to: WorldPoint,
    /// Destination side midpoint.
    pub to: WorldPoint,
}
impl CubicConnector {
    /// Construct source-compatible Bézier controls from exact side midpoints.
    /// Control-distance behavior follows the MIT XYFlow geometry used by the source app;
    /// see docs/upstream.md for the inspected provenance, not a new graph dependency.
    pub fn between(
        from: WorldPoint,
        from_side: Side,
        to: WorldPoint,
        to_side: Side,
    ) -> Result<Self> {
        WorldPoint::new(from.x, from.y)?;
        WorldPoint::new(to.x, to.y)?;
        fn offset(distance: f64) -> f64 {
            if distance >= 0.0 {
                distance * 0.5
            } else {
                6.25 * (-distance).sqrt()
            }
        }
        fn control(a: WorldPoint, b: WorldPoint, side: Side) -> WorldPoint {
            match side {
                Side::Left => WorldPoint {
                    x: a.x - offset(a.x - b.x),
                    y: a.y,
                },
                Side::Right => WorldPoint {
                    x: a.x + offset(b.x - a.x),
                    y: a.y,
                },
                Side::Top => WorldPoint {
                    x: a.x,
                    y: a.y - offset(a.y - b.y),
                },
                Side::Bottom => WorldPoint {
                    x: a.x,
                    y: a.y + offset(b.y - a.y),
                },
            }
        }
        let control_from = control(from, to, from_side);
        let control_to = control(to, from, to_side);
        WorldPoint::new(control_from.x, control_from.y)?;
        WorldPoint::new(control_to.x, control_to.y)?;
        Ok(Self {
            from,
            control_from,
            control_to,
            to,
        })
    }
    /// Position on the curve. Inputs outside0..1 are refused rather than extrapolated.
    pub fn at(self, t: f64) -> Result<WorldPoint> {
        if !t.is_finite() || !(0.0..=1.0).contains(&t) {
            return Err(CanvasError::Invalid(
                "connector parameter out of range".into(),
            ));
        }
        let u = 1.0 - t;
        WorldPoint::new(
            u * u * u * self.from.x
                + 3.0 * u * u * t * self.control_from.x
                + 3.0 * u * t * t * self.control_to.x
                + t * t * t * self.to.x,
            u * u * u * self.from.y
                + 3.0 * u * u * t * self.control_from.y
                + 3.0 * u * t * t * self.control_to.y
                + t * t * t * self.to.y,
        )
    }
    /// Conservative bounds of all control points, safe for culling/hit preflight.
    pub fn bounds(self) -> WorldRect {
        let p = [self.from, self.control_from, self.control_to, self.to];
        let x = p.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let y = p.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        WorldRect {
            x,
            y,
            width: p.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) - x,
            height: p.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max) - y,
        }
    }
    /// Flatten adaptively with charged work and an explicit complete output cap.
    /// Exhaustion refuses the derived geometry; no canonical data is rewritten.
    pub fn flatten(
        self,
        tolerance: f64,
        max_segments: usize,
        work: &mut WorkControl<'_>,
    ) -> Result<Vec<WorldPoint>> {
        if !tolerance.is_finite() || tolerance <= 0.0 || max_segments == 0 || max_segments > 4096 {
            return Err(CanvasError::Invalid(
                "invalid connector flatten bounds".into(),
            ));
        }
        let mut output = vec![self.from];
        let mut stack = vec![(self, 0u8)];
        while let Some((curve, depth)) = stack.pop() {
            work.charge(1)?;
            let flat = distance_to_segment(curve.control_from, curve.from, curve.to)
                .max(distance_to_segment(curve.control_to, curve.from, curve.to))
                <= tolerance;
            if flat {
                if output.len() > max_segments {
                    return Err(CanvasError::Limit("connector segments"));
                }
                output.push(curve.to);
            } else {
                if depth >= 24 || output.len() + stack.len() + 2 > max_segments {
                    return Err(CanvasError::Limit("connector subdivision"));
                }
                let (a, b) = curve.split();
                stack.push((b, depth + 1));
                stack.push((a, depth + 1));
            }
        }
        Ok(output)
    }
    fn split(self) -> (Self, Self) {
        fn mid(a: WorldPoint, b: WorldPoint) -> WorldPoint {
            WorldPoint {
                x: a.x + (b.x - a.x) * 0.5,
                y: a.y + (b.y - a.y) * 0.5,
            }
        }
        let a = mid(self.from, self.control_from);
        let b = mid(self.control_from, self.control_to);
        let c = mid(self.control_to, self.to);
        let d = mid(a, b);
        let e = mid(b, c);
        let m = mid(d, e);
        (
            Self {
                from: self.from,
                control_from: a,
                control_to: d,
                to: m,
            },
            Self {
                from: m,
                control_from: e,
                control_to: c,
                to: self.to,
            },
        )
    }
}
/// Stable nearest distance used by source-compatible connector/stroke hit testing.
pub fn distance_to_segment(point: WorldPoint, from: WorldPoint, to: WorldPoint) -> f64 {
    let x = to.x - from.x;
    let y = to.y - from.y;
    let length = x * x + y * y;
    if length == 0.0 {
        return (point.x - from.x).hypot(point.y - from.y);
    }
    let t = (((point.x - from.x) * x + (point.y - from.y) * y) / length).clamp(0.0, 1.0);
    (point.x - from.x - t * x).hypot(point.y - from.y - t * y)
}
