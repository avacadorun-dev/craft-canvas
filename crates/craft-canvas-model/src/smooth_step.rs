//! Bounded f64 port of XYFlow's default smooth-step routing; see docs/upstream.md.
use crate::{CanvasError, CubicConnector, Result, Side, WorkControl, WorldPoint, WorldRect};

/// Derived orthogonal connector with rounded bends and the source-compatible label anchor.
/// This presentation never writes fallback sides or route points into canonical JSON.
#[derive(Clone, Debug, PartialEq)]
pub struct SmoothStepConnector {
    points: Vec<WorldPoint>,
    label: WorldPoint,
}

impl SmoothStepConnector {
    /// Route the exact side endpoints using the source defaults: offset20, step0.5, radius5.
    pub fn between(
        from: WorldPoint,
        from_side: Side,
        to: WorldPoint,
        to_side: Side,
    ) -> Result<Self> {
        WorldPoint::new(from.x, from.y)?;
        WorldPoint::new(to.x, to.y)?;
        let from_dir = direction(from_side);
        let to_dir = direction(to_side);
        let from_gap = translated(from, from_dir, 20.0);
        let to_gap = translated(to, to_dir, 20.0);
        let horizontal = from_dir.x != 0.0;
        let axis = |point: WorldPoint| if horizontal { point.x } else { point.y };
        let opposite = |point: WorldPoint| if horizontal { point.y } else { point.x };
        let current_direction = if axis(from_gap) < axis(to_gap) {
            1.0
        } else {
            -1.0
        };
        let mut from_adjustment = WorldPoint::default();
        let mut to_adjustment = WorldPoint::default();
        let middle;
        let label;
        if axis(from_dir) * axis(to_dir) == -1.0 {
            label = midpoint(from_gap, to_gap);
            let vertical = [
                WorldPoint {
                    x: label.x,
                    y: from_gap.y,
                },
                WorldPoint {
                    x: label.x,
                    y: to_gap.y,
                },
            ];
            let horizontal_points = [
                WorldPoint {
                    x: from_gap.x,
                    y: label.y,
                },
                WorldPoint {
                    x: to_gap.x,
                    y: label.y,
                },
            ];
            middle = if (axis(from_dir) == current_direction) == horizontal {
                vertical.to_vec()
            } else {
                horizontal_points.to_vec()
            };
        } else {
            let source_target = WorldPoint {
                x: from_gap.x,
                y: to_gap.y,
            };
            let target_source = WorldPoint {
                x: to_gap.x,
                y: from_gap.y,
            };
            let mut corner = if (axis(from_dir) == current_direction) == horizontal {
                target_source
            } else {
                source_target
            };
            if from_side == to_side {
                let difference = (axis(from) - axis(to)).abs();
                if difference <= 20.0 {
                    let adjustment = (20.0 - difference).min(19.0);
                    let (gap, endpoint, destination) = if axis(from_dir) == current_direction {
                        (from_gap, from, &mut from_adjustment)
                    } else {
                        (to_gap, to, &mut to_adjustment)
                    };
                    let value = if axis(gap) > axis(endpoint) {
                        -adjustment
                    } else {
                        adjustment
                    };
                    if horizontal {
                        destination.x = value;
                    } else {
                        destination.y = value;
                    }
                }
            } else {
                let same_direction = axis(from_dir) == opposite(to_dir);
                let source_greater = opposite(from_gap) > opposite(to_gap);
                let source_less = opposite(from_gap) < opposite(to_gap);
                let flip = if axis(from_dir) == 1.0 {
                    (!same_direction && source_greater) || (same_direction && source_less)
                } else {
                    (!same_direction && source_less) || (same_direction && source_greater)
                };
                if flip {
                    corner = if horizontal {
                        source_target
                    } else {
                        target_source
                    };
                }
            }
            let a = translated(from_gap, from_adjustment, 1.0);
            let b = translated(to_gap, to_adjustment, 1.0);
            let dx = (a.x - corner.x).abs().max((b.x - corner.x).abs());
            let dy = (a.y - corner.y).abs().max((b.y - corner.y).abs());
            let center = midpoint(a, b);
            label = if dx >= dy {
                WorldPoint {
                    x: center.x,
                    y: corner.y,
                }
            } else {
                WorldPoint {
                    x: corner.x,
                    y: center.y,
                }
            };
            middle = vec![corner];
        }
        let mut points = Vec::with_capacity(6);
        points.push(from);
        let a = translated(from_gap, from_adjustment, 1.0);
        if middle.first() != Some(&a) {
            points.push(a);
        }
        points.extend_from_slice(&middle);
        let b = translated(to_gap, to_adjustment, 1.0);
        if middle.last() != Some(&b) {
            points.push(b);
        }
        points.push(to);
        for point in points.iter().chain([&label]) {
            WorldPoint::new(point.x, point.y)?;
        }
        Ok(Self { points, label })
    }

    /// Source routing's label location on the central/longest segment.
    pub fn label_anchor(&self) -> WorldPoint {
        self.label
    }

    /// Route a same-card edge outside its body, avoiding the source's hidden opposite-side path.
    /// The shortest side route wins; opposite-side ties use clockwise order. Equal legacy
    /// handles retain a visible complete loop. The record and its sides remain untouched.
    pub fn loop_around(bounds: WorldRect, from_side: Side, to_side: Side) -> Result<Self> {
        Self::loop_between(
            bounds,
            from_side.point(bounds),
            from_side,
            to_side.point(bounds),
            to_side,
        )
    }

    /// Route exact same-node anchors through an exterior ring, retaining their tangent sides.
    /// Distinct anchors on one side share its outside segment; coincident anchors retain a
    /// complete visible loop. The legacy midpoint wrapper produces its original geometry.
    pub fn loop_between(
        bounds: WorldRect,
        from: WorldPoint,
        from_side: Side,
        to: WorldPoint,
        to_side: Side,
    ) -> Result<Self> {
        WorldPoint::new(from.x, from.y)?;
        WorldPoint::new(to.x, to.y)?;
        if !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width <= 0.0
            || bounds.height <= 0.0
        {
            return Err(CanvasError::Invalid("invalid connector loop bounds".into()));
        }
        let left = bounds.x.min(from.x).min(to.x) - 20.0;
        let top = bounds.y.min(from.y).min(to.y) - 20.0;
        let right = (bounds.x + bounds.width).max(from.x).max(to.x) + 20.0;
        let bottom = (bounds.y + bounds.height).max(from.y).max(to.y) + 20.0;
        let corners = [
            WorldPoint { x: right, y: top },
            WorldPoint {
                x: right,
                y: bottom,
            },
            WorldPoint { x: left, y: bottom },
            WorldPoint { x: left, y: top },
        ];
        let index = |side| match side {
            Side::Top => 0usize,
            Side::Right => 1,
            Side::Bottom => 2,
            Side::Left => 3,
        };
        let gap = |anchor: WorldPoint, side: Side| match side {
            Side::Top => WorldPoint {
                x: anchor.x,
                y: top,
            },
            Side::Right => WorldPoint {
                x: right,
                y: anchor.y,
            },
            Side::Bottom => WorldPoint {
                x: anchor.x,
                y: bottom,
            },
            Side::Left => WorldPoint {
                x: left,
                y: anchor.y,
            },
        };
        let mut points = vec![from, gap(from, from_side)];
        let mut current = index(from_side);
        let clockwise_steps = (index(to_side) + 4 - current) % 4;
        let clockwise = clockwise_steps != 3;
        let steps = if clockwise_steps == 0 && from != to {
            0
        } else if clockwise_steps == 0 {
            4
        } else if clockwise {
            clockwise_steps
        } else {
            1
        };
        for _ in 0..steps {
            if clockwise {
                points.push(corners[current]);
                current = (current + 1) % 4;
            } else {
                current = (current + 3) % 4;
                points.push(corners[current]);
            }
        }
        points.push(gap(to, to_side));
        points.push(to);
        for point in &points {
            WorldPoint::new(point.x, point.y)?;
        }
        let mut longest = 0.0;
        let mut label = points[1];
        for segment in points[1..points.len() - 1].windows(2) {
            let distance = (segment[0].x - segment[1].x).hypot(segment[0].y - segment[1].y);
            if distance > longest {
                longest = distance;
                label = midpoint(segment[0], segment[1]);
            }
        }
        Ok(Self { points, label })
    }

    /// Exact routed vertices before rounding, bounded to eight points including self-loops.
    pub fn waypoints(&self) -> &[WorldPoint] {
        &self.points
    }

    /// Conservative route bounds used before converting relative coordinates to f32.
    pub fn bounds(&self) -> WorldRect {
        let x = self
            .points
            .iter()
            .map(|point| point.x)
            .fold(f64::INFINITY, f64::min);
        let y = self
            .points
            .iter()
            .map(|point| point.y)
            .fold(f64::INFINITY, f64::min);
        WorldRect {
            x,
            y,
            width: self
                .points
                .iter()
                .map(|point| point.x)
                .fold(f64::NEG_INFINITY, f64::max)
                - x,
            height: self
                .points
                .iter()
                .map(|point| point.y)
                .fold(f64::NEG_INFINITY, f64::max)
                - y,
        }
    }

    /// Flatten the exact line/quadratic route with charged work and a complete segment cap.
    pub fn flatten(
        &self,
        tolerance: f64,
        max_segments: usize,
        work: &mut WorkControl<'_>,
    ) -> Result<Vec<WorldPoint>> {
        if !tolerance.is_finite() || tolerance <= 0.0 || max_segments == 0 || max_segments > 4096 {
            return Err(CanvasError::Invalid(
                "invalid connector flatten bounds".into(),
            ));
        }
        let mut output = Vec::with_capacity(32.min(max_segments + 1));
        push(&mut output, self.points[0], max_segments, work)?;
        for triple in self.points.windows(3) {
            let [a, b, c] = [triple[0], triple[1], triple[2]];
            if (a.x == b.x && b.x == c.x) || (a.y == b.y && b.y == c.y) {
                push(&mut output, b, max_segments, work)?;
                continue;
            }
            let radius = (a.x - b.x)
                .hypot(a.y - b.y)
                .min((c.x - b.x).hypot(c.y - b.y))
                * 0.5;
            let radius = radius.min(5.0);
            let (start, end) = if a.y == b.y {
                (
                    WorldPoint {
                        x: b.x + if a.x < c.x { -radius } else { radius },
                        y: b.y,
                    },
                    WorldPoint {
                        x: b.x,
                        y: b.y + if a.y < c.y { radius } else { -radius },
                    },
                )
            } else {
                (
                    WorldPoint {
                        x: b.x,
                        y: b.y + if a.y < c.y { -radius } else { radius },
                    },
                    WorldPoint {
                        x: b.x + if a.x < c.x { radius } else { -radius },
                        y: b.y,
                    },
                )
            };
            push(&mut output, start, max_segments, work)?;
            // An exact quadratic-to-cubic conversion reuses bounded adaptive subdivision.
            let curve = CubicConnector {
                from: start,
                control_from: translated(
                    start,
                    WorldPoint {
                        x: b.x - start.x,
                        y: b.y - start.y,
                    },
                    2.0 / 3.0,
                ),
                control_to: translated(
                    end,
                    WorldPoint {
                        x: b.x - end.x,
                        y: b.y - end.y,
                    },
                    2.0 / 3.0,
                ),
                to: end,
            };
            let remaining = max_segments.saturating_sub(output.len() - 1);
            if remaining == 0 {
                return Err(CanvasError::Limit("connector segments"));
            }
            for point in curve
                .flatten(tolerance, remaining, work)?
                .into_iter()
                .skip(1)
            {
                push(&mut output, point, max_segments, work)?;
            }
        }
        push(
            &mut output,
            self.points[self.points.len() - 1],
            max_segments,
            work,
        )?;
        Ok(output)
    }
}

fn push(
    points: &mut Vec<WorldPoint>,
    point: WorldPoint,
    max_segments: usize,
    work: &mut WorkControl<'_>,
) -> Result<()> {
    work.charge(1)?;
    if points.last() == Some(&point) {
        return Ok(());
    }
    if points.len() > max_segments {
        return Err(CanvasError::Limit("connector segments"));
    }
    points.push(point);
    Ok(())
}
fn direction(side: Side) -> WorldPoint {
    match side {
        Side::Top => WorldPoint { x: 0.0, y: -1.0 },
        Side::Right => WorldPoint { x: 1.0, y: 0.0 },
        Side::Bottom => WorldPoint { x: 0.0, y: 1.0 },
        Side::Left => WorldPoint { x: -1.0, y: 0.0 },
    }
}
fn translated(point: WorldPoint, direction: WorldPoint, distance: f64) -> WorldPoint {
    WorldPoint {
        x: point.x + direction.x * distance,
        y: point.y + direction.y * distance,
    }
}
fn midpoint(a: WorldPoint, b: WorldPoint) -> WorldPoint {
    WorldPoint {
        x: a.x + (b.x - a.x) * 0.5,
        y: a.y + (b.y - a.y) * 0.5,
    }
}
