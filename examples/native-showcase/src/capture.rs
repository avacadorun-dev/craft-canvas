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
    Click(&'static str),
    Drag(&'static str, &'static str),
    Cancel(&'static str),
    Check(Check),
}
fn steps() -> Vec<Step> {
    vec![
        Step::Snapshot("baseline"),
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
            done: false,
        })
    }
    pub fn inject(&mut self, input: &mut RawInput, observation: Option<&Observation>) {
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
        input.events.extend(events);
        self.last_injected = observation.frame;
        self.phase += 1;
        let complete = match step {
            Step::Click(_) => self.phase >= 2,
            Step::Drag(..) => self.phase >= 4,
            Step::Cancel(_) => self.phase >= 3,
            _ => false,
        };
        if complete {
            self.advance();
        }
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
            "os_input_acceptance":false,"physical_touchscreen_acceptance":false,"production_acceptance":false,
            "initial_canonical_json":self.initial,"elapsed_seconds":self.started.elapsed().as_secs_f64(),
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
