//! Isolated synthetic native host for CraftCanvas interaction qualification.
mod capture;

use craft_canvas_egui::{
    CanvasInputOwnership, CanvasIntent, CanvasView, CanvasViewConfig, CanvasViewHost,
    ConnectionOptions, ConnectionSubmissionState, NodeBodyResponse,
};
use craft_canvas_model::{
    CanvasCommand, CanvasDocument, CanvasError, CanvasLimits, CanvasSession, CanvasTransaction,
    ConnectorEndpointKind, ContentIdentity, NodeRecord, PortEndpoint, PreparedChange,
    StandardDialect,
};
use eframe::egui::{self, Id, Pos2, Rect, Ui, Vec2};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const FIXTURE: &str = r#"{"nodes":[
 {"id":"a","type":"text","x":-410,"y":-180,"width":260,"height":230,"text":"Note · editable native body","craftCanvas":{"version":1,"ports":[{"id":"main","kind":"source","side":"right","x":260,"y":60},{"id":"alternative","kind":"source","side":"right","x":260,"y":150},{"id":"return","kind":"target","side":"left","x":0,"y":190}]}},
 {"id":"b","type":"text","x":60,"y":-180,"width":260,"height":230,"text":"Scene · scrollable body","craftCanvas":{"version":1,"ports":[{"id":"input","kind":"target","side":"left","x":0,"y":60},{"id":"alternative","kind":"target","side":"left","x":0,"y":150},{"id":"next","kind":"source","side":"right","x":260,"y":190}]}},
 {"id":"c","type":"text","x":-160,"y":160,"width":300,"height":150,"text":"Embedded controls","craftCanvas":{"version":1,"ports":[{"id":"in","kind":"target","side":"left","x":0,"y":80}]}}
 ],"edges":[{"id":"legacy","fromNode":"a","toNode":"b","label":"Legacy · omitted sides","future":{"precision":1.23000}}],"synthetic":true}"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Acknowledgement {
    Immediate,
    Hold,
}

struct Host {
    draft: String,
    slider: f32,
    clicks: u32,
    next_edge: u64,
    ownership: CanvasInputOwnership,
    text_input_id: Option<Id>,
    ime_active: bool,
    ime_for_frame: bool,
}
impl Default for Host {
    fn default() -> Self {
        Self {
            draft:
                "Write here without moving the card.\nThis synthetic draft stays in this window."
                    .into(),
            slider: 0.4,
            clicks: 0,
            next_edge: 1,
            ownership: CanvasInputOwnership::default(),
            text_input_id: None,
            ime_active: false,
            ime_for_frame: false,
        }
    }
}
impl Host {
    fn observe_ime(&mut self, events: &[egui::Event]) {
        for event in events {
            if let egui::Event::Ime(ime) = event {
                self.ime_for_frame = true;
                match ime {
                    egui::ImeEvent::Preedit { text, .. } => self.ime_active = !text.is_empty(),
                    egui::ImeEvent::Commit(_) => self.ime_active = false,
                    _ => {}
                }
            }
        }
    }
    fn prepare_input(&mut self, ctx: &egui::Context, input: &egui::RawInput) {
        let owned = self
            .text_input_id
            .is_some_and(|id| ctx.memory(|memory| memory.has_focus(id)));
        self.ime_for_frame = owned && self.ime_active;
        if !owned || !input.focused {
            self.ime_active = false;
        } else {
            self.observe_ime(&input.events);
        }
    }
}
impl CanvasViewHost for Host {
    fn allocate_edge_id(
        &mut self,
        _base: ContentIdentity,
        _origin: &PortEndpoint,
    ) -> craft_canvas_model::Result<Option<String>> {
        let id = format!("showcase-edge-{}", self.next_edge);
        self.next_edge = self
            .next_edge
            .checked_add(1)
            .ok_or(CanvasError::IdentityExhausted)?;
        Ok(Some(id))
    }
    fn input_ownership(&self) -> CanvasInputOwnership {
        CanvasInputOwnership {
            ime: self.ownership.ime || self.ime_active || self.ime_for_frame,
            ..self.ownership
        }
    }
    fn paint_body(
        &mut self,
        ui: &mut Ui,
        node: &NodeRecord,
        rect: Rect,
        _zoom: f64,
    ) -> NodeBodyResponse {
        let mut child = ui.new_child(egui::UiBuilder::new().id_salt(node.id()).max_rect(rect));
        child.set_clip_rect(rect.intersect(ui.clip_rect()));
        child.label(node.body());
        let pointer;
        let mut keyboard = false;
        if node.id() == "a" {
            let response = child.add_sized(
                [rect.width(), 86.0],
                egui::TextEdit::multiline(&mut self.draft).id_salt("synthetic-draft"),
            );
            pointer =
                response.is_pointer_button_down_on() || response.clicked() || response.dragged();
            keyboard = response.has_focus();
            self.text_input_id = Some(response.id);
            if keyboard {
                child.input(|input| self.observe_ime(&input.events));
            }
            child.small("Two source ports on the right: main / alternative.");
        } else if node.id() == "b" {
            let hovered = child.rect_contains_pointer(rect);
            egui::ScrollArea::vertical()
                .id_salt("body-scroll")
                .max_height(150.0)
                .show(&mut child, |ui| {
                    for line in 1..=18 {
                        ui.label(format!("Synthetic scene beat {line}"));
                    }
                });
            pointer = hovered
                && child.input(|input| input.pointer.any_down() || input.pointer.any_released());
            self.ownership.scroll |= hovered
                && child.input(|input| {
                    input
                        .events
                        .iter()
                        .any(|event| matches!(event, egui::Event::MouseWheel { .. }))
                });
        } else {
            let slider =
                child.add(egui::Slider::new(&mut self.slider, 0.0..=1.0).text("Local value"));
            let button = child.button(format!("Local clicks: {}", self.clicks));
            if button.clicked() {
                self.clicks = self.clicks.saturating_add(1);
            }
            pointer = slider.is_pointer_button_down_on()
                || slider.dragged()
                || button.is_pointer_button_down_on()
                || button.clicked();
            keyboard = slider.has_focus() || button.has_focus();
        }
        self.ownership.pointer |= pointer;
        self.ownership.keyboard |= keyboard;
        self.ownership.ime |= keyboard
            && child.input(|input| {
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Ime(_)))
            });
        self.ownership.touch |= pointer
            && child.input(|input| {
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Touch { .. }))
            });
        NodeBodyResponse {
            pointer_captured: pointer,
            keyboard_captured: keyboard,
        }
    }
}

enum Action {
    Apply,
    Reject,
    Retry,
    Dismiss,
    Undo,
    Redo,
    External,
}

struct Showcase {
    session: CanvasSession,
    view: CanvasView,
    host: Host,
    prepared: Option<PreparedChange>,
    acknowledgement: Acknowledgement,
    overlay: bool,
    message: String,
    source_sha: String,
    capture: Option<capture::Capture>,
    observation: Option<capture::Observation>,
    counters: capture::Counters,
    event_log: Vec<Value>,
    frame: u64,
    last_reported: Option<(u64, String, usize)>,
}
impl Showcase {
    fn new(source_sha: String, capture_dir: Option<PathBuf>) -> AppResult<Self> {
        let limits = CanvasLimits::default();
        let session = CanvasSession::new(CanvasDocument::parse(FIXTURE, &limits)?, limits)?;
        let capturing = capture_dir.is_some();
        Ok(Self {
            session,
            view: CanvasView::default(),
            host: Host::default(),
            prepared: None,
            acknowledgement: if capturing {
                Acknowledgement::Hold
            } else {
                Acknowledgement::Immediate
            },
            overlay: false,
            message: "Synthetic in-memory host; Apply is not a durable save.".into(),
            capture: capture_dir
                .map(|directory| capture::Capture::new(directory, &source_sha, FIXTURE))
                .transpose()?,
            source_sha,
            observation: None,
            counters: capture::Counters::default(),
            event_log: vec![],
            frame: 0,
            last_reported: None,
        })
    }
    fn submit(&mut self, transaction: CanvasTransaction) -> craft_canvas_model::Result<()> {
        let candidate = match self.session.prepare(transaction, &StandardDialect) {
            Ok(candidate) => candidate,
            Err(error) => {
                if self.view.connection_pending() {
                    self.view.reject_connection(&error.to_string())?;
                }
                return Err(error);
            }
        };
        if self.view.connection_pending() {
            self.view.bind_connection_prepared(&candidate)?;
            self.counters.submissions += 1;
            self.prepared = Some(candidate);
            if self.acknowledgement == Acknowledgement::Immediate {
                self.apply_pending()?;
            } else {
                self.message = "Prepared and held. Canonical data is unchanged until Apply.".into();
            }
        } else {
            self.session.apply(candidate)?;
        }
        Ok(())
    }
    fn apply_pending(&mut self) -> craft_canvas_model::Result<()> {
        let candidate = self
            .prepared
            .take()
            .ok_or_else(|| CanvasError::Invalid("No held candidate to apply".into()))?;
        self.session.apply(candidate)?;
        self.view.acknowledge_connection(&self.session)?;
        self.counters.acknowledgements += 1;
        self.message = "Host acknowledged the exact in-memory candidate.".into();
        Ok(())
    }
    fn action(&mut self, action: Action) -> craft_canvas_model::Result<()> {
        match action {
            Action::Apply => self.apply_pending(),
            Action::Reject => {
                self.view
                    .reject_connection("Synthetic host refusal; retry or dismiss explicitly.")?;
                self.prepared = None;
                self.message =
                    "Host refused the candidate; the pending request remains visible.".into();
                Ok(())
            }
            Action::Retry => {
                let transaction = self
                    .view
                    .retry_connection(&self.session, &StandardDialect)?;
                self.submit(transaction)
            }
            Action::Dismiss => {
                self.view.dismiss_connection()?;
                self.prepared = None;
                self.message = "Rejected request dismissed without changing canonical data.".into();
                Ok(())
            }
            Action::Undo => self.session.apply(self.session.prepare_undo()?).map(|_| ()),
            Action::Redo => self.session.apply(self.session.prepare_redo()?).map(|_| ()),
            Action::External => {
                let node = self
                    .session
                    .document()
                    .node("a")
                    .ok_or_else(|| CanvasError::NotFound("a".into()))?;
                let transaction = CanvasTransaction::new(
                    self.session.identity(),
                    vec![CanvasCommand::EditText {
                        node_id: "a".into(),
                        expected: node.body().into(),
                        text: format!("{} · external", node.body()),
                    }],
                );
                self.session
                    .apply(self.session.prepare(transaction, &StandardDialect)?)?;
                self.message =
                    "External synthetic revision applied; a held request must report conflict."
                        .into();
                Ok(())
            }
        }
    }
    fn record_result(&mut self, result: craft_canvas_model::Result<()>) {
        if let Err(error) = result {
            self.message = error.to_string();
        }
    }
    fn header(&mut self, ui: &mut Ui, targets: &mut BTreeMap<String, Pos2>) {
        ui.heading("CraftCanvas Showcase");
        ui.small(format!(
            "Source {} · drag a port; select an edge to reconnect either end",
            self.source_sha
        ));
        let mut actions = vec![];
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.acknowledgement,
                Acknowledgement::Immediate,
                "Immediate acknowledgement",
            );
            ui.selectable_value(
                &mut self.acknowledgement,
                Acknowledgement::Hold,
                "Hold for host",
            );
            ui.checkbox(&mut self.overlay, "Overlay exclusion");
        });
        let pending = self.view.connection_pending();
        let rejected = matches!(
            self.view.connection_status(),
            Some(ConnectionSubmissionState::Rejected(_))
        );
        ui.horizontal(|ui| {
            for (key, label, enabled, action) in [
                (
                    "apply",
                    "Apply held",
                    self.prepared.is_some(),
                    Action::Apply,
                ),
                ("reject", "Reject", pending && !rejected, Action::Reject),
                ("retry", "Retry", rejected, Action::Retry),
                ("dismiss", "Dismiss", rejected, Action::Dismiss),
                (
                    "undo",
                    "Undo",
                    !pending && self.session.history_usage().undo_steps > 0,
                    Action::Undo,
                ),
                (
                    "redo",
                    "Redo",
                    !pending && self.session.history_usage().redo_steps > 0,
                    Action::Redo,
                ),
                ("external", "External revision", pending, Action::External),
            ] {
                let response = ui.add_enabled(enabled, egui::Button::new(label));
                targets.insert(format!("button:{key}"), response.rect.center());
                if response.clicked() {
                    actions.push(action);
                }
            }
        });
        for action in actions {
            let result = self.action(action);
            self.record_result(result);
        }
        ui.label(&self.message);
        ui.small(format!(
            "Revision {} · edges {} · Undo {} · connection {:?}",
            self.session.identity().revision(),
            self.session.document().edges().len(),
            self.session.history_usage().undo_steps,
            self.view.connection_status()
        ));
    }
}

impl eframe::App for Showcase {
    fn raw_input_hook(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        self.host.prepare_input(ctx, input);
        if let Some(capture) = &mut self.capture {
            capture.inject(input, self.observation.as_ref());
        }
    }
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.frame = self.frame.saturating_add(1);
        let ctx = ui.ctx().clone();
        let mut targets = BTreeMap::new();
        egui::Panel::top("showcase-header").show(ui, |ui| self.header(ui, &mut targets));
        let mut exclusions = vec![];
        let mut overlay_keyboard = false;
        if self.overlay {
            let at = self
                .observation
                .as_ref()
                .and_then(|o| o.targets.get("port:b:target:input"))
                .copied()
                .unwrap_or(Pos2::new(650.0, 280.0));
            let overlay = egui::Area::new(Id::new("showcase-overlay"))
                .order(egui::Order::Foreground)
                .fixed_pos(at - Vec2::new(70.0, 25.0))
                .show(&ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label("Overlay owns this area");
                        let button = ui.button("Inert overlay control");
                        overlay_keyboard = button.has_focus();
                    });
                });
            exclusions.push(overlay.response.rect);
        }
        self.host.ownership = CanvasInputOwnership::default();
        egui::CentralPanel::default().show(ui, |ui| {
            let mut config = CanvasViewConfig::new(Id::new("isolated-synthetic-board"), ui);
            config.connections = Some(ConnectionOptions::default());
            config.input_exclusions = exclusions;
            config.input_ownership.keyboard = overlay_keyboard;
            config.input_ownership.ime = overlay_keyboard && ui.input(|input|input.events.iter().any(|event|matches!(event,egui::Event::Ime(_))));
            let output = self.view.show(ui, &self.session, &StandardDialect, &mut self.host, &config);
            for port in &output.ports { targets.insert(capture::port_target(&port.endpoint), port.center); }
            for handle in &output.reconnect_handles { targets.insert(format!("reconnect:{}:{}", handle.edge_id, if handle.end == ConnectorEndpointKind::From { "from" } else { "to" }), handle.center); }
            for edge in &output.edges { targets.insert(format!("edge:{}", edge.edge_id), edge.label_anchor); }
            for event in &output.connection_events {
                self.counters.observe(event);
                if self.event_log.len() == 256 { self.event_log.remove(0); }
                self.event_log.push(capture::event_json(event, self.frame));
            }
            let frame_geometry = json!({
                "viewport": [output.viewport.min.x, output.viewport.min.y, output.viewport.max.x, output.viewport.max.y],
                "camera": {"center": [output.frame_camera.center().x, output.frame_camera.center().y], "zoom": output.frame_camera.zoom()},
                "visible_ports": output.ports.len(), "visible_edges": output.edges.len(),
                "preview": output.connection_preview.as_ref().map(|preview| format!("{:?}", preview.state)),
                "consumed_input": format!("{:?}", output.consumed_input),
            });
            let errors: Vec<_> = output.errors.iter().map(ToString::to_string).collect();
            for error in output.errors { self.message = error.to_string(); }
            for intent in output.intents {
                let result = match intent {
                    CanvasIntent::Transaction(transaction) => { self.counters.intents += 1; self.submit(transaction) }
                    CanvasIntent::Undo(base) if base == self.session.identity() => self.action(Action::Undo),
                    CanvasIntent::Redo(base) if base == self.session.identity() => self.action(Action::Redo),
                    _ => { self.message = "This synthetic host only demonstrates Canvas connections and embedded controls.".into(); Ok(()) }
                };
                self.record_result(result);
            }
            self.observation = Some(capture::Observation::new(&self.session, &self.view, capture::FrameSnapshot { targets,geometry:frame_geometry,errors,frame:self.frame }, &self.counters, &self.event_log, &self.message));
        });
        if let (Some(capture), Some(observation)) = (&mut self.capture, &self.observation) {
            capture.observe(&ctx, observation);
            ctx.request_repaint_after(Duration::from_millis(30));
        }
        let observed_key = (
            self.session.identity().revision(),
            format!("{:?}", self.view.connection_status()),
            self.counters.begin
                + self.counters.submitted
                + self.counters.no_op
                + self.counters.invalid
                + self.counters.empty
                + self.counters.cancelled,
        );
        if self.last_reported.as_ref() != Some(&observed_key)
            && let Some(observation) = &self.observation
        {
            println!(
                "{}",
                json!({
                    "kind":"observed-host-state","source_sha":self.source_sha,
                    "input_provenance":if self.capture.is_some() {"scripted RawInput"} else {"native window events; no in-app injection"},
                    "observed_state":observation.state,
                })
            );
            self.last_reported = Some(observed_key);
        }
    }
}

fn main() -> AppResult<()> {
    let baked = option_env!("CRAFT_CANVAS_SOURCE_SHA");
    let mut source_sha = baked.unwrap_or("development").to_owned();
    let mut capture_dir = None;
    let mut scenario = "connections".to_owned();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--capture-dir" => {
                capture_dir = Some(PathBuf::from(
                    args.next().ok_or("--capture-dir requires a path")?,
                ))
            }
            "--source-sha" => source_sha = args.next().ok_or("--source-sha requires a value")?,
            "--scenario" => scenario = args.next().ok_or("--scenario requires a value")?,
            "--help" => {
                println!(
                    "CraftCanvas Showcase [--capture-dir NEW_DIRECTORY] [--source-sha SHA] [--scenario connections]\nSynthetic in-memory data only; captures identify injected events separately from OS input."
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown argument: {argument}").into()),
        }
    }
    if baked.is_some_and(|compiled| compiled != source_sha) {
        return Err("--source-sha does not match the compiled source identity".into());
    }
    if source_sha.len() > 64 || scenario != "connections" {
        return Err("Unsupported source identity or capture scenario".into());
    }
    let app = Showcase::new(source_sha.clone(), capture_dir)?;
    let title = format!(
        "CraftCanvas Showcase {}",
        source_sha.chars().take(12).collect::<String>()
    );
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(&title)
            .with_app_id("dev.craftcanvas.showcase")
            .with_inner_size([1200.0, 900.0])
            .with_min_inner_size([980.0, 760.0]),
        renderer: eframe::Renderer::Wgpu,
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(&title, options, Box::new(move |_cc| Ok(Box::new(app))))?;
    Ok(())
}
