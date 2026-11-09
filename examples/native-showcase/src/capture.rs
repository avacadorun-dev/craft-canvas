//! Bounded actual-frame readback and explicitly scripted native-event evidence.
use craft_canvas_egui::{
    CanvasView, ConnectionCancelReason, ConnectionEndOutcome, ConnectionEvent,
    ConnectionSubmissionState,
};
use craft_canvas_model::{CanvasSession, ConnectorEndpointKind, PortBinding, PortEndpoint};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Vec2};
use image::ImageEncoder;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

type CaptureResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const DEADLINE: Duration = Duration::from_secs(45);
const RETRY: Duration = Duration::from_millis(600);
const INPUT_TRACE_LIMIT: usize = 512;
const FOCUS_DEADLINE: Duration = Duration::from_secs(10);
const SCRIPTED_ZOOM: f32 = 1.125;

#[derive(Clone, Copy)]
enum InputOrigin {
    Upstream,
    Scripted,
}
impl InputOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::Upstream => "upstream RawInput",
            Self::Scripted => "scripted RawInput",
        }
    }
    fn delivery(self, event: &Event) -> &'static str {
        match self {
            Self::Upstream if native_lifecycle_event(event) => "retained-native-lifecycle",
            Self::Upstream => "suppressed",
            Self::Scripted => "injected",
        }
    }
}
fn native_lifecycle_event(event: &Event) -> bool {
    matches!(event, Event::WindowFocused(_) | Event::Screenshot { .. })
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Counters {
    pub begin: usize,
    pub submitted: usize,
    pub no_op: usize,
    pub invalid: usize,
    pub empty: usize,
    pub cancelled: usize,
    pub escape: usize,
    pub events: usize,
    pub intents: usize,
    pub submissions: usize,
    pub acknowledgements: usize,
}
impl Counters {
    pub fn observe(&mut self, event: &ConnectionEvent) {
        self.events += 1;
        match event {
            ConnectionEvent::Begin(_) => self.begin += 1,
            ConnectionEvent::End { outcome, .. } => match outcome {
                ConnectionEndOutcome::Submitted => self.submitted += 1,
                ConnectionEndOutcome::NoOp => self.no_op += 1,
                ConnectionEndOutcome::Invalid => self.invalid += 1,
                ConnectionEndOutcome::Empty => self.empty += 1,
            },
            ConnectionEvent::Cancel { reason, .. } => {
                self.cancelled += 1;
                self.escape += usize::from(*reason == ConnectionCancelReason::Escape);
            }
            ConnectionEvent::Update { .. } => {}
        }
    }
    fn json(self) -> Value {
        json!({"begin":self.begin,"submitted":self.submitted,"no_op":self.no_op,
            "invalid":self.invalid,"empty":self.empty,"cancelled":self.cancelled,"escape":self.escape,
            "events":self.events,"view_transaction_intents":self.intents,"host_submissions":self.submissions,
            "host_acknowledgements":self.acknowledgements})
    }
}

pub(crate) fn endpoint_json(endpoint: &PortEndpoint) -> Value {
    match &endpoint.binding {
        PortBinding::Named(key) => {
            json!({"node":endpoint.node_id,"port":key.id,"kind":key.kind.as_str()})
        }
        PortBinding::Legacy(side) => {
            json!({"node":endpoint.node_id,"legacy_side":side.map(|side|side.as_str())})
        }
    }
}
pub(crate) fn port_target(endpoint: &PortEndpoint) -> String {
    match &endpoint.binding {
        PortBinding::Named(key) => {
            format!("port:{}:{}:{}", endpoint.node_id, key.kind.as_str(), key.id)
        }
        PortBinding::Legacy(side) => format!(
            "port:{}:legacy:{}",
            endpoint.node_id,
            side.map_or("omitted", |side| side.as_str())
        ),
    }
}
pub(crate) fn event_json(event: &ConnectionEvent, frame: u64) -> Value {
    let (gesture, event_type, detail) = match event {
        ConnectionEvent::Begin(gesture) => (gesture, "begin", Value::Null),
        ConnectionEvent::Update {
            gesture,
            target,
            valid,
        } => (
            gesture,
            "update",
            json!({"target":target.as_ref().map(endpoint_json),"valid":valid}),
        ),
        ConnectionEvent::End { gesture, outcome } => {
            (gesture, "end", json!({"outcome":format!("{outcome:?}")}))
        }
        ConnectionEvent::Cancel { gesture, reason } => {
            (gesture, "cancel", json!({"reason":format!("{reason:?}")}))
        }
    };
    json!({"frame":frame,"event":event_type,"detail":detail,"base_revision":gesture.base.revision(),
        "origin":endpoint_json(&gesture.origin),"start":format!("{:?}",gesture.start),"operation":format!("{:?}",gesture.operation)})
}

pub(crate) struct FrameSnapshot {
    pub targets: BTreeMap<String, Pos2>,
    pub geometry: Value,
    pub errors: Vec<String>,
    pub frame: u64,
}
pub(crate) struct Observation {
    pub targets: BTreeMap<String, Pos2>,
    pub state: Value,
    frame: u64,
    raw: String,
    signature: String,
    edge_count: usize,
    undo: usize,
    pending: bool,
    rejected: bool,
    awaiting: bool,
    counters: Counters,
}
impl Observation {
    pub fn new(
        session: &CanvasSession,
        view: &CanvasView,
        frame: FrameSnapshot,
        counters: &Counters,
        events: &[Value],
        message: &str,
    ) -> Self {
        let document = session.document();
        let raw = document.raw_json().to_owned();
        let history = session.history_usage();
        let pending = view.connection_pending();
        let rejected = matches!(
            view.connection_status(),
            Some(ConnectionSubmissionState::Rejected(_))
        );
        let awaiting = matches!(
            view.connection_status(),
            Some(ConnectionSubmissionState::Awaiting)
        );
        let new_edge = document.edge("showcase-edge-1").map(|edge| {
            json!({
                "id":edge.id(),
                "from":edge.endpoint(ConnectorEndpointKind::From).ok().as_ref().map(endpoint_json),
                "to":edge.endpoint(ConnectorEndpointKind::To).ok().as_ref().map(endpoint_json),
            })
        });
        let target_json: BTreeMap<_, _> = frame
            .targets
            .iter()
            .map(|(key, point)| (key.clone(), [point.x, point.y]))
            .collect();
        let signature = json!({"raw":raw,"geometry":frame.geometry,"targets":target_json,"pending":pending,"rejected":rejected}).to_string();
        let state = json!({"frame":frame.frame,"revision":session.identity().revision(),"canonical_json":raw,
            "node_count":document.nodes().len(),"edge_count":document.edges().len(),
            "legacy_raw":document.edge("legacy").map(|edge|edge.raw_json()),"new_edge":new_edge,
            "history":{"undo":history.undo_steps,"redo":history.redo_steps},
            "pending":pending,"submission_state":format!("{:?}",view.connection_status()),"has_gesture":view.has_gesture(),
            "counters":counters.json(),"events":events,"geometry":frame.geometry,"targets":target_json,
            "engine_errors":frame.errors,"host_message":message});
        Self {
            targets: frame.targets,
            state,
            frame: frame.frame,
            raw,
            signature,
            edge_count: document.edges().len(),
            undo: history.undo_steps,
            pending,
            rejected,
            awaiting,
            counters: *counters,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Check {
    Zoomed,
    HeldInitial,
    Rejected,
    Created,
    HeldReconnect,
    Reconnected,
    Restored,
    NoOp,
    Cancelled,
}
#[derive(Clone, Copy, Debug)]
enum Step {
    Snapshot(&'static str),
    Zoom,
    Click(&'static str),
    Drag(&'static str, &'static str),
    Cancel(&'static str),
    Check(Check),
}
fn steps() -> Vec<Step> {
    vec![
        Step::Snapshot("baseline"),
        Step::Zoom,
        Step::Check(Check::Zoomed),
        Step::Drag("port:a:source:main", "port:b:target:input"),
        Step::Check(Check::HeldInitial),
        Step::Click("button:reject"),
        Step::Check(Check::Rejected),
        Step::Snapshot("rejected"),
        Step::Click("button:retry"),
        Step::Check(Check::HeldInitial),
        Step::Click("button:apply"),
        Step::Check(Check::Created),
        Step::Snapshot("connected"),
        Step::Click("edge:showcase-edge-1"),
        Step::Drag("reconnect:showcase-edge-1:to", "port:b:target:alternative"),
        Step::Check(Check::HeldReconnect),
        Step::Click("button:apply"),
        Step::Check(Check::Reconnected),
        Step::Snapshot("reconnected"),
        Step::Click("button:undo"),
        Step::Check(Check::Created),
        Step::Click("button:undo"),
        Step::Check(Check::Restored),
        Step::Click("edge:legacy"),
        Step::Drag("reconnect:legacy:from", "port:a:legacy:top"),
        Step::Check(Check::NoOp),
        Step::Cancel("port:a:source:alternative"),
        Step::Check(Check::Cancelled),
        Step::Snapshot("complete"),
    ]
}

struct Readback {
    name: &'static str,
    signature: String,
    requested_frame: u64,
    sent: Instant,
    attempts: u8,
}
pub(crate) struct Capture {
    directory: PathBuf,
    source_sha: String,
    initial: String,
    created: Option<String>,
    started: Instant,
    steps: Vec<Step>,
    at: usize,
    phase: u8,
    flight: Option<(Pos2, Pos2)>,
    last_injected: u64,
    stable_signature: String,
    stable_frames: u8,
    readback: Option<Readback>,
    observations: Vec<Value>,
    captures: Vec<Value>,
    input_trace: Vec<Value>,
    input_trace_dropped: usize,
    upstream_suppressed_counts: BTreeMap<&'static str, usize>,
    waiting_for_focus: bool,
    focus_requested: bool,
    input_failure: Option<&'static str>,
    done: bool,
}
impl Capture {
    pub fn new(directory: PathBuf, source_sha: &str, initial: &str) -> CaptureResult<Self> {
        if !directory.is_absolute() {
            return Err("--capture-dir must be an absolute new directory".into());
        }
        std::fs::create_dir(&directory)?;
        Ok(Self {
            directory,
            source_sha: source_sha.into(),
            initial: initial.into(),
            created: None,
            started: Instant::now(),
            steps: steps(),
            at: 0,
            phase: 0,
            flight: None,
            last_injected: 0,
            stable_signature: String::new(),
            stable_frames: 0,
            readback: None,
            observations: vec![],
            captures: vec![],
            input_trace: vec![],
            input_trace_dropped: 0,
            upstream_suppressed_counts: BTreeMap::new(),
            waiting_for_focus: true,
            focus_requested: false,
            input_failure: None,
            done: false,
        })
    }
    pub fn inject(&mut self, input: &mut RawInput, observation: Option<&Observation>) {
        // Only this opt-in fixture owns its interaction stream. Keep native focus,
        // viewport/time/scale context and screenshot lifecycle exactly as received.
        // Trace before filtering; even failure/wait/close frames remain isolated.
        let after_frame = observation.map_or(0, |observation| observation.frame);
        self.trace_input(
            InputOrigin::Upstream,
            after_frame,
            input.focused,
            &input.events,
        );
        input.events.retain(|event| {
            if native_lifecycle_event(event) {
                true
            } else {
                self.count_suppressed(input_event_kind(event), 1);
                false
            }
        });
        self.trace_files(
            after_frame,
            input.focused,
            "hovered-files-redacted",
            input.hovered_files.len(),
        );
        self.trace_files(
            after_frame,
            input.focused,
            "dropped-files-redacted",
            input.dropped_files.len(),
        );
        input.hovered_files.clear();
        input.dropped_files.clear();
        if self.done || self.input_failure.is_some() {
            return;
        }
        // A background macOS launch may not receive native focus immediately.
        // Never invent focus or advance a scripted gesture while it is absent.
        self.waiting_for_focus = !input.focused
            || input
                .events
                .iter()
                .any(|event| matches!(event, Event::WindowFocused(false)));
        if self.waiting_for_focus {
            if self.last_injected > 0 {
                self.input_failure = Some(
                    "Native focus was lost after scripted input began; rerun the isolated verification without competing input",
                );
            }
            return;
        }
        let Some(observation) = observation else {
            return;
        };
        if self.done || self.last_injected == observation.frame {
            return;
        }
        let Some(step) = self.steps.get(self.at).copied() else {
            return;
        };
        let points = match step {
            Step::Zoom => {
                // The initial synthetic camera center is the empty gap between cards.
                // Read its viewport position rather than using screen/DPI constants.
                let viewport = &observation.state["geometry"]["viewport"];
                viewport[0]
                    .as_f64()
                    .zip(viewport[2].as_f64())
                    .zip(viewport[1].as_f64().zip(viewport[3].as_f64()))
                    .map(|((left, right), (top, bottom))| {
                        let point =
                            Pos2::new(((left + right) / 2.0) as f32, ((top + bottom) / 2.0) as f32);
                        (point, point)
                    })
                    .filter(|(point, _)| point.is_finite())
            }
            Step::Click(target) | Step::Cancel(target) => observation
                .targets
                .get(target)
                .map(|point| (*point, *point)),
            Step::Drag(from, to) => observation
                .targets
                .get(from)
                .zip(observation.targets.get(to))
                .map(|(from, to)| (*from, *to)),
            Step::Snapshot(_) | Step::Check(_) => return,
        };
        if self.flight.is_none() {
            self.flight = points;
        }
        let Some((from, to)) = self.flight else {
            return;
        };
        let midpoint = if from.distance(to) < 10.0 {
            from + Vec2::new(45.0, -35.0)
        } else {
            from.lerp(to, 0.5)
        };
        let events = match (step, self.phase) {
            (Step::Zoom, 0) => vec![Event::PointerMoved(from), Event::Zoom(SCRIPTED_ZOOM)],
            (_, 0) => pointer(from, true),
            (Step::Click(_), _) => pointer(from, false),
            (Step::Drag(..), 1) => vec![Event::PointerMoved(midpoint)],
            (Step::Drag(..), 2) => vec![Event::PointerMoved(to)],
            (Step::Drag(..), _) => pointer(to, false),
            (Step::Cancel(_), 1) => vec![Event::PointerMoved(from + Vec2::new(45.0, 0.0))],
            (Step::Cancel(_), _) => {
                let mut events = vec![Event::Key {
                    key: Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }];
                events.extend(pointer(from + Vec2::new(45.0, 0.0), false));
                events
            }
            _ => return,
        };
        self.trace_input(
            InputOrigin::Scripted,
            observation.frame,
            input.focused,
            &events,
        );
        input.events.extend(events);
        self.last_injected = observation.frame;
        self.phase += 1;
        let complete = match step {
            Step::Zoom => self.phase >= 1,
            Step::Click(_) => self.phase >= 2,
            Step::Drag(..) => self.phase >= 4,
            Step::Cancel(_) => self.phase >= 3,
            _ => false,
        };
        if complete {
            self.advance();
        }
    }
    fn trace_input(
        &mut self,
        origin: InputOrigin,
        after_frame: u64,
        focused: bool,
        events: &[Event],
    ) {
        for event in events {
            let entry = json!({"kind":"capture-input","provenance":origin.label(),
                "delivery":origin.delivery(event),"after_frame":after_frame,"step":self.at,
                "phase":self.phase,"focused":focused,"event":input_event_json(event)});
            self.retain_trace(entry);
        }
    }
    fn trace_files(&mut self, after_frame: u64, focused: bool, kind: &'static str, count: usize) {
        if count == 0 {
            return;
        }
        self.count_suppressed(kind, count);
        self.retain_trace(
            json!({"kind":"capture-input","provenance":"upstream RawInput",
            "delivery":"suppressed","after_frame":after_frame,"step":self.at,
            "phase":self.phase,"focused":focused,"event":{"kind":kind,"count":count}}),
        );
    }
    fn count_suppressed(&mut self, kind: &'static str, count: usize) {
        // The exhaustive event classifier and two file classes bound the map keys.
        let total = self.upstream_suppressed_counts.entry(kind).or_default();
        *total = total.saturating_add(count);
    }
    fn retain_trace(&mut self, entry: Value) {
        if self.input_trace.len() == INPUT_TRACE_LIMIT {
            self.input_trace_dropped = self.input_trace_dropped.saturating_add(1);
            return;
        }
        println!("{entry}");
        self.input_trace.push(entry);
    }
    fn advance(&mut self) {
        self.at += 1;
        self.phase = 0;
        self.flight = None;
        self.stable_frames = 0;
        self.stable_signature.clear();
    }
    pub fn observe(&mut self, ctx: &egui::Context, observation: &Observation) {
        if self.done {
            return;
        }
        if let Err(error) = self.observe_inner(ctx, observation) {
            self.finish(ctx, observation, Some(error.to_string()));
        }
    }
    fn observe_inner(
        &mut self,
        ctx: &egui::Context,
        observation: &Observation,
    ) -> CaptureResult<()> {
        if let Some(failure) = self.input_failure {
            return Err(failure.into());
        }
        if self.waiting_for_focus {
            if self.started.elapsed() > FOCUS_DEADLINE {
                return Err(
                    "Verification window did not receive native focus; no scripted input was sent"
                        .into(),
                );
            }
            if !self.focus_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                self.focus_requested = true;
            }
            return Ok(());
        }
        if self.started.elapsed() > DEADLINE {
            return Err(format!(
                "Capture deadline at step {}: {:?}",
                self.at,
                self.steps.get(self.at)
            )
            .into());
        }
        if observation.frame > 2000 || observation.counters.events > 256 {
            return Err("Capture frame/event bound exceeded".into());
        }
        if observation.state["engine_errors"]
            .as_array()
            .is_some_and(|errors| !errors.is_empty())
        {
            return Err("The actual native frame reported an engine error".into());
        }
        if let Some(readback) = &mut self.readback {
            if observation.signature != readback.signature {
                return Err("Frame changed during pending native screenshot readback".into());
            }
            let image = ctx.input(|input| {
                input.events.iter().find_map(|event| {
                    if let Event::Screenshot {
                        viewport_id,
                        user_data,
                        image,
                    } = event
                    {
                        (*viewport_id == egui::ViewportId::ROOT
                            && user_data
                                .data
                                .as_ref()
                                .and_then(|data| data.downcast_ref::<String>())
                                .is_some_and(|name| name == readback.name))
                        .then(|| image.clone())
                    } else {
                        None
                    }
                })
            });
            if let Some(image) = image {
                if image.width() > 4096
                    || image.height() > 4096
                    || image.width() == 0
                    || image.height() == 0
                {
                    return Err("Native screenshot dimensions exceed the capture bound".into());
                }
                let filename = format!("{}.png", readback.name);
                let file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(self.directory.join(&filename))?;
                image::codecs::png::PngEncoder::new(&file).write_image(
                    image.as_raw(),
                    u32::try_from(image.width())?,
                    u32::try_from(image.height())?,
                    image::ExtendedColorType::Rgba8,
                )?;
                file.sync_all()?;
                self.captures.push(json!({"name":readback.name,"file":filename,"requested_frame":readback.requested_frame,"readback_frame":observation.frame,"width":image.width(),"height":image.height(),"observed_state":observation.state}));
                self.readback = None;
                self.advance();
            } else if readback.sent.elapsed() >= RETRY {
                if readback.attempts >= 5 {
                    return Err("Native WGPU screenshot readback did not arrive".into());
                }
                readback.sent = Instant::now();
                readback.attempts += 1;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    readback.name.to_owned(),
                )));
            }
            return Ok(());
        }
        let Some(step) = self.steps.get(self.at).copied() else {
            let count = observation.counters;
            if observation.raw != self.initial
                || observation.undo != 0
                || observation.pending
                || count.begin != 4
                || count.submitted != 2
                || count.no_op != 1
                || count.cancelled != 1
                || count.escape != 1
                || count.invalid != 0
                || count.empty != 0
                || count.intents != 2
                || count.submissions != 3
                || count.acknowledgements != 2
            {
                return Err(
                    "Final canonical/history/lifecycle contract did not match the scripted journey"
                        .into(),
                );
            }
            self.finish(ctx, observation, None);
            return Ok(());
        };
        match step {
            Step::Snapshot(name) => {
                if observation.signature == self.stable_signature {
                    self.stable_frames = self.stable_frames.saturating_add(1);
                } else {
                    self.stable_signature = observation.signature.clone();
                    self.stable_frames = 1;
                }
                if self.stable_frames >= 3 {
                    self.readback = Some(Readback {
                        name,
                        signature: observation.signature.clone(),
                        requested_frame: observation.frame,
                        sent: Instant::now(),
                        attempts: 1,
                    });
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name.to_owned(),
                    )));
                }
            }
            Step::Check(check) if self.check(check, observation) => {
                self.observations
                    .push(json!({"check":format!("{check:?}"),"observed_state":observation.state}));
                if matches!(check, Check::Created) && self.created.is_none() {
                    self.created = Some(observation.raw.clone());
                }
                self.advance();
            }
            _ => {}
        }
        Ok(())
    }
    fn check(&self, check: Check, observation: &Observation) -> bool {
        let initial =
            observation.raw == self.initial && observation.undo == 0 && observation.edge_count == 1;
        let restored = initial && !observation.pending;
        let connected = !observation.pending
            && observation.edge_count == 2
            && observation.state["new_edge"]["from"]["node"] == "a"
            && observation.state["new_edge"]["from"]["port"] == "main"
            && observation.state["new_edge"]["to"]["node"] == "b";
        match check {
            Check::Zoomed => {
                let Some(baseline) = self.captures.first() else {
                    return false;
                };
                let before = &baseline["observed_state"];
                let zoom_changed = before["geometry"]["camera"]["zoom"]
                    .as_f64()
                    .zip(observation.state["geometry"]["camera"]["zoom"].as_f64())
                    .is_some_and(|(before, after)| {
                        (after - before * f64::from(SCRIPTED_ZOOM)).abs() < 1e-6
                    });
                restored
                    && observation.state["revision"] == 0
                    && observation.state["history"]["redo"] == 0
                    && !observation.state["has_gesture"].as_bool().unwrap_or(true)
                    && observation.counters.intents == 0
                    && zoom_changed
                    && ["port:a:source:main", "port:b:target:input"]
                        .iter()
                        .all(|target| {
                            observation.targets.contains_key(*target)
                                && observation.state["targets"][*target]
                                    != before["targets"][*target]
                        })
            }
            Check::HeldInitial => initial && observation.pending && observation.awaiting,
            Check::Rejected => initial && observation.pending && observation.rejected,
            Check::Created => {
                connected
                    && observation.undo == 1
                    && observation.state["new_edge"]["to"]["port"] == "input"
            }
            Check::HeldReconnect => {
                observation.pending
                    && observation.awaiting
                    && observation.undo == 1
                    && self.created.as_ref() == Some(&observation.raw)
            }
            Check::Reconnected => {
                connected
                    && observation.undo == 2
                    && observation.state["new_edge"]["to"]["port"] == "alternative"
            }
            Check::Restored => restored,
            Check::NoOp => restored && observation.counters.no_op == 1,
            Check::Cancelled => restored && observation.counters.escape == 1,
        }
    }
    fn finish(&mut self, ctx: &egui::Context, observation: &Observation, failure: Option<String>) {
        self.done = true;
        let evidence = json!({"schema_version":1,"scenario":"connections","status":if failure.is_none(){"passed"}else{"failed"},
            "failure":failure,"source_sha":self.source_sha,"compiled_source_sha":option_env!("CRAFT_CANVAS_SOURCE_SHA"),
            "evidence_kind":"scripted RawInput through compiled native widgets with WGPU readback",
            "zoom_evidence":"Injected native Zoom event followed by newly measured painted-port drags; not OS pinch/scroll acceptance",
            "os_input_acceptance":false,"physical_touchscreen_acceptance":false,"production_acceptance":false,
            "initial_canonical_json":self.initial,"elapsed_seconds":self.started.elapsed().as_secs_f64(),
            "input_policy":{"mode":"isolated-scripted-fixture",
                "native_events_delivered":["window-focus","screenshot-readback"],
                "native_context":"Unmodified focus, viewport, geometry, time, scale and theme",
                "upstream_interaction_events":"Suppressed before scripted events; all phases remain isolated",
                "file_input":"Hovered and dropped files suppressed; counts only, no paths or contents",
                "suppression_counts":"Saturating totals by fixed class, including events omitted from the bounded trace"},
            "upstream_suppressed_counts":self.upstream_suppressed_counts,
            "input_trace":self.input_trace,"input_trace_dropped":self.input_trace_dropped,
            "completed_steps":self.at,"total_steps":self.steps.len(),"captures":self.captures,"checks":self.observations,"final":observation.state});
        let result = (|| -> CaptureResult<()> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.directory.join("evidence.json"))?;
            serde_json::to_writer_pretty(&mut file, &evidence)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("Cannot retain capture evidence: {error}");
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

// Never format a whole Event: text, paste, IME and accessibility payloads may
// contain private input. Character keys are also redacted, even without Text.
fn input_event_kind(event: &Event) -> &'static str {
    match event {
        Event::PointerMoved(_) => "pointer-moved",
        Event::PointerButton { .. } => "pointer-button",
        Event::Key { .. } => "key",
        Event::WindowFocused(_) => "window-focus",
        Event::Touch { .. } => "touch",
        Event::PointerGone => "pointer-gone",
        Event::Text(_) => "text-redacted",
        Event::Paste(_) => "paste-redacted",
        Event::Ime(_) => "ime-redacted",
        Event::AccessKitActionRequest(_) => "accessibility-redacted",
        Event::Screenshot { .. } => "screenshot-readback",
        Event::Copy => "copy",
        Event::Cut => "cut",
        Event::ModifiersChanged(_) => "modifiers-changed",
        Event::MouseMoved(_) => "mouse-moved",
        Event::MouseWheel { .. } => "mouse-wheel",
        Event::Zoom(_) => "zoom",
        Event::Rotate(_) => "rotate",
    }
}
pub(crate) fn input_event_json(event: &Event) -> Value {
    let kind = input_event_kind(event);
    match event {
        Event::PointerMoved(pos) => json!({"kind":kind,"position":[pos.x,pos.y]}),
        Event::PointerButton {
            pos,
            button,
            pressed,
            ..
        } => json!({"kind":kind,
            "position":[pos.x,pos.y],"button":format!("{button:?}"),"pressed":pressed}),
        Event::Key {
            key,
            pressed,
            repeat,
            ..
        } => {
            let control = match key {
                Key::Escape
                | Key::Tab
                | Key::Enter
                | Key::Backspace
                | Key::Delete
                | Key::ArrowDown
                | Key::ArrowLeft
                | Key::ArrowRight
                | Key::ArrowUp
                | Key::Home
                | Key::End
                | Key::PageUp
                | Key::PageDown => format!("{key:?}"),
                _ => "redacted".into(),
            };
            json!({"kind":kind,"control":control,"pressed":pressed,"repeat":repeat})
        }
        Event::WindowFocused(focused) => json!({"kind":kind,"focused":focused}),
        Event::Touch { phase, pos, .. } => {
            json!({"kind":kind,"phase":format!("{phase:?}"),"position":[pos.x,pos.y]})
        }
        _ => json!({"kind":kind}),
    }
}

fn pointer(point: Pos2, pressed: bool) -> Vec<Event> {
    vec![
        Event::PointerMoved(point),
        Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    fn focus_fixture() -> CaptureResult<(Capture, Observation)> {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "craft-canvas-capture-focus-{}-{stamp}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut capture = Capture::new(directory, "focus-regression", crate::FIXTURE)?;
        capture.steps = vec![Step::Drag("from", "to")];
        let app = crate::Showcase::new("focus-regression".into(), None)?;
        let observation = Observation::new(
            &app.session,
            &app.view,
            FrameSnapshot {
                targets: BTreeMap::from([
                    ("from".into(), Pos2::new(10.0, 10.0)),
                    ("to".into(), Pos2::new(100.0, 10.0)),
                ]),
                geometry: Value::Null,
                errors: vec![],
                frame: 1,
            },
            &Counters::default(),
            &[],
            "",
        );
        Ok((capture, observation))
    }

    #[test]
    fn injected_zoom_waits_for_the_actual_showcase_camera_and_new_port_positions()
    -> CaptureResult<()> {
        let (mut capture, _) = focus_fixture()?;
        capture.steps = vec![Step::Zoom, Step::Check(Check::Zoomed)];
        let ctx = egui::Context::default();
        ctx.all_styles_mut(|style| style.animation_time = 0.0);
        ctx.options_mut(|options| options.max_passes = std::num::NonZeroUsize::MIN);
        let mut app = crate::Showcase::new("zoom-regression".into(), None)?;
        let mut zoom_events = 0;
        for frame in 0..6 {
            if frame == 3 {
                let baseline = app.observation.as_ref().ok_or("baseline observation")?;
                capture
                    .captures
                    .push(json!({"observed_state":baseline.state}));
            }
            let mut input = RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(1200.0, 900.0),
                )),
                time: Some(f64::from(frame) / 60.0),
                focused: true,
                ..Default::default()
            };
            if frame >= 3 {
                capture.inject(&mut input, app.observation.as_ref());
                zoom_events += input
                    .events
                    .iter()
                    .filter(|event| matches!(event, Event::Zoom(_)))
                    .count();
                assert!(
                    !input
                        .events
                        .iter()
                        .any(|event| matches!(event, Event::PointerButton { .. }))
                );
            }
            app.host.prepare_input(&ctx, &input);
            let mut output = ctx.run_ui(input, |root| app.show(root));
            output.textures_delta.clear();
            output.platform_output.commands.clear();
            if frame == 3 {
                // The view reports the camera used to paint this frame, not the
                // navigation camera scheduled by that frame's Zoom event.
                assert!(!capture.check(
                    Check::Zoomed,
                    app.observation.as_ref().ok_or("zoom input observation")?
                ));
            }
        }
        assert_eq!(zoom_events, 1);
        assert_eq!(capture.at, 1);
        assert!(capture.check(
            Check::Zoomed,
            app.observation.as_ref().ok_or("zoomed observation")?
        ));
        assert_eq!(app.session.document().raw_json(), crate::FIXTURE);
        std::fs::remove_dir(&capture.directory)?;
        Ok(())
    }

    #[test]
    fn capture_waits_for_real_focus_without_consuming_the_first_gesture() -> CaptureResult<()> {
        let (mut capture, observation) = focus_fixture()?;
        let mut input = RawInput {
            focused: false,
            events: vec![Event::WindowFocused(false)],
            ..Default::default()
        };
        capture.inject(&mut input, Some(&observation));
        assert!(!input.focused);
        assert_eq!(input.events.len(), 1);
        assert!(capture.waiting_for_focus);
        assert_eq!(capture.phase, 0);
        assert_eq!(capture.last_injected, 0);
        assert!(capture.input_failure.is_none());

        let mut focused = RawInput {
            focused: true,
            events: vec![Event::WindowFocused(true)],
            ..Default::default()
        };
        capture.inject(&mut focused, Some(&observation));
        assert_eq!(focused.events.len(), 3);
        assert_eq!(capture.phase, 1);
        assert_eq!(capture.last_injected, 1);
        assert!(!capture.waiting_for_focus);
        std::fs::remove_dir(&capture.directory)?;
        Ok(())
    }

    #[test]
    fn capture_stops_after_focus_loss_even_if_focus_returns() -> CaptureResult<()> {
        let (mut capture, mut observation) = focus_fixture()?;
        let mut input = RawInput {
            focused: true,
            ..Default::default()
        };
        capture.inject(&mut input, Some(&observation));
        assert_eq!(capture.phase, 1);
        observation.frame += 1;
        // A loss and regain in one integration batch must not hide interruption.
        let mut interrupted = RawInput {
            focused: true,
            events: vec![Event::WindowFocused(false), Event::WindowFocused(true)],
            ..Default::default()
        };
        capture.inject(&mut interrupted, Some(&observation));
        assert_eq!(interrupted.events.len(), 2);
        assert!(capture.input_failure.is_some());
        let mut regained = RawInput {
            focused: true,
            ..Default::default()
        };
        capture.inject(&mut regained, Some(&observation));
        assert!(regained.events.is_empty());
        assert_eq!(capture.phase, 1);
        assert_eq!(capture.last_injected, 1);
        std::fs::remove_dir(&capture.directory)?;
        Ok(())
    }

    fn competing_input(time: f64) -> RawInput {
        RawInput {
            focused: true,
            time: Some(time),
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(900.0, 500.0),
            )),
            events: vec![
                Event::PointerMoved(Pos2::new(839.0, 281.0)),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: Vec2::new(0.0, -12.0),
                    phase: egui::TouchPhase::Move,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        }
    }
    fn reject_button(ctx: &egui::Context, input: RawInput, center: Pos2) -> bool {
        let mut clicked = false;
        let output = ctx.run_ui(input, |root| {
            egui::CentralPanel::default().show(root, |ui| {
                clicked = ui
                    .put(
                        egui::Rect::from_center_size(center, Vec2::new(90.0, 28.0)),
                        egui::Button::new("Reject"),
                    )
                    .clicked();
            });
        });
        output.drop_without_applying_deltas();
        clicked
    }

    #[test]
    fn isolated_capture_preserves_a_real_button_click_despite_competing_pointer_and_wheel()
    -> CaptureResult<()> {
        let center = Pos2::new(125.0, 129.0);
        // Reproduce the observed integration batch with the old append-only policy.
        let mixed = egui::Context::default();
        reject_button(&mixed, competing_input(0.0), center);
        let mut press = competing_input(0.1);
        press.events.extend(pointer(center, true));
        assert!(!reject_button(&mixed, press, center));
        let mut release = competing_input(0.2);
        release.events.extend(pointer(center, false));
        assert!(!reject_button(&mixed, release, center));

        let (mut capture, mut observation) = focus_fixture()?;
        capture.steps = vec![Step::Click("button:reject")];
        observation.targets.insert("button:reject".into(), center);
        let isolated = egui::Context::default();
        let mut warmup = competing_input(0.0);
        capture.inject(&mut warmup, None);
        reject_button(&isolated, warmup, center);
        let mut press = competing_input(0.1);
        capture.inject(&mut press, Some(&observation));
        assert!(!reject_button(&isolated, press, center));
        observation.frame += 1;
        let mut release = competing_input(0.2);
        capture.inject(&mut release, Some(&observation));
        assert!(reject_button(&isolated, release, center));
        assert_eq!(capture.at, 1);
        assert_eq!(
            capture.upstream_suppressed_counts.get("pointer-moved"),
            Some(&3)
        );
        assert_eq!(
            capture.upstream_suppressed_counts.get("mouse-wheel"),
            Some(&3)
        );
        assert_eq!(
            capture
                .input_trace
                .iter()
                .filter(|entry| entry["delivery"] == "suppressed")
                .count(),
            6
        );
        assert_eq!(
            capture
                .input_trace
                .iter()
                .filter(|entry| entry["delivery"] == "injected")
                .count(),
            4
        );
        std::fs::remove_dir(&capture.directory)?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn isolation_keeps_native_lifecycle_and_redacts_suppressed_payloads() -> CaptureResult<()> {
        use std::sync::Arc;

        #[derive(Debug)]
        struct PrivateDrop {
            accessed: Arc<AtomicUsize>,
        }
        impl egui::DroppedFile for PrivateDrop {
            fn path(&self) -> &std::path::Path {
                self.accessed.fetch_add(1, Ordering::Relaxed);
                std::path::Path::new("/synthetic/private-drop-path")
            }
            fn bytes(&self) -> std::result::Result<Vec<u8>, String> {
                self.accessed.fetch_add(1, Ordering::Relaxed);
                Err("the isolated fixture must not read a dropped file".into())
            }
        }
        let (mut capture, _) = focus_fixture()?;
        let image = Arc::new(egui::ColorImage::filled([2, 2], egui::Color32::RED));
        let lifecycle = vec![
            Event::WindowFocused(true),
            Event::Screenshot {
                viewport_id: egui::ViewportId::ROOT,
                user_data: egui::UserData::new("private-screenshot-payload".to_owned()),
                image,
            },
        ];
        let mut events = vec![
            Event::Text("private-text-payload".into()),
            Event::Paste("private-paste-payload".into()),
            Event::Ime(egui::ImeEvent::Commit("private-ime-payload".into())),
            Event::Key {
                key: Key::Z,
                physical_key: Some(Key::Z),
                pressed: true,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            },
            Event::ModifiersChanged(Modifiers::COMMAND),
            Event::Touch {
                device_id: egui::TouchDeviceId(42),
                id: egui::TouchId(7),
                phase: egui::TouchPhase::Start,
                pos: Pos2::new(32.0, 48.0),
                force: Some(0.5),
            },
        ];
        events.extend(lifecycle.iter().cloned());
        let accessed = Arc::new(AtomicUsize::new(0));
        let rect = egui::Rect::from_min_size(Pos2::new(3.0, 5.0), Vec2::new(640.0, 480.0));
        let mut input = RawInput {
            focused: true,
            time: Some(42.0),
            predicted_dt: 0.04,
            screen_rect: Some(rect),
            max_texture_side: Some(4096),
            system_theme: Some(egui::Theme::Dark),
            events,
            hovered_files: vec![egui::HoveredFile {
                path: Some(PathBuf::from("/synthetic/private-hover-path")),
                mime: "private-mime-payload".into(),
            }],
            dropped_files: vec![Arc::new(PrivateDrop {
                accessed: Arc::clone(&accessed),
            })],
            ..Default::default()
        };
        if let Some(viewport) = input.viewports.get_mut(&egui::ViewportId::ROOT) {
            viewport.native_pixels_per_point = Some(2.0);
            viewport.inner_rect = Some(rect);
            viewport.events.push(egui::ViewportEvent::Close);
        }
        capture.inject(&mut input, None);
        assert_eq!(input.events, lifecycle);
        assert!(input.focused);
        assert_eq!(input.time, Some(42.0));
        assert_eq!(input.predicted_dt, 0.04);
        assert_eq!(input.screen_rect, Some(rect));
        assert_eq!(input.max_texture_side, Some(4096));
        assert_eq!(input.system_theme, Some(egui::Theme::Dark));
        let viewport = input
            .viewports
            .get(&egui::ViewportId::ROOT)
            .ok_or("native viewport was removed")?;
        assert_eq!(viewport.native_pixels_per_point, Some(2.0));
        assert_eq!(viewport.inner_rect, Some(rect));
        assert_eq!(viewport.events, vec![egui::ViewportEvent::Close]);
        assert!(input.hovered_files.is_empty() && input.dropped_files.is_empty());
        assert_eq!(accessed.load(Ordering::Relaxed), 0);
        assert_eq!(
            capture
                .upstream_suppressed_counts
                .get("hovered-files-redacted"),
            Some(&1)
        );
        assert_eq!(
            capture
                .upstream_suppressed_counts
                .get("dropped-files-redacted"),
            Some(&1)
        );
        let trace = serde_json::to_string(&capture.input_trace)?;
        assert!(!trace.contains("private-"));
        assert!(trace.contains("retained-native-lifecycle"));
        assert_eq!(
            capture
                .input_trace
                .iter()
                .filter(|entry| entry["delivery"] == "suppressed")
                .count(),
            8
        );

        // Completion stops injection, not isolation, until the native close completes.
        capture.done = true;
        let mut closing = competing_input(43.0);
        closing.focused = false;
        closing.events.push(Event::WindowFocused(false));
        capture.inject(&mut closing, None);
        assert!(!closing.focused);
        assert_eq!(closing.events, vec![Event::WindowFocused(false)]);
        std::fs::remove_dir(&capture.directory)?;
        Ok(())
    }
}
