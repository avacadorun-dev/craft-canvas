//! Prepared changes cannot be deserialized or applied to a foreign/stale session.
use crate::{
    CanvasDialect, CanvasDocument, CanvasError, CanvasLimits, CanvasTransaction, Result,
    SelectionMap, WorkControl,
    commands::execute,
    document::{DocumentData, EdgeRecord, NodeRecord, RawMap},
};
use std::{
    collections::{HashSet, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);
/// Opaque exact in-process session identity. It is input context, not project authorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentIdentity {
    epoch: u64,
    revision: u64,
}
impl ContentIdentity {
    /// Whether both tokens belong to the same in-process editing session.
    pub fn same_session(self, other: Self) -> bool {
        self.epoch == other.epoch
    }
    /// Monotonic local change revision, including Undo/Redo.
    pub fn revision(self) -> u64 {
        self.revision
    }
}
/// Bounded retained source/record payload accounting, separate from allocator overhead.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HistoryUsage {
    /// Complete available Undo actions.
    pub undo_steps: usize,
    /// Complete available Redo actions.
    pub redo_steps: usize,
    /// Current snapshot's source/index/record text payload bytes.
    pub current_payload_bytes: usize,
    /// Unique retained text/source payload across current, past and future snapshots.
    pub retained_payload_bytes: usize,
    /// Worst historical payload across any retained current-state choice. This ensures
    /// that Undo/Redo cannot exceed the acknowledged byte budget just by changing current.
    pub history_budget_bytes: usize,
}
#[derive(Clone)]
struct HistoryEntry {
    document: CanvasDocument,
    mapping: SelectionMap,
}
/// A validated candidate and complete history transition, constructible only by a session.
pub struct PreparedChange {
    base: ContentIdentity,
    before: CanvasDocument,
    after: CanvasDocument,
    past: VecDeque<HistoryEntry>,
    future: Vec<HistoryEntry>,
    usage: HistoryUsage,
    mapping: SelectionMap,
    changed: bool,
}
impl PreparedChange {
    /// Frozen input session identity.
    pub fn base(&self) -> ContentIdentity {
        self.base
    }
    /// Complete candidate to validate/checkpoint through the host before Apply.
    pub fn document(&self) -> &CanvasDocument {
        &self.after
    }
    /// Stable identity relationships for the host's current selection.
    pub fn selection_map(&self) -> &SelectionMap {
        &self.mapping
    }
    /// Whether the change actually alters canonical bytes.
    pub fn changed(&self) -> bool {
        self.changed
    }
    /// Exact preflighted history accounting if acknowledged.
    pub fn history_usage(&self) -> HistoryUsage {
        self.usage
    }
}
/// One canonical editing session; durable persistence and ownership remain host concerns.
pub struct CanvasSession {
    identity: ContentIdentity,
    document: CanvasDocument,
    limits: CanvasLimits,
    past: VecDeque<HistoryEntry>,
    future: Vec<HistoryEntry>,
    usage: HistoryUsage,
}
impl CanvasSession {
    /// Open an already parsed document with the same source-compatible limits.
    pub fn new(document: CanvasDocument, limits: CanvasLimits) -> Result<Self> {
        limits.validate()?;
        // Reparse once under this session's limits; a proof from another limit profile
        // cannot silently authorize a larger editable board.
        let document = CanvasDocument::parse(document.raw_json(), &limits)?;
        let epoch = NEXT_SESSION
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| CanvasError::IdentityExhausted)?;
        let past = VecDeque::new();
        let future = vec![];
        let usage = usage(&document, &past, &future)?;
        Ok(Self {
            identity: ContentIdentity { epoch, revision: 0 },
            document,
            limits,
            past,
            future,
            usage,
        })
    }
    /// Current immutable canonical board.
    pub fn document(&self) -> &CanvasDocument {
        &self.document
    }
    /// Exact token for UI/headless requests.
    pub fn identity(&self) -> ContentIdentity {
        self.identity
    }
    /// Active bounded source policy.
    pub fn limits(&self) -> &CanvasLimits {
        &self.limits
    }
    /// Cached acknowledged history accounting.
    pub fn history_usage(&self) -> HistoryUsage {
        self.usage
    }
    /// Prepare with the configured default charged-work budget.
    pub fn prepare(
        &self,
        transaction: CanvasTransaction,
        dialect: &dyn CanvasDialect,
    ) -> Result<PreparedChange> {
        let mut work = WorkControl::new(self.limits.max_work_units, None);
        self.prepare_with_control(transaction, dialect, &mut work)
    }
    /// Prepare on a host worker with explicit charged work/cancellation. No session mutation.
    pub fn prepare_with_control(
        &self,
        transaction: CanvasTransaction,
        dialect: &dyn CanvasDialect,
        work: &mut WorkControl<'_>,
    ) -> Result<PreparedChange> {
        if transaction.base != self.identity {
            return Err(CanvasError::StaleBase);
        }
        let (after, mapping) = execute(
            &self.document,
            transaction.commands,
            dialect,
            &self.limits,
            work,
        )?;
        let changed = after.raw_json() != self.document.raw_json();
        let mut past = self.past.clone();
        let mut future = self.future.clone();
        if changed {
            self.identity
                .revision
                .checked_add(1)
                .ok_or(CanvasError::IdentityExhausted)?;
            past.push_back(HistoryEntry {
                document: self.document.clone(),
                mapping: mapping.clone(),
            });
            future.clear();
        }
        while past.len() > self.limits.max_history_steps {
            past.pop_front();
        }
        let mut next_usage = usage(&after, &past, &future)?;
        while next_usage.history_budget_bytes > self.limits.max_history_bytes && past.len() > 1 {
            past.pop_front();
            next_usage = usage(&after, &past, &future)?;
        }
        if next_usage.history_budget_bytes > self.limits.max_history_bytes {
            return Err(CanvasError::HistoryBudget);
        }
        Ok(PreparedChange {
            base: self.identity,
            before: self.document.clone(),
            after,
            past,
            future,
            usage: next_usage,
            mapping,
            changed,
        })
    }
    /// Prepare one compensating Undo at the current exact identity.
    pub fn prepare_undo(&self) -> Result<PreparedChange> {
        let mut past = self.past.clone();
        let previous = past.pop_back().ok_or(CanvasError::NoHistory)?;
        let mut future = self.future.clone();
        future.push(HistoryEntry {
            document: self.document.clone(),
            mapping: previous.mapping.clone(),
        });
        let mapping = previous.mapping.inverse();
        self.history_change(previous.document, past, future, mapping)
    }
    /// Prepare one compensating Redo at the current exact identity.
    pub fn prepare_redo(&self) -> Result<PreparedChange> {
        let mut future = self.future.clone();
        let next = future.pop().ok_or(CanvasError::NoHistory)?;
        let mut past = self.past.clone();
        past.push_back(HistoryEntry {
            document: self.document.clone(),
            mapping: next.mapping.clone(),
        });
        self.history_change(next.document, past, future, next.mapping)
    }
    fn history_change(
        &self,
        after: CanvasDocument,
        past: VecDeque<HistoryEntry>,
        future: Vec<HistoryEntry>,
        mapping: SelectionMap,
    ) -> Result<PreparedChange> {
        self.identity
            .revision
            .checked_add(1)
            .ok_or(CanvasError::IdentityExhausted)?;
        let next_usage = usage(&after, &past, &future)?;
        if next_usage.history_budget_bytes > self.limits.max_history_bytes {
            return Err(CanvasError::HistoryBudget);
        }
        Ok(PreparedChange {
            base: self.identity,
            before: self.document.clone(),
            after,
            past,
            future,
            usage: next_usage,
            mapping,
            changed: true,
        })
    }
    /// Acknowledge only this session's exact prepared transition. Host persistence
    /// must succeed before calling this when durable acknowledgement is required.
    pub fn apply(&mut self, change: PreparedChange) -> Result<SelectionMap> {
        if change.base != self.identity || !Arc::ptr_eq(&change.before.data, &self.document.data) {
            return Err(CanvasError::StaleBase);
        }
        if !change.changed {
            return Ok(change.mapping);
        }
        let revision = self
            .identity
            .revision
            .checked_add(1)
            .ok_or(CanvasError::IdentityExhausted)?;
        self.document = change.after;
        self.past = change.past;
        self.future = change.future;
        self.usage = change.usage;
        self.identity.revision = revision;
        Ok(change.mapping)
    }
}
#[derive(Default)]
struct Payloads {
    documents: HashSet<*const DocumentData>,
    nodes: HashSet<*const NodeRecord>,
    edges: HashSet<*const EdgeRecord>,
    tops: HashSet<*const RawMap>,
    bytes: usize,
}
impl Payloads {
    fn add(&mut self, document: &CanvasDocument) -> Result<()> {
        if self.documents.insert(Arc::as_ptr(&document.data)) {
            self.charge(document.raw_json().len())?;
            for node in document.nodes() {
                self.charge(node.id().len())?;
            }
            for edge in document.edges() {
                self.charge(edge.id().len())?;
            }
            if self.tops.insert(Arc::as_ptr(&document.data.top)) {
                for (key, value) in document.data.top.iter() {
                    self.charge(key.len() + value.get().len())?;
                }
            }
            for node in document.nodes() {
                if self.nodes.insert(Arc::as_ptr(node)) {
                    self.charge(node.payload_bytes())?;
                }
            }
            for edge in document.edges() {
                if self.edges.insert(Arc::as_ptr(edge)) {
                    self.charge(edge.payload_bytes())?;
                }
            }
        }
        Ok(())
    }
    fn charge(&mut self, amount: usize) -> Result<()> {
        self.bytes = self
            .bytes
            .checked_add(amount)
            .ok_or(CanvasError::HistoryBudget)?;
        Ok(())
    }
}
fn usage(
    current: &CanvasDocument,
    past: &VecDeque<HistoryEntry>,
    future: &[HistoryEntry],
) -> Result<HistoryUsage> {
    let mut all = Payloads::default();
    all.add(current)?;
    let current_payload_bytes = all.bytes;
    let mut min_current = current_payload_bytes;
    for entry in past.iter().chain(future) {
        let mut single = Payloads::default();
        single.add(&entry.document)?;
        min_current = min_current.min(single.bytes);
        all.add(&entry.document)?;
    }
    Ok(HistoryUsage {
        undo_steps: past.len(),
        redo_steps: future.len(),
        current_payload_bytes,
        retained_payload_bytes: all.bytes,
        history_budget_bytes: all
            .bytes
            .checked_sub(min_current)
            .ok_or(CanvasError::HistoryBudget)?,
    })
}
