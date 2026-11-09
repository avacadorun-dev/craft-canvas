//! One bounded frame of named/legacy anchors for painting, routing and hit testing.
use crate::{CanvasViewHost, view::VisibleEdgeGeometry};
use craft_canvas_model::{
    Camera, CanvasDialect, CanvasError, CanvasSession, ConnectionError, ConnectorEnd,
    ConnectorEndpointKind, EdgeRecord, ExtensionStatus, NodeRecord, PortBinding, PortEndpoint,
    PortKey, PortKind, ReconnectPolicy, Result, Side, SmoothStepConnector, WorkControl, WorldPoint,
    WorldRect,
};
use egui::{Pos2, Rect, Vec2};
use std::collections::HashMap;

/// Current-frame measured geometry for an existing canonical named identity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PortAnchor {
    /// Finite node-local point; measuring never creates or renames a port.
    pub local: WorldPoint,
    /// Connector tangent at this measured anchor.
    pub side: Side,
}

/// An actually visible port, shared by its paint and input paths.
#[derive(Clone, Debug, PartialEq)]
pub struct VisiblePortGeometry {
    /// Exact named or explicit legacy-side identity.
    pub endpoint: PortEndpoint,
    /// Current connector tangent.
    pub side: Side,
    /// Current finite world anchor.
    pub world: WorldPoint,
    /// Current screen anchor in egui points.
    pub center: Pos2,
    /// Painted port bounds intersected with the actual viewport clip.
    pub hit_rect: Rect,
    /// Combined node and port eligibility for new connections.
    pub connectable: bool,
    /// Whether this port may initiate a connection.
    pub start: bool,
    /// Whether this port may receive a connection.
    pub end: bool,
}

/// Selected-edge endpoint affordance retaining exact and displayed bindings separately.
#[derive(Clone, Debug, PartialEq)]
pub struct VisibleReconnectHandle {
    /// Stable canonical edge identity.
    pub edge_id: String,
    /// Endpoint deliberately being moved.
    pub end: ConnectorEndpointKind,
    /// Exact saved endpoint, including omitted legacy side.
    pub original: PortEndpoint,
    /// Explicit displayed identity; omitted legacy side is displayed at Top.
    pub display_endpoint: PortEndpoint,
    /// Current connector tangent.
    pub side: Side,
    /// Current world anchor.
    pub world: WorldPoint,
    /// True screen anchor; leader lines connect it to any displaced grip.
    pub anchor_center: Pos2,
    /// Painted grip center; overlapping endpoint grips are separated for direct access.
    pub center: Pos2,
    /// Current clipped endpoint handle bounds.
    pub hit_rect: Rect,
}

#[derive(Clone, Copy)]
pub(crate) struct PortFrameConfig {
    pub camera: Camera,
    pub viewport: Rect,
    pub clip: Rect,
    pub port_radius: f32,
    pub reconnect_radius: f32,
}

pub(crate) struct ResolvedPort {
    pub endpoint: PortEndpoint,
    pub display_endpoint: PortEndpoint,
    pub world: WorldPoint,
    pub side: Side,
    pub node_bounds: WorldRect,
}

#[derive(Clone, Copy)]
struct Anchor {
    world: WorldPoint,
    side: Side,
    connectable: bool,
    start: bool,
    end: bool,
}
struct FrameNode {
    bounds: WorldRect,
    order: usize,
    status: ExtensionStatus,
    legacy: [Anchor; 4],
    named: HashMap<PortKey, Anchor>,
}

/// Disposable derived lookup; the session remains the only canonical graph.
pub(crate) struct PortFrame {
    pub config: PortFrameConfig,
    pub ports: Vec<VisiblePortGeometry>,
    nodes: HashMap<String, FrameNode>,
    port_orders: Vec<usize>,
    occlusion: Occlusion,
}
impl PortFrame {
    pub fn empty(config: PortFrameConfig) -> Self {
        Self {
            config,
            ports: vec![],
            nodes: HashMap::new(),
            port_orders: vec![],
            occlusion: Occlusion::new(config.clip.min),
        }
    }

    pub fn build(
        session: &CanvasSession,
        dialect: &dyn CanvasDialect,
        host: &dyn CanvasViewHost,
        config: PortFrameConfig,
        preview_bounds: impl Fn(&NodeRecord) -> WorldRect,
        work: &mut WorkControl<'_>,
    ) -> Result<Self> {
        validate_config(config)?;
        let mut frame = Self::empty(PortFrameConfig {
            clip: config.clip.intersect(config.viewport),
            ..config
        });
        let world_clip = frame.world_rect(frame.config.clip);
        for (order, node) in session.document().nodes().iter().enumerate() {
            work.charge(1)?;
            let bounds = preview_bounds(node);
            if ![bounds.x, bounds.y, bounds.width, bounds.height]
                .into_iter()
                .all(f64::is_finite)
                || bounds.width <= 0.0
                || bounds.height <= 0.0
            {
                return Err(CanvasError::Invalid("invalid port node geometry".into()));
            }
            let eligible = dialect.can_connect(node);
            let legacy = Side::ALL.map(|side| Anchor {
                world: side.point(bounds),
                side,
                connectable: eligible,
                start: true,
                end: true,
            });
            for anchor in legacy {
                work.charge(1)?;
                WorldPoint::new(anchor.world.x, anchor.world.y)?;
            }
            let mut named = HashMap::with_capacity(node.ports().len());
            for port in node.ports() {
                work.charge(1)?;
                let measured = host.port_anchor(node, &port.key).unwrap_or(PortAnchor {
                    local: WorldPoint {
                        x: port.x,
                        y: port.y,
                    },
                    side: port.side,
                });
                if !measured.local.x.is_finite()
                    || !measured.local.y.is_finite()
                    || measured.local.x.abs() > session.limits().max_coordinate
                    || measured.local.y.abs() > session.limits().max_coordinate
                {
                    return Err(CanvasError::Limit("measured port coordinate"));
                }
                let world =
                    WorldPoint::new(bounds.x + measured.local.x, bounds.y + measured.local.y)?;
                named.insert(
                    port.key.clone(),
                    Anchor {
                        world,
                        side: measured.side,
                        connectable: eligible && port.connectable,
                        start: port.start,
                        end: port.end,
                    },
                );
            }
            if frame.config.clip.is_positive() && bounds.intersects(world_clip) {
                let visible = WorldRect {
                    x: bounds.x.max(world_clip.x),
                    y: bounds.y.max(world_clip.y),
                    width: (bounds.x + bounds.width).min(world_clip.x + world_clip.width)
                        - bounds.x.max(world_clip.x),
                    height: (bounds.y + bounds.height).min(world_clip.y + world_clip.height)
                        - bounds.y.max(world_clip.y),
                };
                let rect = Rect::from_min_max(
                    frame.screen(WorldPoint {
                        x: visible.x,
                        y: visible.y,
                    })?,
                    frame.screen(WorldPoint {
                        x: visible.x + visible.width,
                        y: visible.y + visible.height,
                    })?,
                );
                frame.occlusion.insert(order, rect, work)?;
            }
            frame.nodes.insert(
                node.id().into(),
                FrameNode {
                    bounds,
                    order,
                    status: node.port_extension_status(),
                    legacy,
                    named,
                },
            );
        }
        // Canonical node/port order also determines deterministic painted-handle ties.
        for node in session.document().nodes() {
            let derived = frame
                .nodes
                .get(node.id())
                .ok_or_else(|| CanvasError::NotFound(node.id().into()))?;
            let order = derived.order;
            let mut entries = Vec::with_capacity(4 + node.ports().len());
            for (side, anchor) in Side::ALL.into_iter().zip(derived.legacy) {
                entries.push((PortBinding::Legacy(Some(side)), anchor));
            }
            for port in node.ports() {
                if let Some(anchor) = derived.named.get(&port.key) {
                    entries.push((PortBinding::Named(port.key.clone()), *anchor));
                }
            }
            for (binding, anchor) in entries {
                work.charge(1)?;
                if let Some(port) =
                    frame.visible_port(node.id(), binding, anchor, frame.config.port_radius)?
                    && frame
                        .occlusion
                        .topmost(port.center, work)?
                        .is_none_or(|top| top <= order)
                {
                    frame.ports.push(port);
                    frame.port_orders.push(order);
                }
            }
        }
        Ok(frame)
    }

    pub fn resolve(&self, endpoint: &PortEndpoint) -> Result<ResolvedPort> {
        let node = self
            .nodes
            .get(&endpoint.node_id)
            .ok_or_else(|| ConnectionError::MissingNode(endpoint.node_id.clone()))?;
        let (anchor, display) = match &endpoint.binding {
            PortBinding::Legacy(side) => {
                let side = side.unwrap_or(Side::Top);
                (
                    node.legacy[side_index(side)],
                    PortBinding::Legacy(Some(side)),
                )
            }
            PortBinding::Named(key) => {
                match node.status {
                    ExtensionStatus::Opaque => {
                        return Err(
                            ConnectionError::OpaqueExtension(endpoint.node_id.clone()).into()
                        );
                    }
                    ExtensionStatus::Malformed => {
                        return Err(
                            ConnectionError::MalformedExtension(endpoint.node_id.clone()).into(),
                        );
                    }
                    _ => {}
                }
                let anchor =
                    node.named
                        .get(key)
                        .copied()
                        .ok_or_else(|| ConnectionError::MissingPort {
                            node_id: endpoint.node_id.clone(),
                            port: key.clone(),
                        })?;
                (anchor, PortBinding::Named(key.clone()))
            }
        };
        Ok(ResolvedPort {
            endpoint: endpoint.clone(),
            display_endpoint: PortEndpoint {
                node_id: endpoint.node_id.clone(),
                binding: display,
            },
            world: anchor.world,
            side: anchor.side,
            node_bounds: node.bounds,
        })
    }

    pub fn route(&self, from: &PortEndpoint, to: &PortEndpoint) -> Result<SmoothStepConnector> {
        let a = self.resolve(from)?;
        let b = self.resolve(to)?;
        if a.endpoint.node_id == b.endpoint.node_id {
            SmoothStepConnector::loop_between(a.node_bounds, a.world, a.side, b.world, b.side)
        } else {
            SmoothStepConnector::between(a.world, a.side, b.world, b.side)
        }
    }

    pub fn edge_geometry(
        &self,
        edge: &EdgeRecord,
        work: &mut WorkControl<'_>,
    ) -> Result<Option<VisibleEdgeGeometry>> {
        work.charge(1)?;
        let route = self.route(
            &edge.endpoint(ConnectorEndpointKind::From)?,
            &edge.endpoint(ConnectorEndpointKind::To)?,
        )?;
        if !self.config.clip.is_positive()
            || !route
                .bounds()
                .intersects(self.world_rect(self.config.clip.expand(12.0)))
        {
            return Ok(None);
        }
        let points = route
            .flatten(0.35 / self.config.camera.zoom(), 64, work)?
            .into_iter()
            .map(|p| self.screen(p))
            .collect::<Result<Vec<_>>>()?;
        let marker = (8.0 * self.config.camera.zoom()) as f32;
        Ok(Some(VisibleEdgeGeometry {
            edge_id: edge.id().into(),
            label_anchor: self.screen(route.label_anchor())?,
            source_arrow: (edge.from_end() == ConnectorEnd::Arrow)
                .then(|| arrow(&points, false, marker))
                .flatten(),
            target_arrow: (edge.to_end() == ConnectorEnd::Arrow)
                .then(|| arrow(&points, true, marker))
                .flatten(),
            points,
        }))
    }

    pub fn reconnect_handles(
        &self,
        edge: &EdgeRecord,
        dialect: &dyn CanvasDialect,
        work: &mut WorkControl<'_>,
    ) -> Result<Vec<VisibleReconnectHandle>> {
        let policy = match edge.reconnect_policy()? {
            ReconnectPolicy::Inherit => dialect.default_reconnect_policy(),
            policy => policy,
        };
        if policy == ReconnectPolicy::Inherit {
            return Err(ConnectionError::InvalidReconnectDefault.into());
        }
        let mut handles = vec![];
        for end in [ConnectorEndpointKind::From, ConnectorEndpointKind::To] {
            work.charge(1)?;
            if !match policy {
                ReconnectPolicy::Both => true,
                ReconnectPolicy::From => end == ConnectorEndpointKind::From,
                ReconnectPolicy::To => end == ConnectorEndpointKind::To,
                _ => false,
            } {
                continue;
            }
            let original = edge.endpoint(end)?;
            let anchor = self.resolve(&original)?;
            let order = self
                .nodes
                .get(&original.node_id)
                .map(|node| node.order)
                .ok_or_else(|| CanvasError::NotFound(original.node_id.clone()))?;
            let Some(visible) = self.visible_port(
                &original.node_id,
                anchor.display_endpoint.binding.clone(),
                Anchor {
                    world: anchor.world,
                    side: anchor.side,
                    connectable: true,
                    start: true,
                    end: true,
                },
                self.config.reconnect_radius,
            )?
            else {
                continue;
            };
            if self
                .occlusion
                .topmost(visible.center, work)?
                .is_some_and(|top| top > order)
            {
                continue;
            }
            handles.push(VisibleReconnectHandle {
                edge_id: edge.id().into(),
                end,
                original,
                display_endpoint: anchor.display_endpoint,
                side: anchor.side,
                world: anchor.world,
                anchor_center: visible.center,
                center: visible.center,
                hit_rect: visible.hit_rect,
            });
        }
        if handles.len() == 2 && handles[0].hit_rect.intersects(handles[1].hit_rect) {
            self.separate_reconnect_handles(&mut handles, work)?;
        }
        let mut visible = Vec::with_capacity(handles.len());
        for handle in handles {
            let order = self
                .nodes
                .get(&handle.original.node_id)
                .map(|node| node.order);
            if handle.hit_rect.is_positive()
                && self
                    .occlusion
                    .topmost(handle.center, work)?
                    .zip(order)
                    .is_none_or(|(top, own)| top <= own)
            {
                visible.push(handle);
            }
        }
        Ok(visible)
    }

    fn separate_reconnect_handles(
        &self,
        handles: &mut [VisibleReconnectHandle],
        work: &mut WorkControl<'_>,
    ) -> Result<()> {
        let [from, to] = handles else {
            return Ok(());
        };
        let tangent = |side| match side {
            Side::Top | Side::Bottom => Vec2::X,
            Side::Left | Side::Right => Vec2::Y,
        };
        let radius = self.config.reconnect_radius;
        let step = radius + 2.0;
        // Keep each grip on its own anchor's tangent. Moving from their midpoint
        // or along the normal can push a previously visible grip behind its neighbour.
        // One-sided alternatives keep both ends reachable near clipping or occlusion.
        for (from_steps, to_steps) in [
            (-1.0, 1.0),
            (1.0, -1.0),
            (-2.0, 0.0),
            (2.0, 0.0),
            (0.0, -2.0),
            (0.0, 2.0),
            (-2.0, 2.0),
            (2.0, -2.0),
            (-2.0, -1.0),
            (-1.0, -2.0),
            (1.0, 2.0),
            (2.0, 1.0),
        ] {
            work.charge(1)?;
            let centers = [
                from.anchor_center + tangent(from.side) * (step * from_steps),
                to.anchor_center + tangent(to.side) * (step * to_steps),
            ];
            let rects = centers.map(|center| {
                Rect::from_center_size(center, Vec2::splat(radius * 2.0))
                    .intersect(self.config.clip)
            });
            if rects[0].intersects(rects[1])
                || rects.iter().any(|rect| !rect.is_positive())
                || centers
                    .iter()
                    .any(|center| !self.config.clip.contains(*center))
            {
                continue;
            }
            let mut unoccluded = true;
            for (handle, center) in [(&*from, centers[0]), (&*to, centers[1])] {
                let order = self
                    .nodes
                    .get(&handle.original.node_id)
                    .map(|node| node.order);
                if self
                    .occlusion
                    .topmost(center, work)?
                    .zip(order)
                    .is_some_and(|(top, own)| top > own)
                {
                    unoccluded = false;
                    break;
                }
            }
            if unoccluded {
                from.center = centers[0];
                from.hit_rect = rects[0];
                to.center = centers[1];
                to.hit_rect = rects[1];
                return Ok(());
            }
        }
        // A crowded/very narrow clip may offer no bounded placement. Retain both
        // original visible grips instead of hiding an endpoint during displacement.
        Ok(())
    }

    pub fn world_at(&self, screen: Pos2) -> WorldPoint {
        self.config.camera.view_to_world(
            WorldPoint {
                x: f64::from(screen.x - self.config.viewport.left()),
                y: f64::from(screen.y - self.config.viewport.top()),
            },
            WorldPoint {
                x: f64::from(self.config.viewport.width()),
                y: f64::from(self.config.viewport.height()),
            },
        )
    }
    fn world_rect(&self, rect: Rect) -> WorldRect {
        let min = self.world_at(rect.min);
        let max = self.world_at(rect.max);
        WorldRect {
            x: min.x,
            y: min.y,
            width: max.x - min.x,
            height: max.y - min.y,
        }
    }
    pub fn screen(&self, world: WorldPoint) -> Result<Pos2> {
        let local = self.config.camera.world_to_view(
            world,
            WorldPoint {
                x: f64::from(self.config.viewport.width()),
                y: f64::from(self.config.viewport.height()),
            },
        );
        let p = Pos2::new(
            self.config.viewport.left() + local.x as f32,
            self.config.viewport.top() + local.y as f32,
        );
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(CanvasError::Limit("port screen coordinate"));
        }
        Ok(p)
    }
    fn visible_port(
        &self,
        node_id: &str,
        binding: PortBinding,
        anchor: Anchor,
        radius: f32,
    ) -> Result<Option<VisiblePortGeometry>> {
        if !self.config.clip.is_positive()
            || !self
                .world_rect(self.config.clip.expand(radius))
                .contains(anchor.world)
        {
            return Ok(None);
        }
        let center = self.screen(anchor.world)?;
        let hit_rect =
            Rect::from_center_size(center, Vec2::splat(radius * 2.0)).intersect(self.config.clip);
        Ok(hit_rect.is_positive().then(|| VisiblePortGeometry {
            endpoint: PortEndpoint {
                node_id: node_id.into(),
                binding,
            },
            side: anchor.side,
            world: anchor.world,
            center,
            hit_rect,
            connectable: anchor.connectable,
            start: anchor.start,
            end: anchor.end,
        }))
    }
}

/// Direct painted hits win even when invalid; proximity only resolves empty painted space.
pub(crate) fn hit_port<'a>(
    frame: &'a PortFrame,
    pointer: Pos2,
    radius: f32,
    origin: Option<&PortEndpoint>,
) -> Option<&'a VisiblePortGeometry> {
    if !frame.config.clip.contains(pointer) || !radius.is_finite() || radius < 0.0 {
        return None;
    }
    let top = frame
        .occlusion
        .at(pointer)
        .iter()
        .rev()
        .find(|(_, rect)| rect.contains(pointer))
        .map(|(order, _)| *order);
    let unoccluded = |index: usize| top.is_none_or(|top| frame.port_orders[index] >= top);
    if let Some((_, port)) = frame
        .ports
        .iter()
        .enumerate()
        .rev()
        .find(|(i, port)| unoccluded(*i) && port.hit_rect.contains(pointer))
    {
        return Some(port);
    }
    let preferred = origin.and_then(|origin| match &origin.binding {
        PortBinding::Named(key) => Some(match key.kind {
            PortKind::Source => PortKind::Target,
            PortKind::Target => PortKind::Source,
        }),
        _ => None,
    });
    let mut best: Option<(usize, f32, bool)> = None;
    for (index, port) in frame.ports.iter().enumerate() {
        if !unoccluded(index) {
            continue;
        }
        let distance = port.center.distance_sq(pointer);
        if distance > radius * radius {
            continue;
        }
        let opposite = matches!((&port.endpoint.binding, preferred), (PortBinding::Named(key), Some(kind)) if key.kind == kind);
        if best.is_none_or(|(_, old, old_opposite)| {
            distance < old || (distance == old && (opposite || !old_opposite))
        }) {
            best = Some((index, distance, opposite));
        }
    }
    best.map(|(index, _, _)| &frame.ports[index])
}

// Small screen buckets avoid testing every node against every visible handle.
struct Occlusion {
    origin: Pos2,
    cells: HashMap<(i32, i32), Vec<(usize, Rect)>>,
}
impl Occlusion {
    fn new(origin: Pos2) -> Self {
        Self {
            origin,
            cells: HashMap::new(),
        }
    }
    fn cell(&self, p: Pos2) -> (i32, i32) {
        (
            ((p.x - self.origin.x) / 64.0).floor() as i32,
            ((p.y - self.origin.y) / 64.0).floor() as i32,
        )
    }
    fn at(&self, p: Pos2) -> &[(usize, Rect)] {
        self.cells.get(&self.cell(p)).map_or(&[], Vec::as_slice)
    }
    fn insert(&mut self, order: usize, rect: Rect, work: &mut WorkControl<'_>) -> Result<()> {
        if !rect.is_positive() {
            return Ok(());
        }
        let (x0, y0) = self.cell(rect.min);
        let (x1, y1) = self.cell(rect.max);
        for x in x0..=x1 {
            for y in y0..=y1 {
                work.charge(1)?;
                self.cells.entry((x, y)).or_default().push((order, rect));
            }
        }
        Ok(())
    }
    fn topmost(&self, p: Pos2, work: &mut WorkControl<'_>) -> Result<Option<usize>> {
        for (order, rect) in self.at(p).iter().rev() {
            work.charge(1)?;
            if rect.contains(p) {
                return Ok(Some(*order));
            }
        }
        Ok(None)
    }
}
fn side_index(side: Side) -> usize {
    match side {
        Side::Top => 0,
        Side::Right => 1,
        Side::Bottom => 2,
        Side::Left => 3,
    }
}
fn validate_config(config: PortFrameConfig) -> Result<()> {
    if ![
        config.viewport.min.x,
        config.viewport.min.y,
        config.viewport.max.x,
        config.viewport.max.y,
        config.clip.min.x,
        config.clip.min.y,
        config.clip.max.x,
        config.clip.max.y,
        config.port_radius,
        config.reconnect_radius,
    ]
    .into_iter()
    .all(f32::is_finite)
        || !config.viewport.is_positive()
        || config.port_radius <= 0.0
        || config.port_radius > 1024.0
        || config.reconnect_radius <= 0.0
        || config.reconnect_radius > 1024.0
    {
        return Err(CanvasError::Invalid("invalid port frame geometry".into()));
    }
    Ok(())
}
fn arrow(points: &[Pos2], at_end: bool, size: f32) -> Option<[Pos2; 3]> {
    let (tip, neighbor) = if at_end {
        let tip = *points.last()?;
        (tip, *points.iter().rev().find(|point| **point != tip)?)
    } else {
        let tip = *points.first()?;
        (tip, *points.iter().find(|point| **point != tip)?)
    };
    let direction = (neighbor - tip).normalized();
    let center = tip + direction * size;
    let perpendicular = Vec2::new(-direction.y, direction.x) * size * 0.5;
    Some([tip, center + perpendicular, center - perpendicular])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeBodyResponse, PlainTextBody};
    use craft_canvas_model::{CanvasDocument, CanvasLimits, StandardDialect};
    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    fn test_session(raw: &str) -> Result<CanvasSession> {
        let limits = CanvasLimits::default();
        CanvasSession::new(CanvasDocument::parse(raw, &limits)?, limits)
    }
    fn config() -> PortFrameConfig {
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        PortFrameConfig {
            camera: Camera::default(),
            viewport,
            clip: viewport,
            port_radius: 5.0,
            reconnect_radius: 8.0,
        }
    }
    fn test_frame(session: &CanvasSession, host: &dyn CanvasViewHost) -> Result<PortFrame> {
        PortFrame::build(
            session,
            &StandardDialect,
            host,
            config(),
            NodeRecord::bounds,
            &mut WorkControl::new(100_000, None),
        )
    }
    fn named(node: &str, id: &str, kind: PortKind) -> PortEndpoint {
        PortEndpoint {
            node_id: node.into(),
            binding: PortBinding::Named(PortKey {
                id: id.into(),
                kind,
            }),
        }
    }
    struct Measured;
    impl CanvasViewHost for Measured {
        fn paint_body(
            &mut self,
            _: &mut egui::Ui,
            _: &NodeRecord,
            _: Rect,
            _: f64,
        ) -> NodeBodyResponse {
            NodeBodyResponse::default()
        }
        fn port_anchor(&self, _: &NodeRecord, key: &PortKey) -> Option<PortAnchor> {
            (key.id == "detached").then_some(PortAnchor {
                local: WorldPoint {
                    x: 1020.25,
                    y: 30.5,
                },
                side: Side::Right,
            })
        }
    }

    #[test]
    fn measured_anchor_is_visible_even_when_its_node_body_is_outside_the_frame() -> TestResult {
        let session = test_session(
            r#"{"nodes":[{"id":"a","type":"text","x":-1000,"y":0,"width":100,"height":100,"text":"A","craftCanvas":{"version":1,"ports":[{"id":"detached","kind":"source","side":"left","x":0,"y":0}]}}],"edges":[]}"#,
        )?;
        let frame = test_frame(&session, &Measured)?;
        let port = frame
            .ports
            .iter()
            .find(|port| port.endpoint == named("a", "detached", PortKind::Source))
            .ok_or("detached port")?;
        assert_eq!(port.world, WorldPoint { x: 20.25, y: 30.5 });
        assert_eq!(port.center, Pos2::new(420.25, 330.5));
        assert_eq!(port.side, Side::Right);
        assert_eq!(
            hit_port(&frame, port.center, 20.0, None).map(|p| &p.endpoint),
            Some(&port.endpoint)
        );
        assert_eq!(session.document().node("a").ok_or("a")?.ports()[0].x, 0.0);
        Ok(())
    }

    #[test]
    fn legacy_reconnect_handle_keeps_original_omission_and_the_existing_route() -> TestResult {
        let session = test_session(
            r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":100,"height":100,"text":"A"}],"edges":[{"id":"e","fromNode":"a","toNode":"a","craftCanvas":{"version":1,"reconnect":"from"}}]}"#,
        )?;
        let frame = test_frame(&session, &PlainTextBody)?;
        let edge = session.document().edge("e").ok_or("e")?;
        let mut work = WorkControl::new(1000, None);
        let handles = frame.reconnect_handles(edge, &StandardDialect, &mut work)?;
        assert_eq!(handles.len(), 1);
        assert_eq!(handles[0].end, ConnectorEndpointKind::From);
        assert_eq!(handles[0].original.binding, PortBinding::Legacy(None));
        assert_eq!(
            handles[0].display_endpoint.binding,
            PortBinding::Legacy(Some(Side::Top))
        );
        let geometry = frame
            .edge_geometry(edge, &mut work)?
            .ok_or("visible loop")?;
        assert_eq!(geometry.points.first(), Some(&handles[0].center));
        assert_eq!(geometry.points.last(), Some(&handles[0].center));
        assert!(geometry.points.len() > 8 && geometry.target_arrow.is_some());
        Ok(())
    }

    #[test]
    fn adjacent_reconnect_grips_stay_on_their_own_visible_anchor_tangents() -> TestResult {
        let session = test_session(
            r#"{"nodes":[{"id":"a","type":"text","x":-100,"y":-50,"width":100,"height":100,"text":"A"},{"id":"b","type":"text","x":10,"y":-50,"width":100,"height":100,"text":"B"}],"edges":[{"id":"e","fromNode":"a","fromSide":"right","toNode":"b","toSide":"left"}]}"#,
        )?;
        let original = session.document().raw_json().to_owned();
        let frame = test_frame(&session, &PlainTextBody)?;
        let edge = session.document().edge("e").ok_or("adjacent edge")?;
        let mut work = WorkControl::new(1000, None);
        let handles = frame.reconnect_handles(edge, &StandardDialect, &mut work)?;
        assert_eq!(handles.len(), 2, "neither grip moves behind the later node");
        assert!(!handles[0].hit_rect.intersects(handles[1].hit_rect));
        assert_eq!(handles[0].world, WorldPoint { x: 0.0, y: 0.0 });
        assert_eq!(handles[1].world, WorldPoint { x: 10.0, y: 0.0 });
        for handle in &handles {
            assert_eq!(handle.center.x, handle.anchor_center.x);
            assert_eq!(handle.anchor_center, frame.screen(handle.world)?);
            assert!(frame.config.clip.contains(handle.center));
        }
        let geometry = frame
            .edge_geometry(edge, &mut work)?
            .ok_or("visible edge")?;
        assert_eq!(geometry.points.first(), Some(&handles[0].anchor_center));
        assert_eq!(geometry.points.last(), Some(&handles[1].anchor_center));
        assert_eq!(session.document().raw_json(), original);
        Ok(())
    }

    #[test]
    fn coincident_loop_grips_separate_even_when_one_tangent_direction_is_clipped() -> TestResult {
        let session = test_session(
            r#"{"nodes":[{"id":"a","type":"text","x":-50,"y":-50,"width":100,"height":100,"text":"A"}],"edges":[{"id":"e","fromNode":"a","toNode":"a"}]}"#,
        )?;
        let edge = session.document().edge("e").ok_or("loop edge")?;
        for clipped in [false, true] {
            let mut config = config();
            if clipped {
                config.clip.min.x = 400.0;
            }
            let mut work = WorkControl::new(1000, None);
            let frame = PortFrame::build(
                &session,
                &StandardDialect,
                &PlainTextBody,
                config,
                NodeRecord::bounds,
                &mut work,
            )?;
            let handles = frame.reconnect_handles(edge, &StandardDialect, &mut work)?;
            assert_eq!(handles.len(), 2);
            assert!(!handles[0].hit_rect.intersects(handles[1].hit_rect));
            assert_eq!(handles[0].world, handles[1].world);
            assert_eq!(handles[0].anchor_center, handles[1].anchor_center);
            for handle in &handles {
                assert_eq!(handle.original.binding, PortBinding::Legacy(None));
                assert_eq!(handle.anchor_center, frame.screen(handle.world)?);
                assert_eq!(handle.center.y, handle.anchor_center.y);
                assert!(frame.config.clip.contains(handle.center));
            }
        }
        Ok(())
    }

    #[test]
    fn invalid_direct_hit_wins_and_later_node_occlusion_prevents_hidden_targets() -> TestResult {
        let raw = r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":200,"height":120,"text":"A","craftCanvas":{"version":1,"ports":[{"id":"bad","kind":"source","side":"right","x":200,"y":30,"connectable":false},{"id":"good","kind":"target","side":"right","x":200,"y":44}]}}],"edges":[]}"#;
        let session = test_session(raw)?;
        let frame = test_frame(&session, &PlainTextBody)?;
        let bad = frame
            .ports
            .iter()
            .find(|p| p.endpoint == named("a", "bad", PortKind::Source))
            .ok_or("bad port")?;
        let hit = hit_port(&frame, bad.center, 20.0, None).ok_or("hit")?;
        assert_eq!(hit.endpoint, bad.endpoint);
        assert!(!hit.connectable);
        let blocked_raw = raw.replace("}],\"edges\":[]}", "},{\"id\":\"front\",\"type\":\"text\",\"x\":190,\"y\":20,\"width\":80,\"height\":80,\"text\":\"Front\"}],\"edges\":[]}");
        let blocked = test_session(&blocked_raw)?;
        let blocked_frame = test_frame(&blocked, &PlainTextBody)?;
        assert!(
            !blocked_frame
                .ports
                .iter()
                .any(|p| p.endpoint == bad.endpoint)
        );
        assert!(
            hit_port(&blocked_frame, bad.center, 20.0, None)
                .is_none_or(|p| p.endpoint.node_id != "a")
        );
        Ok(())
    }

    #[test]
    fn unsupported_named_edges_refuse_geometry_and_frame_work_is_bounded() -> TestResult {
        let session = test_session(
            r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":100,"height":100,"text":"A","craftCanvas":{"version":2,"ports":"future"}}],"edges":[{"id":"e","fromNode":"a","toNode":"a","craftCanvas":{"version":1,"fromPort":{"id":"missing","kind":"source"}}}]}"#,
        )?;
        let frame = test_frame(&session, &PlainTextBody)?;
        assert_eq!(frame.ports.len(), 4);
        assert!(matches!(
            frame.edge_geometry(
                session.document().edge("e").ok_or("e")?,
                &mut WorkControl::new(1000, None)
            ),
            Err(CanvasError::Connection(ConnectionError::OpaqueExtension(_)))
        ));
        assert!(matches!(
            PortFrame::build(
                &session,
                &StandardDialect,
                &PlainTextBody,
                config(),
                NodeRecord::bounds,
                &mut WorkControl::new(0, None)
            ),
            Err(CanvasError::WorkLimit)
        ));
        Ok(())
    }
}
