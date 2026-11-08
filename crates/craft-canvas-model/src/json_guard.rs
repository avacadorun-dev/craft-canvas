//! Structural preflight rejects duplicate decoded keys and bounds allocation before serde.
use crate::{CanvasError, CanvasLimits, Result};
use std::collections::HashSet;

pub(crate) fn check_json(bytes: &[u8], limits: &CanvasLimits) -> Result<()> {
    if bytes.is_empty() || bytes.len() > limits.max_bytes {
        return Err(CanvasError::Limit("snapshot bytes"));
    }
    let mut guard = Guard {
        bytes,
        at: 0,
        limits,
    };
    guard.value(0, None)?;
    guard.whitespace();
    if guard.at != bytes.len() {
        return Err(CanvasError::Invalid("trailing JSON input".into()));
    }
    Ok(())
}
struct Guard<'a> {
    bytes: &'a [u8],
    at: usize,
    limits: &'a CanvasLimits,
}
impl Guard<'_> {
    fn whitespace(&mut self) {
        while self.bytes.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at += 1;
        }
    }
    fn value(&mut self, depth: usize, array_limit: Option<usize>) -> Result<()> {
        if depth > self.limits.max_depth {
            return Err(CanvasError::Limit("JSON nesting"));
        }
        self.whitespace();
        match self.bytes.get(self.at) {
            Some(b'{') => {
                self.at += 1;
                self.whitespace();
                let mut keys = HashSet::new();
                if self.bytes.get(self.at) == Some(&b'}') {
                    self.at += 1;
                    return Ok(());
                }
                loop {
                    self.whitespace();
                    let start = self.at;
                    self.string()?;
                    let key: String = serde_json::from_slice(&self.bytes[start..self.at])?;
                    if !keys.insert(key.clone()) {
                        return Err(CanvasError::DuplicateKey(key));
                    }
                    self.whitespace();
                    self.take(b':')?;
                    let bound = if depth == 0 {
                        match key.as_str() {
                            "nodes" => Some(self.limits.max_nodes),
                            "edges" => Some(self.limits.max_edges),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    self.value(depth + 1, bound)?;
                    self.whitespace();
                    match self.bytes.get(self.at) {
                        Some(b',') => self.at += 1,
                        Some(b'}') => {
                            self.at += 1;
                            break;
                        }
                        _ => return self.invalid(),
                    }
                }
            }
            Some(b'[') => {
                self.at += 1;
                self.whitespace();
                let mut count = 0usize;
                if self.bytes.get(self.at) == Some(&b']') {
                    self.at += 1;
                    return Ok(());
                }
                loop {
                    count = count
                        .checked_add(1)
                        .ok_or(CanvasError::Limit("array items"))?;
                    if array_limit.is_some_and(|limit| count > limit) {
                        return Err(CanvasError::Limit("node/edge array items"));
                    }
                    self.value(depth + 1, None)?;
                    self.whitespace();
                    match self.bytes.get(self.at) {
                        Some(b',') => self.at += 1,
                        Some(b']') => {
                            self.at += 1;
                            break;
                        }
                        _ => return self.invalid(),
                    }
                }
            }
            Some(b'"') => self.string()?,
            Some(_) => {
                let start = self.at;
                while self
                    .bytes
                    .get(self.at)
                    .is_some_and(|byte| !byte.is_ascii_whitespace() && !b",]}".contains(byte))
                {
                    self.at += 1;
                }
                if self.at == start {
                    return self.invalid();
                }
            }
            None => return self.invalid(),
        }
        Ok(())
    }
    fn string(&mut self) -> Result<()> {
        self.take(b'"')?;
        loop {
            match self.bytes.get(self.at) {
                Some(b'"') => {
                    self.at += 1;
                    return Ok(());
                }
                Some(b'\\') => {
                    self.at += 2;
                }
                Some(byte) if *byte < 0x20 => return self.invalid(),
                Some(_) => self.at += 1,
                None => return self.invalid(),
            }
        }
    }
    fn take(&mut self, byte: u8) -> Result<()> {
        if self.bytes.get(self.at) != Some(&byte) {
            return self.invalid();
        }
        self.at += 1;
        Ok(())
    }
    fn invalid<T>(&self) -> Result<T> {
        Err(CanvasError::Invalid("malformed JSON structure".into()))
    }
}
