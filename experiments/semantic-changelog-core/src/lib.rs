//! Bounded semantic evidence; analysis never edits original artifacts or executes plugins.
//! Explicit checkpoint module separately creates local metadata files.
use std::collections::{BTreeMap, BTreeSet};
pub mod skyrim;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Unsigned(u64),
    FloatBits(u32),
    Text(String),
    Bytes(Vec<u8>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entity {
    pub fields: BTreeMap<String, Value>,
    /// Unknown payloads remain exact evidence, not inferred semantics.
    pub opaque: BTreeMap<String, Vec<u8>>,
    pub limitations: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub format: String,
    pub parser: String,
    pub entities: BTreeMap<String, Entity>,
    /// True only when every entity identity was enumerated successfully.
    pub inventory_complete: bool,
    pub limitations: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Added(String),
    Removed(String),
    Field {
        entity: String,
        path: String,
        before: Option<Value>,
        after: Option<Value>,
    },
    Opaque {
        entity: String,
        path: String,
    },
    Unknown {
        entity: String,
        reason: String,
    },
}
#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    pub changes: Vec<Change>,
    pub semantic_complete: bool,
    pub limitations: Vec<String>,
}
/// Caller binds snapshots to exact release identities and cryptographic source hashes.
/// This layer must never label bytes as bug fixes, author intent, or compatibility.
fn unavailable(entity: &Entity, path: &str) -> bool {
    !entity.limitations.is_empty()
        || entity.opaque.keys().any(|root| {
            path == root
                || path
                    .strip_prefix(root.as_str())
                    .is_some_and(|tail| tail.starts_with('.') || tail.starts_with('['))
        })
}
pub fn compare(old: &Snapshot, new: &Snapshot) -> Result<Report, &'static str> {
    if old.format != new.format || old.parser != new.parser {
        return Err("incompatible snapshot format/parser");
    }
    let mut r = Report {
        changes: Vec::new(),
        semantic_complete: old.inventory_complete
            && new.inventory_complete
            && old.limitations.is_empty()
            && new.limitations.is_empty(),
        limitations: old
            .limitations
            .iter()
            .chain(&new.limitations)
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    };
    let keys: BTreeSet<_> = old.entities.keys().chain(new.entities.keys()).collect();
    for key in keys {
        match (old.entities.get(key), new.entities.get(key)) {
            (None, Some(_)) if old.inventory_complete => r.changes.push(Change::Added(key.clone())),
            (Some(_), None) if new.inventory_complete => {
                r.changes.push(Change::Removed(key.clone()))
            }
            (Some(a), Some(b)) => {
                let paths: BTreeSet<_> = a.fields.keys().chain(b.fields.keys()).collect();
                for path in paths {
                    if a.fields.get(path) != b.fields.get(path) {
                        if (!a.fields.contains_key(path) && unavailable(a, path))
                            || (!b.fields.contains_key(path) && unavailable(b, path))
                        {
                            r.semantic_complete = false;
                            r.changes.push(Change::Unknown {
                                entity: key.clone(),
                                reason: format!(
                                    "{path}: missing value on an incompletely decoded side"
                                ),
                            });
                        } else {
                            r.changes.push(Change::Field {
                                entity: key.clone(),
                                path: path.clone(),
                                before: a.fields.get(path).cloned(),
                                after: b.fields.get(path).cloned(),
                            });
                        }
                    }
                }
                let opaque: BTreeSet<_> = a.opaque.keys().chain(b.opaque.keys()).collect();
                for path in opaque {
                    if a.opaque.get(path) != b.opaque.get(path) {
                        r.changes.push(Change::Opaque {
                            entity: key.clone(),
                            path: path.clone(),
                        });
                    }
                }
            }
            _ => {
                r.semantic_complete = false;
                r.changes.push(Change::Unknown {
                    entity: key.clone(),
                    reason: "counterpart inventory incomplete; absence is not addition/removal"
                        .into(),
                });
            }
        }
        // Uninterpreted data or partial decodes cannot become a complete semantic verdict.
        for e in [old.entities.get(key), new.entities.get(key)]
            .into_iter()
            .flatten()
        {
            if !e.opaque.is_empty() || !e.limitations.is_empty() {
                r.semantic_complete = false;
            }
            for reason in &e.limitations {
                let c = Change::Unknown {
                    entity: key.clone(),
                    reason: reason.clone(),
                };
                if !r.changes.contains(&c) {
                    r.changes.push(c);
                }
            }
        }
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snap() -> Snapshot {
        Snapshot {
            format: "fixture".into(),
            parser: "1".into(),
            entities: BTreeMap::new(),
            inventory_complete: true,
            limitations: vec![],
        }
    }
    #[test]
    fn incomplete_absence_not_removal() {
        let mut a = snap();
        a.entities.insert(
            "a".into(),
            Entity {
                fields: BTreeMap::new(),
                opaque: BTreeMap::new(),
                limitations: vec![],
            },
        );
        let mut b = snap();
        b.inventory_complete = false;
        let r = compare(&a, &b).unwrap();
        assert!(!r.semantic_complete);
        assert!(matches!(r.changes[0], Change::Unknown { .. }));
    }
    #[test]
    fn parser_mismatch_refused() {
        let a = snap();
        let mut b = snap();
        b.parser = "2".into();
        assert!(compare(&a, &b).is_err());
    }
    #[test]
    fn changed_opaque_not_interpreted() {
        let mut a = snap();
        a.entities.insert(
            "a".into(),
            Entity {
                fields: BTreeMap::new(),
                opaque: BTreeMap::from([("x".into(), vec![1])]),
                limitations: vec![],
            },
        );
        let mut b = a.clone();
        b.entities
            .get_mut("a")
            .unwrap()
            .opaque
            .insert("x".into(), vec![2]);
        let r = compare(&a, &b).unwrap();
        assert_eq!(
            r.changes,
            vec![Change::Opaque {
                entity: "a".into(),
                path: "x".into()
            }]
        );
        assert!(!r.semantic_complete);
    }
    #[test]
    fn opaque_parent_never_proves_field_removed() {
        let mut a = snap();
        a.entities.insert(
            "x".into(),
            Entity {
                fields: BTreeMap::from([("DATA[0].Damage".into(), Value::Unsigned(8))]),
                opaque: BTreeMap::new(),
                limitations: vec![],
            },
        );
        let mut b = a.clone();
        let e = b.entities.get_mut("x").unwrap();
        e.fields.clear();
        e.opaque.insert("DATA[0]".into(), vec![0; 10]);
        let r = compare(&a, &b).unwrap();
        assert!(!r.changes.iter().any(|c| matches!(c, Change::Field { .. })));
        assert!(
            r.changes
                .iter()
                .any(|c| matches!(c, Change::Unknown { .. }))
        );
    }
}

pub mod dbpf;

pub mod history;

pub mod edit_analysis;

pub mod checkpoint;

pub mod ini;

pub mod monitor;
