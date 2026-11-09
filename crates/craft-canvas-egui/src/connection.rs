//! Owned native connection gestures and exact, nonauthoritative submission receipts.
use crate::{
    CanvasViewHost,
    port_geometry::{PortFrame, VisibleReconnectHandle, hit_port},
};
use craft_canvas_model::{
    Camera, CanvasCommand, CanvasDialect, CanvasError, CanvasSession, CanvasTransaction,
    ConnectionOperation, ConnectionRequest, ConnectorEndpointKind, ContentIdentity, EdgeRecord,
    PortBinding, PortEndpoint, PortKind, PreparedChange, Result, Side, SmoothStepConnector,
    validate_connection,
};
use egui::{
    Event, Id, Key, PointerButton, Pos2, Rect, TouchDeviceId, TouchId, TouchPhase, ViewportId,
};

/// Explicit ownership of native input by an embedded control, overlay or board gesture.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CanvasInputOwnership {
    /// Pointer buttons and motion belong to the owner.
    pub pointer: bool,
    /// Keyboard commands belong to the owner.
    pub keyboard: bool,
    /// Native text composition belongs to the owner.
    pub ime: bool,
    /// Wheel, scrolling and pinch input belong to the owner.
    pub scroll: bool,
    /// Touch contacts belong to the owner.
    pub touch: bool,
}
impl CanvasInputOwnership {
    /// Combine independent child and host ownership without releasing either owner's input.
    pub fn merge(self, other: Self) -> Self {
        Self {
            pointer: self.pointer || other.pointer,
            keyboard: self.keyboard || other.keyboard,
            ime: self.ime || other.ime,
            scroll: self.scroll || other.scroll,
            touch: self.touch || other.touch,
        }
    }
    /// Whether any channel prevents a new board connection gesture.
    pub fn any(self) -> bool {
        self.pointer || self.keyboard || self.ime || self.scroll || self.touch
    }
    fn captured() -> Self {
        Self {
            pointer: true,
            keyboard: true,
            ime: true,
            scroll: true,
            touch: true,
        }
    }
}

/// Host-localized accessible connection labels; generic defaults serve examples only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionLabels {
    /// Accessible creation-port action.
    pub port: String,
    /// Accessible source-end reconnection action.
    pub reconnect_from: String,
    /// Accessible target-end reconnection action.
    pub reconnect_to: String,
    /// Pending submission status, without a claim of persistence.
    pub pending: String,
}
impl Default for ConnectionLabels {
    fn default() -> Self {
        Self {
            port: "Connect".into(),
            reconnect_from: "Reconnect source".into(),
            reconnect_to: "Reconnect target".into(),
            pending: "Waiting for acknowledgement".into(),
        }
    }
}

/// Opt-in native handle interaction settings, measured in screen points.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionOptions {
    /// Localized accessible labels supplied by the embedding host.
    pub labels: ConnectionLabels,
    /// Minimum movement before allocating an identity or beginning a connection.
    pub drag_threshold: f32,
    /// Nearest-port acquisition distance after exact painted hits are considered.
    pub target_radius: f32,
    /// Radius of a painted creation port.
    pub port_radius: f32,
    /// Radius of a selected edge's reconnect handle.
    pub reconnect_radius: f32,
    /// Host revision of policy or measured anchors; changing it cancels capture.
    pub generation: u64,
}
impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            labels: ConnectionLabels::default(),
            drag_threshold: 3.0,
            target_radius: 20.0,
            port_radius: 5.0,
            reconnect_radius: 8.0,
            generation: 0,
        }
    }
}
impl ConnectionOptions {
    /// Reject nonfinite, negative or unreasonably large interaction geometry.
    pub fn validate(&self) -> Result<()> {
        if [
            &self.labels.port,
            &self.labels.reconnect_from,
            &self.labels.reconnect_to,
            &self.labels.pending,
        ]
        .into_iter()
        .any(|label| label.len() > 4096)
        {
            return Err(CanvasError::Limit("connection label bytes"));
        }
        for value in [
            self.drag_threshold,
            self.target_radius,
            self.port_radius,
            self.reconnect_radius,
        ] {
            if !value.is_finite() || !(0.0..=256.0).contains(&value) {
                return Err(CanvasError::Limit("connection interaction radius"));
            }
        }
        if self.port_radius == 0.0 || self.reconnect_radius == 0.0 {
            return Err(CanvasError::Invalid(
                "connection handles require a positive radius".into(),
            ));
        }
        Ok(())
    }
}

/// Retained host submission status; none of these states claims durable persistence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionSubmissionState {
    /// One submitted transaction awaits an explicit host outcome.
    Awaiting,
    /// The host refused or cancelled its write; explicit retry or dismissal is possible.
    Rejected(String),
    /// The canonical base changed while the submitted request remains retained.
    Conflict,
}
/// Why an uncommitted captured connection ended without submitting a mutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionCancelReason {
    /// The author pressed Escape.
    Escape,
    /// The native window lost focus.
    FocusLost,
    /// The initiating pointer/contact disappeared or was cancelled.
    PointerLost,
    /// The initiating release occurred outside the board.
    Outside,
    /// The exact canonical content identity changed.
    BaseChanged,
    /// The host switched the board widget identity.
    BoardChanged,
    /// The native viewport or board rectangle changed.
    ViewportChanged,
    /// The camera changed during capture.
    CameraChanged,
    /// The host changed policy or measured-anchor generation.
    GenerationChanged,
    /// The host disabled connection input.
    Disabled,
    /// An embedded control or exclusion overlay took input.
    HostTakeover,
    /// The host explicitly cancelled transient interaction.
    Explicit,
    /// The host cancelled transient interaction at a lifecycle boundary.
    HostCancelled,
    /// The frame cannot safely resolve shared geometry.
    GeometryUnavailable,
    /// The host replaced the board selection.
    SelectionChanged,
    /// The host declined to allocate a new edge identity.
    IdDeclined,
    /// Invalid or over-budget input prevented safe continuation.
    InvalidInput,
}
/// A completed drag outcome, independent of host persistence and history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionEndOutcome {
    /// One exact-base transaction was emitted; it has not been acknowledged.
    Submitted,
    /// The unchanged exact original binding was restored without a transaction.
    NoOp,
    /// The selected target violated the current model or host policy.
    Invalid,
    /// The release had no target port.
    Empty,
}
/// Frozen identity of a begun connection gesture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionGestureInfo {
    /// Exact canonical input context.
    pub base: ContentIdentity,
    /// Actual initiating port; for reconnect this is the unchanged opposite end.
    pub origin: PortEndpoint,
    /// Actual initiating normalized role, including target-start creation.
    pub start: ConnectorEndpointKind,
    /// Planned caller-owned edge identity and deliberate operation.
    pub operation: ConnectionOperation,
}
/// Ordered, nonauthoritative interaction observations. Every Begin gets one terminal event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionEvent {
    /// The armed input crossed the drag threshold with a usable edge identity.
    Begin(ConnectionGestureInfo),
    /// The captured pointer changed its current target/preview.
    Update {
        /// Frozen gesture context.
        gesture: ConnectionGestureInfo,
        /// Exact current target, including a directly hit invalid target.
        target: Option<PortEndpoint>,
        /// Whether the complete target passes current model and host policy.
        valid: bool,
    },
    /// The initiating input released normally.
    End {
        /// Frozen gesture context.
        gesture: ConnectionGestureInfo,
        /// Submission/no-op/refusal outcome, never a save acknowledgement.
        outcome: ConnectionEndOutcome,
    },
    /// A begun gesture was interrupted before any submission.
    Cancel {
        /// Frozen gesture context.
        gesture: ConnectionGestureInfo,
        /// Explicit reason for releasing capture.
        reason: ConnectionCancelReason,
    },
}
/// Current visual status of a disposable connection route.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionPreviewState {
    /// The candidate target passes model and host validation.
    Valid,
    /// A target was hit but failed validation.
    Invalid,
    /// The pointer is not attached to a target.
    Unattached,
    /// A submitted route remains visible until an explicit host outcome.
    Pending(ConnectionSubmissionState),
}
/// Additional route presentation; the canonical edge remains untouched and visible.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionPreview {
    /// Exact normalized source, or no binding at a freely moving source end.
    pub from: Option<PortEndpoint>,
    /// Exact normalized target, or no binding at a freely moving target end.
    pub to: Option<PortEndpoint>,
    /// World-space route derived from the same frame anchors used for hit testing.
    pub route: SmoothStepConnector,
    /// Current validation/submission feedback for host styling.
    pub state: ConnectionPreviewState,
}

pub(crate) struct ConnectionInput<'a> {
    pub session: &'a CanvasSession,
    pub dialect: &'a dyn CanvasDialect,
    pub host: &'a mut dyn CanvasViewHost,
    pub ports: &'a PortFrame,
    pub handles: &'a [VisibleReconnectHandle],
    pub options: Option<&'a ConnectionOptions>,
    pub board_id: Id,
    pub viewport_id: ViewportId,
    pub viewport: Rect,
    pub camera: Camera,
    pub events: &'a [Event],
    pub pointer_pos: Option<Pos2>,
    pub primary_down: bool,
    pub focused: bool,
    pub enabled: bool,
    pub ownership: CanvasInputOwnership,
    pub exclusions: &'a [Rect],
}
#[derive(Default)]
pub(crate) struct ConnectionStep {
    pub transaction: Option<CanvasTransaction>,
    pub events: Vec<ConnectionEvent>,
    pub preview: Option<ConnectionPreview>,
    pub consumed_input: CanvasInputOwnership,
    pub errors: Vec<CanvasError>,
    pub blocking: bool,
}
impl ConnectionStep {
    fn consume(&mut self) {
        self.blocking = true;
        self.consumed_input = CanvasInputOwnership::captured();
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum InputOwner {
    Mouse,
    Touch(TouchDeviceId, TouchId),
}
struct GestureContext {
    base: ContentIdentity,
    board: Id,
    viewport_id: ViewportId,
    viewport: Rect,
    camera: Camera,
    options: ConnectionOptions,
}
enum CaptureKind {
    Create {
        edge_id: Option<String>,
    },
    Reconnect {
        edge_id: String,
        end: ConnectorEndpointKind,
        original: PortEndpoint,
        display: PortEndpoint,
    },
}
struct Capture {
    context: GestureContext,
    owner: InputOwner,
    press: Pos2,
    current: Pos2,
    origin: PortEndpoint,
    start: ConnectorEndpointKind,
    kind: CaptureKind,
    begun: bool,
    target: Option<PortEndpoint>,
    preview: Option<ConnectionPreview>,
}
impl Capture {
    fn info(&self) -> Option<ConnectionGestureInfo> {
        if !self.begun {
            return None;
        }
        let operation = match &self.kind {
            CaptureKind::Create {
                edge_id: Some(edge_id),
            } => ConnectionOperation::Create {
                edge_id: edge_id.clone(),
            },
            CaptureKind::Create { edge_id: None } => return None,
            CaptureKind::Reconnect { edge_id, end, .. } => ConnectionOperation::Reconnect {
                edge_id: edge_id.clone(),
                endpoint: *end,
            },
        };
        Some(ConnectionGestureInfo {
            base: self.context.base,
            origin: self.origin.clone(),
            start: self.start,
            operation,
        })
    }
    fn request(&self, target: &PortEndpoint) -> Option<ConnectionRequest> {
        let info = self.info()?;
        let target = match &self.kind {
            CaptureKind::Reconnect {
                original, display, ..
            } if target == display => original,
            _ => target,
        };
        let (from, to) = match self.start {
            ConnectorEndpointKind::From => (self.origin.clone(), target.clone()),
            ConnectorEndpointKind::To => (target.clone(), self.origin.clone()),
        };
        Some(ConnectionRequest {
            from,
            to,
            start: self.start,
            operation: info.operation,
        })
    }
    fn no_op(&self, request: &ConnectionRequest) -> bool {
        match &self.kind {
            CaptureKind::Reconnect { end, original, .. } => endpoint(request, *end) == original,
            CaptureKind::Create { .. } => false,
        }
    }
}
struct Pending {
    context: GestureContext,
    request: ConnectionRequest,
    original: Option<PortEndpoint>,
    preview: ConnectionPreview,
    receipt: Option<Box<str>>,
    max_bytes: usize,
    state: ConnectionSubmissionState,
    host_rejected: bool,
}
/// Internal state contains at most one captured gesture and one submitted raw receipt.
#[derive(Default)]
pub(crate) struct ConnectionController {
    capture: Option<Capture>,
    pending: Option<Pending>,
    queued_terminal: Option<ConnectionEvent>,
    queued_consumed: bool,
}
impl ConnectionController {
    pub fn has_gesture(&self) -> bool {
        self.capture.is_some()
    }
    pub fn connection_pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn connection_status(&self) -> Option<&ConnectionSubmissionState> {
        self.pending.as_ref().map(|pending| &pending.state)
    }
    pub fn cancel(&mut self, reason: ConnectionCancelReason) {
        if let Some(capture) = self.capture.take() {
            self.queued_consumed = true;
            if let Some(gesture) = capture.info() {
                self.queued_terminal = Some(ConnectionEvent::Cancel { gesture, reason });
            }
        }
    }
    fn drain_terminal(&mut self, output: &mut ConnectionStep) {
        if std::mem::take(&mut self.queued_consumed) {
            output.consume();
        }
        if let Some(event) = self.queued_terminal.take() {
            output.events.push(event);
        }
    }
    pub fn step(&mut self, mut input: ConnectionInput<'_>) -> ConnectionStep {
        let mut output = ConnectionStep::default();
        self.drain_terminal(&mut output);
        if let Some(pending) = &mut self.pending {
            output.consume();
            if input.session.identity() != pending.context.base {
                pending.state = ConnectionSubmissionState::Conflict;
            }
            if pending.context.board == input.board_id
                && pending.context.base.same_session(input.session.identity())
            {
                let mut preview = pending.preview.clone();
                preview.state = ConnectionPreviewState::Pending(pending.state.clone());
                output.preview = Some(preview);
            }
            return output;
        }
        // A public cancellation owns the entire next frame, including a queued release.
        if output.blocking {
            return output;
        }
        let Some(options) = input.options else {
            self.cancel(ConnectionCancelReason::Disabled);
            self.drain_terminal(&mut output);
            return output;
        };
        if let Err(error) = options.validate() {
            self.cancel(ConnectionCancelReason::InvalidInput);
            self.drain_terminal(&mut output);
            output.errors.push(error);
            return output;
        }
        if let Some(reason) = self.context_refusal(&input) {
            self.cancel(reason);
            self.drain_terminal(&mut output);
            return output;
        }
        // Bound event fan-out and repeated hit traversal independently of host input volume.
        if input.events.len() > 4096
            || input
                .events
                .len()
                .checked_mul(input.ports.ports.len().saturating_add(1))
                .is_none_or(|work| work > input.session.limits().max_work_units)
        {
            self.cancel(ConnectionCancelReason::InvalidInput);
            self.drain_terminal(&mut output);
            output.errors.push(CanvasError::WorkLimit);
            return output;
        }
        let touch_batch = input
            .events
            .iter()
            .any(|event| matches!(event, Event::Touch { .. }));
        let suppress_mouse = self
            .capture
            .as_ref()
            .is_some_and(|capture| matches!(capture.owner, InputOwner::Touch(..)))
            || (self.capture.is_none() && touch_batch);
        if self.capture.is_some() {
            output.consume();
        }
        for event in input.events {
            match event {
                Event::Key {
                    key: Key::Escape,
                    pressed: true,
                    ..
                } => {
                    self.cancel(ConnectionCancelReason::Escape);
                    self.drain_terminal(&mut output);
                    // Escape cannot also arm a later press from this same event batch.
                    break;
                }
                Event::WindowFocused(false) => {
                    self.cancel(ConnectionCancelReason::FocusLost);
                    self.drain_terminal(&mut output);
                    break;
                }
                Event::PointerGone => {
                    self.cancel(ConnectionCancelReason::PointerLost);
                    self.drain_terminal(&mut output);
                    if output.blocking {
                        break;
                    }
                }
                Event::Touch {
                    device_id,
                    id,
                    phase,
                    pos,
                    ..
                } => {
                    let owner = InputOwner::Touch(*device_id, *id);
                    match phase {
                        TouchPhase::Start if self.capture.is_none() && !output.blocking => {
                            self.arm(&input, owner, *pos, &mut output)
                        }
                        TouchPhase::Move if self.owned_by(owner) => {
                            self.motion(&mut input, *pos, &mut output)
                        }
                        TouchPhase::End if self.owned_by(owner) => {
                            self.release(&mut input, *pos, &mut output);
                            break;
                        }
                        TouchPhase::Cancel if self.owned_by(owner) => {
                            self.cancel(ConnectionCancelReason::PointerLost);
                            self.drain_terminal(&mut output);
                            break;
                        }
                        _ => {}
                    }
                }
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    ..
                } if !suppress_mouse => {
                    if *pressed && self.capture.is_none() && !output.blocking {
                        self.arm(&input, InputOwner::Mouse, *pos, &mut output);
                    } else if !*pressed && self.owned_by(InputOwner::Mouse) {
                        self.release(&mut input, *pos, &mut output);
                        break;
                    }
                }
                Event::PointerMoved(pos) if !suppress_mouse && self.owned_by(InputOwner::Mouse) => {
                    self.motion(&mut input, *pos, &mut output)
                }
                _ => {}
            }
            if self.capture.is_none() && output.blocking {
                break;
            }
        }
        if self.owned_by(InputOwner::Mouse) && (input.pointer_pos.is_none() || !input.primary_down)
        {
            self.cancel(ConnectionCancelReason::PointerLost);
            self.drain_terminal(&mut output);
        }
        if let Some(capture) = &self.capture {
            output.preview = capture.preview.clone();
        }
        if let Some(pending) = &self.pending {
            output.preview = Some(pending.preview.clone());
        }
        output
    }
    fn owned_by(&self, owner: InputOwner) -> bool {
        self.capture
            .as_ref()
            .is_some_and(|capture| capture.owner == owner)
    }
    fn context_refusal(&self, input: &ConnectionInput<'_>) -> Option<ConnectionCancelReason> {
        if !input.focused {
            return Some(ConnectionCancelReason::FocusLost);
        }
        if !input.enabled {
            return Some(ConnectionCancelReason::Disabled);
        }
        if input.ownership.any() {
            return Some(ConnectionCancelReason::HostTakeover);
        }
        let capture = self.capture.as_ref()?;
        if input
            .exclusions
            .iter()
            .any(|rect| rect.contains(input.pointer_pos.unwrap_or(capture.current)))
        {
            return Some(ConnectionCancelReason::HostTakeover);
        }
        let context = &capture.context;
        if input.session.identity() != context.base {
            Some(ConnectionCancelReason::BaseChanged)
        } else if input.board_id != context.board {
            Some(ConnectionCancelReason::BoardChanged)
        } else if input.viewport_id != context.viewport_id || input.viewport != context.viewport {
            Some(ConnectionCancelReason::ViewportChanged)
        } else if input.camera != context.camera {
            Some(ConnectionCancelReason::CameraChanged)
        } else if input.options != Some(&context.options) {
            Some(ConnectionCancelReason::GenerationChanged)
        } else {
            None
        }
    }
    fn arm(
        &mut self,
        input: &ConnectionInput<'_>,
        owner: InputOwner,
        pos: Pos2,
        output: &mut ConnectionStep,
    ) {
        if !available_position(input, pos) {
            return;
        }
        let Some(options) = input.options else {
            return;
        };
        let (origin, start, kind) = if let Some(handle) = input
            .handles
            .iter()
            .rev()
            .find(|handle| handle.hit_rect.contains(pos))
        {
            let Some(edge) = input.session.document().edge(&handle.edge_id) else {
                return;
            };
            let start = opposite_end(handle.end);
            let origin = match edge.endpoint(start) {
                Ok(origin) => origin,
                Err(error) => {
                    output.errors.push(error);
                    return;
                }
            };
            (
                origin,
                start,
                CaptureKind::Reconnect {
                    edge_id: handle.edge_id.clone(),
                    end: handle.end,
                    original: handle.original.clone(),
                    display: handle.display_endpoint.clone(),
                },
            )
        } else {
            // Creation requires an actual painted hit, not the much larger drop magnet.
            let Some(port) = hit_port(input.ports, pos, 0.0, None) else {
                return;
            };
            if !port.connectable || !port.start {
                return;
            }
            let start = match &port.endpoint.binding {
                PortBinding::Named(key) if key.kind == PortKind::Target => {
                    ConnectorEndpointKind::To
                }
                _ => ConnectorEndpointKind::From,
            };
            (
                port.endpoint.clone(),
                start,
                CaptureKind::Create { edge_id: None },
            )
        };
        self.capture = Some(Capture {
            context: GestureContext {
                base: input.session.identity(),
                board: input.board_id,
                viewport_id: input.viewport_id,
                viewport: input.viewport,
                camera: input.camera,
                options: options.clone(),
            },
            owner,
            press: pos,
            current: pos,
            origin,
            start,
            kind,
            begun: false,
            target: None,
            preview: None,
        });
        output.consume();
    }
    fn motion(&mut self, input: &mut ConnectionInput<'_>, pos: Pos2, output: &mut ConnectionStep) {
        let Some(mut capture) = self.capture.take() else {
            return;
        };
        output.consume();
        if !pos.is_finite() {
            self.capture = Some(capture);
            self.cancel(ConnectionCancelReason::InvalidInput);
            self.drain_terminal(output);
            return;
        }
        if input.exclusions.iter().any(|rect| rect.contains(pos)) {
            self.capture = Some(capture);
            self.cancel(ConnectionCancelReason::HostTakeover);
            self.drain_terminal(output);
            return;
        }
        capture.current = pos;
        if !capture.begun {
            if pos == capture.press
                || pos.distance(capture.press) < capture.context.options.drag_threshold
            {
                self.capture = Some(capture);
                return;
            }
            if let CaptureKind::Create { edge_id } = &mut capture.kind {
                match input
                    .host
                    .allocate_edge_id(capture.context.base, &capture.origin)
                {
                    Ok(Some(id))
                        if !id.trim().is_empty()
                            && id.len() <= input.session.limits().max_id_bytes
                            && input.session.document().edge(&id).is_none() =>
                    {
                        *edge_id = Some(id)
                    }
                    Ok(None) => {
                        self.capture = Some(capture);
                        self.cancel(ConnectionCancelReason::IdDeclined);
                        self.drain_terminal(output);
                        return;
                    }
                    Ok(Some(_)) => {
                        output.errors.push(CanvasError::Invalid(
                            "host must allocate a fresh bounded nonempty edge ID".into(),
                        ));
                        return;
                    }
                    Err(error) => {
                        output.errors.push(error);
                        return;
                    }
                }
            }
            capture.begun = true;
            if let Some(info) = capture.info() {
                output.events.push(ConnectionEvent::Begin(info));
            }
        }
        capture.target = if input.viewport.contains(pos) {
            // A named port may share the unchanged legacy anchor. Returning to the
            // dragged handle must preserve the exact original binding in that case.
            let unchanged = match &capture.kind {
                CaptureKind::Reconnect { edge_id, end, .. } => input
                    .handles
                    .iter()
                    .find(|handle| {
                        handle.edge_id == *edge_id
                            && handle.end == *end
                            && handle.hit_rect.contains(pos)
                    })
                    .map(|handle| handle.display_endpoint.clone()),
                CaptureKind::Create { .. } => None,
            };
            unchanged.or_else(|| {
                hit_port(
                    input.ports,
                    pos,
                    capture.context.options.target_radius,
                    Some(&capture.origin),
                )
                .map(|port| port.endpoint.clone())
            })
        } else {
            None
        };
        let request = capture
            .target
            .as_ref()
            .and_then(|target| capture.request(target));
        let valid = request.as_ref().is_some_and(|request| {
            capture.no_op(request)
                || validate_connection(input.session.document(), request, input.dialect).is_ok()
        });
        let preview = preview_for(input.ports, &capture, request.as_ref(), valid);
        match preview {
            Ok(preview) => capture.preview = Some(preview),
            Err(error) => {
                output.errors.push(error);
                self.capture = Some(capture);
                self.cancel(ConnectionCancelReason::InvalidInput);
                self.drain_terminal(output);
                return;
            }
        }
        if let Some(gesture) = capture.info() {
            output.events.push(ConnectionEvent::Update {
                gesture,
                target: capture.target.clone(),
                valid,
            });
        }
        self.capture = Some(capture);
    }
    fn release(&mut self, input: &mut ConnectionInput<'_>, pos: Pos2, output: &mut ConnectionStep) {
        output.consume();
        if !input.viewport.contains(pos) {
            self.cancel(ConnectionCancelReason::Outside);
            self.drain_terminal(output);
            return;
        }
        self.motion(input, pos, output);
        let Some(capture) = self.capture.take() else {
            return;
        };
        let Some(gesture) = capture.info() else {
            return;
        };
        let Some(request) = capture
            .target
            .as_ref()
            .and_then(|target| capture.request(target))
        else {
            output.events.push(ConnectionEvent::End {
                gesture,
                outcome: ConnectionEndOutcome::Empty,
            });
            return;
        };
        if capture.no_op(&request) {
            output.events.push(ConnectionEvent::End {
                gesture,
                outcome: ConnectionEndOutcome::NoOp,
            });
            return;
        }
        if let Err(error) = validate_connection(input.session.document(), &request, input.dialect) {
            output.errors.push(error.into());
            output.events.push(ConnectionEvent::End {
                gesture,
                outcome: ConnectionEndOutcome::Invalid,
            });
            return;
        }
        let original = match &capture.kind {
            CaptureKind::Reconnect { original, .. } => Some(original.clone()),
            CaptureKind::Create { .. } => None,
        };
        match transaction_for(
            capture.context.base,
            &request,
            original.as_ref(),
            input.session,
        ) {
            Ok(transaction) => {
                if let Some(mut preview) = capture.preview {
                    preview.state =
                        ConnectionPreviewState::Pending(ConnectionSubmissionState::Awaiting);
                    self.pending = Some(Pending {
                        context: capture.context,
                        request,
                        original,
                        preview,
                        receipt: None,
                        max_bytes: input.session.limits().max_bytes,
                        state: ConnectionSubmissionState::Awaiting,
                        host_rejected: false,
                    });
                    output.transaction = Some(transaction);
                    output.events.push(ConnectionEvent::End {
                        gesture,
                        outcome: ConnectionEndOutcome::Submitted,
                    });
                } else {
                    output.errors.push(CanvasError::Invalid(
                        "connection route is unavailable".into(),
                    ));
                    output.events.push(ConnectionEvent::End {
                        gesture,
                        outcome: ConnectionEndOutcome::Invalid,
                    });
                }
            }
            Err(error) => {
                output.errors.push(error);
                output.events.push(ConnectionEvent::End {
                    gesture,
                    outcome: ConnectionEndOutcome::Invalid,
                });
            }
        }
    }
    pub fn bind_connection_prepared(&mut self, prepared: &PreparedChange) -> Result<()> {
        let pending = self.pending.as_mut().ok_or_else(no_pending)?;
        if pending.state != ConnectionSubmissionState::Awaiting
            || prepared.base() != pending.context.base
            || !prepared.changed()
        {
            return Err(CanvasError::StaleBase);
        }
        let edge = prepared
            .document()
            .edge(edge_id(&pending.request))
            .ok_or(CanvasError::StaleBase)?;
        if edge.endpoint(ConnectorEndpointKind::From)? != pending.request.from
            || edge.endpoint(ConnectorEndpointKind::To)? != pending.request.to
        {
            return Err(CanvasError::StaleBase);
        }
        let raw = prepared.document().raw_json();
        if raw.len() > pending.max_bytes {
            return Err(CanvasError::Limit("connection prepared receipt bytes"));
        }
        if pending
            .receipt
            .as_deref()
            .is_some_and(|receipt| receipt != raw)
        {
            return Err(CanvasError::StaleBase);
        }
        if pending.receipt.is_none() {
            pending.receipt = Some(raw.into());
        }
        Ok(())
    }
    pub fn acknowledge_connection(&mut self, session: &CanvasSession) -> Result<()> {
        let pending = self.pending.as_ref().ok_or_else(no_pending)?;
        if !pending.context.base.same_session(session.identity())
            || pending.context.base.revision().checked_add(1) != Some(session.identity().revision())
            || pending.receipt.as_deref() != Some(session.document().raw_json())
        {
            return Err(CanvasError::StaleBase);
        }
        self.pending = None;
        Ok(())
    }
    pub fn reject_connection(&mut self, reason: &str) -> Result<()> {
        if reason.len() > 4096 {
            return Err(CanvasError::Limit("connection rejection message bytes"));
        }
        let pending = self.pending.as_mut().ok_or_else(no_pending)?;
        pending.state = ConnectionSubmissionState::Rejected(reason.into());
        pending.host_rejected = true;
        Ok(())
    }
    pub fn retry_connection(
        &mut self,
        session: &CanvasSession,
        dialect: &dyn CanvasDialect,
    ) -> Result<CanvasTransaction> {
        let pending = self.pending.as_mut().ok_or_else(no_pending)?;
        if !pending.host_rejected
            || !matches!(pending.state, ConnectionSubmissionState::Rejected(_))
        {
            return Err(CanvasError::Invalid(
                "reject the submitted connection before retrying".into(),
            ));
        }
        if pending.context.base != session.identity() {
            pending.state = ConnectionSubmissionState::Conflict;
            return Err(CanvasError::StaleBase);
        }
        validate_connection(session.document(), &pending.request, dialect)?;
        let transaction = transaction_for(
            pending.context.base,
            &pending.request,
            pending.original.as_ref(),
            session,
        )?;
        pending.state = ConnectionSubmissionState::Awaiting;
        pending.host_rejected = false;
        pending.preview.state =
            ConnectionPreviewState::Pending(ConnectionSubmissionState::Awaiting);
        Ok(transaction)
    }
    pub fn dismiss_connection(&mut self) -> Result<()> {
        if !self.pending.as_ref().ok_or_else(no_pending)?.host_rejected {
            return Err(CanvasError::Invalid(
                "host must reject or cancel its write before dismissing the connection".into(),
            ));
        }
        self.pending = None;
        Ok(())
    }
}
fn available_position(input: &ConnectionInput<'_>, pos: Pos2) -> bool {
    pos.is_finite()
        && input.viewport.contains(pos)
        && !input.exclusions.iter().any(|rect| rect.contains(pos))
}
fn opposite_end(end: ConnectorEndpointKind) -> ConnectorEndpointKind {
    match end {
        ConnectorEndpointKind::From => ConnectorEndpointKind::To,
        ConnectorEndpointKind::To => ConnectorEndpointKind::From,
    }
}
fn opposite_side(side: Side) -> Side {
    match side {
        Side::Top => Side::Bottom,
        Side::Right => Side::Left,
        Side::Bottom => Side::Top,
        Side::Left => Side::Right,
    }
}
fn endpoint(request: &ConnectionRequest, end: ConnectorEndpointKind) -> &PortEndpoint {
    match end {
        ConnectorEndpointKind::From => &request.from,
        ConnectorEndpointKind::To => &request.to,
    }
}
fn edge_id(request: &ConnectionRequest) -> &str {
    match &request.operation {
        ConnectionOperation::Create { edge_id }
        | ConnectionOperation::Reconnect { edge_id, .. }
        | ConnectionOperation::ReconnectBoth { edge_id } => edge_id,
    }
}
fn preview_for(
    ports: &PortFrame,
    capture: &Capture,
    request: Option<&ConnectionRequest>,
    valid: bool,
) -> Result<ConnectionPreview> {
    if let Some(request) = request {
        return Ok(ConnectionPreview {
            from: Some(request.from.clone()),
            to: Some(request.to.clone()),
            route: ports.route(&request.from, &request.to)?,
            state: if valid {
                ConnectionPreviewState::Valid
            } else {
                ConnectionPreviewState::Invalid
            },
        });
    }
    let origin = ports.resolve(&capture.origin)?;
    let pointer = ports.world_at(capture.current);
    let (from, to, route) = match capture.start {
        ConnectorEndpointKind::From => (
            Some(capture.origin.clone()),
            None,
            SmoothStepConnector::between(
                origin.world,
                origin.side,
                pointer,
                opposite_side(origin.side),
            )?,
        ),
        ConnectorEndpointKind::To => (
            None,
            Some(capture.origin.clone()),
            SmoothStepConnector::between(
                pointer,
                opposite_side(origin.side),
                origin.world,
                origin.side,
            )?,
        ),
    };
    Ok(ConnectionPreview {
        from,
        to,
        route,
        state: ConnectionPreviewState::Unattached,
    })
}
fn transaction_for(
    base: ContentIdentity,
    request: &ConnectionRequest,
    original: Option<&PortEndpoint>,
    session: &CanvasSession,
) -> Result<CanvasTransaction> {
    let command = match &request.operation {
        ConnectionOperation::Create { edge_id } => CanvasCommand::ConnectPorts {
            edge: EdgeRecord::connect_ports(
                edge_id.clone(),
                request.from.clone(),
                request.to.clone(),
                session.limits(),
            )?,
            index: session.document().edges().len(),
            start: request.start,
        },
        ConnectionOperation::Reconnect {
            edge_id,
            endpoint: end,
        } => CanvasCommand::ReconnectPort {
            edge_id: edge_id.clone(),
            endpoint: *end,
            expected: original.cloned().ok_or_else(|| {
                CanvasError::Invalid("reconnection has no original endpoint".into())
            })?,
            replacement: endpoint(request, *end).clone(),
        },
        ConnectionOperation::ReconnectBoth { .. } => {
            return Err(CanvasError::Invalid(
                "a handle drag changes one endpoint".into(),
            ));
        }
    };
    Ok(CanvasTransaction::new(base, vec![command]))
}
fn no_pending() -> CanvasError {
    CanvasError::Invalid("no connection is awaiting host acknowledgement".into())
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod tests;
