//! Versioned named-port metadata over preserved record envelopes.
use crate::{
    CanvasError, CanvasLimits, Result, Side, WorldPoint, WorldRect,
    document::{RawMap, raw},
};
use serde_json::value::RawValue;
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
};

/// Directional role of a named port, independently of its geometric side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PortKind {
    /// A normalized strict connection starts at this role.
    Source,
    /// A normalized strict connection ends at this role.
    Target,
}
impl PortKind {
    /// Stable version-one spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Target => "target",
        }
    }
}

/// Stable node-local identity; source and target may share a textual ID.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortKey {
    /// Nonempty identity, retained without normalization.
    pub id: String,
    /// Directional namespace of this identity.
    pub kind: PortKind,
}

/// A named connector anchor in node-local coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct PortDefinition {
    /// Stable node-local identity.
    pub key: PortKey,
    /// Outward connector tangent; this is not the port identity.
    pub side: Side,
    /// Finite node-local horizontal anchor coordinate.
    pub x: f64,
    /// Finite node-local vertical anchor coordinate.
    pub y: f64,
    /// Whether any new/reconnected connection may use this port.
    pub connectable: bool,
    /// Whether a connection may start here, independently of source/target role.
    pub start: bool,
    /// Whether a connection may finish here, independently of source/target role.
    pub end: bool,
}
impl PortDefinition {
    /// Resolve this node-local anchor against current node geometry, refusing overflow.
    pub fn world_anchor(&self, node: WorldRect) -> Result<WorldPoint> {
        WorldPoint::new(node.x + self.x, node.y + self.y)
    }
}

/// Canonical binding, separate from any rendered fallback anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortBinding {
    /// Standard JSON Canvas side; absence is not a named or rendered default.
    Legacy(Option<Side>),
    /// Exact named key, with no fallback to another port or side.
    Named(PortKey),
}

/// Exact node and port binding used by guarded connection commands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortEndpoint {
    /// Board-local node identity.
    pub node_id: String,
    /// Legacy or named canonical binding.
    pub binding: PortBinding,
}

/// Which original edge endpoints permit deliberate reconnection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReconnectPolicy {
    /// Use the host's concrete default.
    #[default]
    Inherit,
    /// Neither endpoint may be reconnected.
    None,
    /// Either endpoint may be reconnected.
    Both,
    /// Only the source endpoint may be reconnected.
    From,
    /// Only the destination endpoint may be reconnected.
    To,
}
impl ReconnectPolicy {
    /// Stable version-one spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::None => "none",
            Self::Both => "both",
            Self::From => "from",
            Self::To => "to",
        }
    }
}

/// Whether preserved extension metadata can be interpreted safely.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtensionStatus {
    /// No extension exists; compatible legacy defaults apply.
    Absent,
    /// Recognized version one with valid known members.
    KnownV1,
    /// Unknown namespace shape or version, retained without interpretation.
    Opaque,
    /// Version one has malformed recognized members, retained without repair.
    Malformed,
}

#[derive(Debug)]
pub(crate) struct NodePorts {
    pub status: ExtensionStatus,
    pub ports: Vec<PortDefinition>,
    index: HashMap<PortKey, usize>,
}
impl NodePorts {
    pub fn parse(fields: &RawMap, limits: &CanvasLimits) -> Result<Self> {
        let (status, namespace) = namespace(fields);
        let Some(namespace) = namespace else {
            return Ok(Self {
                status,
                ports: vec![],
                index: HashMap::new(),
            });
        };
        match parse_ports(&namespace, limits) {
            Ok(ports) => {
                let index = ports
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (p.key.clone(), i))
                    .collect();
                Ok(Self {
                    status,
                    ports,
                    index,
                })
            }
            Err(error @ CanvasError::Limit(_)) => Err(error),
            Err(_) => Ok(Self {
                status: ExtensionStatus::Malformed,
                ports: vec![],
                index: HashMap::new(),
            }),
        }
    }
    pub fn get(&self, key: &PortKey) -> Option<&PortDefinition> {
        self.index.get(key).map(|at| &self.ports[*at])
    }
    pub fn payload_bytes(&self) -> usize {
        self.ports.iter().map(|p| p.key.id.len()).sum::<usize>()
            + self.index.keys().map(|key| key.id.len()).sum::<usize>()
    }
}

#[derive(Debug)]
pub(crate) struct EdgePorts {
    pub status: ExtensionStatus,
    pub from: Option<PortKey>,
    pub to: Option<PortKey>,
    pub reconnect: ReconnectPolicy,
}
impl EdgePorts {
    pub fn parse(fields: &RawMap, limits: &CanvasLimits) -> Result<Self> {
        let (status, namespace) = namespace(fields);
        let mut result = Self {
            status,
            from: None,
            to: None,
            reconnect: ReconnectPolicy::Inherit,
        };
        let Some(namespace) = namespace else {
            return Ok(result);
        };
        let known = || -> Result<_> {
            let from = namespace
                .get("fromPort")
                .map(|v| parse_key_object(v, limits))
                .transpose()?;
            let to = namespace
                .get("toPort")
                .map(|v| parse_key_object(v, limits))
                .transpose()?;
            let reconnect = match namespace.get("reconnect") {
                None => ReconnectPolicy::Inherit,
                Some(value) => match serde_json::from_str::<String>(value.get())?.as_str() {
                    "inherit" => ReconnectPolicy::Inherit,
                    "none" => ReconnectPolicy::None,
                    "both" => ReconnectPolicy::Both,
                    "from" => ReconnectPolicy::From,
                    "to" => ReconnectPolicy::To,
                    _ => return Err(CanvasError::Invalid("invalid reconnect policy".into())),
                },
            };
            Ok((from, to, reconnect))
        };
        match known() {
            Ok((from, to, reconnect)) => {
                result.from = from;
                result.to = to;
                result.reconnect = reconnect;
            }
            Err(error @ CanvasError::Limit(_)) => return Err(error),
            Err(_) => result.status = ExtensionStatus::Malformed,
        }
        Ok(result)
    }
    pub fn payload_bytes(&self) -> usize {
        self.from.as_ref().map_or(0, |p| p.id.len()) + self.to.as_ref().map_or(0, |p| p.id.len())
    }
}

fn namespace(fields: &RawMap) -> (ExtensionStatus, Option<RawMap>) {
    let Some(value) = fields.get("craftCanvas") else {
        return (ExtensionStatus::Absent, None);
    };
    let Ok(fields) = serde_json::from_str::<RawMap>(value.get()) else {
        return (ExtensionStatus::Opaque, None);
    };
    if fields
        .get("version")
        .is_some_and(|value| serde_json::from_str::<u64>(value.get()).ok() == Some(1))
    {
        (ExtensionStatus::KnownV1, Some(fields))
    } else {
        (ExtensionStatus::Opaque, None)
    }
}

fn required<T: serde::de::DeserializeOwned>(fields: &RawMap, key: &str) -> Result<T> {
    Ok(serde_json::from_str(
        fields
            .get(key)
            .ok_or_else(|| CanvasError::Invalid(format!("missing port {key}")))?
            .get(),
    )?)
}
fn parse_key(fields: &RawMap, limits: &CanvasLimits) -> Result<PortKey> {
    let key = PortKey {
        id: required(fields, "id")?,
        kind: match required::<String>(fields, "kind")?.as_str() {
            "source" => PortKind::Source,
            "target" => PortKind::Target,
            _ => return Err(CanvasError::Invalid("invalid port kind".into())),
        },
    };
    validate_key(&key, limits)?;
    Ok(key)
}
fn parse_key_object(value: &RawValue, limits: &CanvasLimits) -> Result<PortKey> {
    parse_key(&serde_json::from_str(value.get())?, limits)
}
pub(crate) fn validate_key(key: &PortKey, limits: &CanvasLimits) -> Result<()> {
    if key.id.len() > limits.max_id_bytes {
        return Err(CanvasError::Limit("port ID bytes"));
    }
    if key.id.trim().is_empty() {
        return Err(CanvasError::Invalid("empty port ID".into()));
    }
    Ok(())
}
fn availability(fields: &RawMap, key: &str) -> Result<bool> {
    fields
        .get(key)
        .map(|value| serde_json::from_str(value.get()).map_err(Into::into))
        .unwrap_or(Ok(true))
}
fn coordinate(fields: &RawMap, key: &str) -> Result<f64> {
    required::<serde_json::Number>(fields, key)?
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or(CanvasError::Limit("port anchor coordinate"))
}
fn parse_ports(namespace: &RawMap, limits: &CanvasLimits) -> Result<Vec<PortDefinition>> {
    let Some(value) = namespace.get("ports") else {
        return Ok(vec![]);
    };
    let values = port_values(value, limits)?;
    let mut ports = Vec::with_capacity(values.len());
    let mut keys = HashSet::with_capacity(values.len());
    for value in values {
        let fields: RawMap = serde_json::from_str(value.get())?;
        let port = PortDefinition {
            key: parse_key(&fields, limits)?,
            side: match required::<String>(&fields, "side")?.as_str() {
                "top" => Side::Top,
                "right" => Side::Right,
                "bottom" => Side::Bottom,
                "left" => Side::Left,
                _ => return Err(CanvasError::Invalid("invalid port side".into())),
            },
            x: coordinate(&fields, "x")?,
            y: coordinate(&fields, "y")?,
            connectable: availability(&fields, "connectable")?,
            start: availability(&fields, "start")?,
            end: availability(&fields, "end")?,
        };
        validate_port(&port, limits)?;
        if !keys.insert(port.key.clone()) {
            return Err(CanvasError::Invalid("duplicate port identity".into()));
        }
        ports.push(port);
    }
    Ok(ports)
}
fn port_values(value: &RawValue, limits: &CanvasLimits) -> Result<Vec<Box<RawValue>>> {
    struct PortList<'a> {
        limit: usize,
        exceeded: &'a Cell<bool>,
    }
    impl<'de> serde::de::Visitor<'de> for PortList<'_> {
        type Value = Vec<Box<RawValue>>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded array of ports")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            while let Some(value) = seq.next_element::<Box<RawValue>>()? {
                if values.len() == self.limit {
                    self.exceeded.set(true);
                    return Err(serde::de::Error::custom("ports per node limit"));
                }
                values.push(value);
            }
            Ok(values)
        }
    }
    let exceeded = Cell::new(false);
    let mut decoder = serde_json::Deserializer::from_str(value.get());
    let parsed = serde::de::Deserializer::deserialize_seq(
        &mut decoder,
        PortList {
            limit: limits.max_ports_per_node,
            exceeded: &exceeded,
        },
    );
    if exceeded.get() {
        return Err(CanvasError::Limit("ports per node"));
    }
    Ok(parsed?)
}
pub(crate) fn validate_port(port: &PortDefinition, limits: &CanvasLimits) -> Result<()> {
    validate_key(&port.key, limits)?;
    if !port.x.is_finite()
        || !port.y.is_finite()
        || port.x.abs() > limits.max_coordinate
        || port.y.abs() > limits.max_coordinate
    {
        return Err(CanvasError::Limit("port anchor coordinate"));
    }
    Ok(())
}
pub(crate) fn ensure_editable(status: ExtensionStatus, id: &str) -> Result<()> {
    match status {
        ExtensionStatus::Absent | ExtensionStatus::KnownV1 => Ok(()),
        ExtensionStatus::Opaque => Err(crate::ConnectionError::OpaqueExtension(id.into()).into()),
        ExtensionStatus::Malformed => {
            Err(crate::ConnectionError::MalformedExtension(id.into()).into())
        }
    }
}
pub(crate) fn editable_namespace(fields: &RawMap) -> Result<RawMap> {
    match namespace(fields) {
        (ExtensionStatus::Absent, _) => {
            let mut fields = RawMap::new();
            fields.insert("version".into(), raw(&1u64)?);
            Ok(fields)
        }
        (_, Some(fields)) => Ok(fields),
        _ => Err(CanvasError::Invalid(
            "unsupported port extension mutation".into(),
        )),
    }
}
pub(crate) fn write_namespace(fields: &mut RawMap, namespace: RawMap) -> Result<()> {
    fields.insert("craftCanvas".into(), raw(&namespace)?);
    Ok(())
}
pub(crate) fn encode_key(old: Option<&RawValue>, key: &PortKey) -> Result<Box<RawValue>> {
    let mut fields: RawMap = old
        .map(|value| serde_json::from_str(value.get()))
        .transpose()?
        .unwrap_or_default();
    if fields
        .get("id")
        .and_then(|v| serde_json::from_str::<String>(v.get()).ok())
        .as_deref()
        != Some(key.id.as_str())
    {
        fields.insert("id".into(), raw(&key.id)?);
    }
    if fields
        .get("kind")
        .and_then(|v| serde_json::from_str::<String>(v.get()).ok())
        .as_deref()
        != Some(key.kind.as_str())
    {
        fields.insert("kind".into(), raw(&key.kind.as_str())?);
    }
    raw(&fields)
}
pub(crate) fn encode_ports(
    fields: &RawMap,
    current: &NodePorts,
    ports: &[PortDefinition],
    limits: &CanvasLimits,
) -> Result<RawMap> {
    if ports.len() > limits.max_ports_per_node {
        return Err(CanvasError::Limit("ports per node"));
    }
    let mut keys = HashSet::with_capacity(ports.len());
    for port in ports {
        validate_port(port, limits)?;
        if !keys.insert(&port.key) {
            return Err(CanvasError::Invalid("duplicate port identity".into()));
        }
    }
    let mut namespace = editable_namespace(fields)?;
    let old: Vec<Box<RawValue>> = namespace
        .get("ports")
        .map(|v| serde_json::from_str(v.get()))
        .transpose()?
        .unwrap_or_default();
    let mut values = Vec::with_capacity(ports.len());
    for port in ports {
        let previous = current
            .index
            .get(&port.key)
            .map(|at| (&current.ports[*at], old[*at].as_ref()));
        if let Some((same, value)) = previous
            && same == port
        {
            values.push(value.to_owned());
            continue;
        }
        let mut entry: RawMap = match previous {
            Some((_, value)) => serde_json::from_str(value.get())?,
            None => RawMap::new(),
        };
        let before = previous.map(|(p, _)| p);
        if before.is_none() {
            entry.insert("id".into(), raw(&port.key.id)?);
            entry.insert("kind".into(), raw(&port.key.kind.as_str())?);
        }
        if before.is_none_or(|p| p.side != port.side) {
            entry.insert("side".into(), raw(&port.side.as_str())?);
        }
        if before.is_none_or(|p| p.x != port.x) {
            entry.insert("x".into(), raw(&port.x)?);
        }
        if before.is_none_or(|p| p.y != port.y) {
            entry.insert("y".into(), raw(&port.y)?);
        }
        for (key, after, prior) in [
            (
                "connectable",
                port.connectable,
                before.is_none_or(|p| p.connectable),
            ),
            ("start", port.start, before.is_none_or(|p| p.start)),
            ("end", port.end, before.is_none_or(|p| p.end)),
        ] {
            if after != prior {
                entry.insert(key.into(), raw(&after)?);
            }
        }
        values.push(raw(&entry)?);
    }
    namespace.insert("ports".into(), raw(&values)?);
    let mut fields = fields.clone();
    write_namespace(&mut fields, namespace)?;
    Ok(fields)
}
