//! Read-only, scoped maintenance evidence. A clue about preserving mod state,
//! never an install decision or a guarantee that a user's files are protected.
use crate::{Observation, Result, Store, observation_from_row, public_url};
use rusqlite::params;
use serde_derive::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub enum MaintenanceAction {
    SaveWithoutMod,
    ReplaceMod,
    RemoveReferencedContent,
}
impl MaintenanceAction {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "save-without-mod" => Ok(Self::SaveWithoutMod),
            "replace-mod" => Ok(Self::ReplaceMod),
            "remove-referenced-content" => Ok(Self::RemoveReferencedContent),
            _ => Err("Unknown maintenance action".into()),
        }
    }
    fn key(self) -> &'static str {
        match self {
            Self::SaveWithoutMod => "save-without-mod",
            Self::ReplaceMod => "replace-mod",
            Self::RemoveReferencedContent => "remove-referenced-content",
        }
    }
    fn predicate(self) -> &'static str {
        match self {
            Self::SaveWithoutMod => "maintenance_hotel_lots_v1",
            Self::ReplaceMod => "maintenance_kitchen_presets_v1",
            Self::RemoveReferencedContent => "maintenance_thread_assets_v1",
        }
    }
    fn source_vocabulary(self) -> (&'static str, &'static str, &'static [&'static str]) {
        match self {
            Self::SaveWithoutMod => ("game_save", "save_without_suite_life", &["game_save"]),
            Self::ReplaceMod => (
                "generated_file",
                "replace_suite_life",
                &["mod_generated_state"],
            ),
            Self::RemoveReferencedContent => (
                "referenced_custom_content",
                "remove_thread_cc",
                &["mod_generated_state", "referenced_custom_content"],
            ),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scope {
    scope: String,
    label: String,
    location: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Notice {
    version: u32,
    kind: String,
    trigger: String,
    risk: String,
    preserve: Vec<Scope>,
    tested_in_game: bool,
    autofix: bool,
    install_status: String,
}

fn safe_copy(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}

fn evidence(row: &Observation, action: MaintenanceAction) -> Result<Value> {
    public_url(&row.source_url)?;
    if row.value.len() > 8192 || row.predicate != action.predicate() {
        return Err("Unsupported maintenance evidence".into());
    }
    let v: Notice = serde_json::from_str(&row.value)?;
    let (kind, trigger, expected) = action.source_vocabulary();
    if v.version != 1
        || v.kind != kind
        || v.trigger != trigger
        || v.tested_in_game
        || v.autofix
        || v.install_status != "unverified"
        || !safe_copy(&v.risk, 512)
        || v.preserve.len() != expected.len()
        || v.preserve.iter().zip(expected).any(|(r, k)| r.scope != *k)
    {
        return Err("Maintenance scope or claims do not match the action".into());
    }
    let mut scopes = Vec::with_capacity(v.preserve.len());
    for scope in v.preserve {
        let label_only = scope.scope != "mod_generated_state";
        if !safe_copy(&scope.label, 384) || label_only != scope.location.is_none() {
            return Err("Generated and player data location semantics differ".into());
        }
        if let Some(basename) = &scope.location
            && (basename.len() > 120
                || basename.starts_with('.')
                || basename.ends_with('.')
                || basename.is_empty()
                || !basename
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.')))
        {
            return Err("Generated state name must be a single basename, never a path".into());
        }
        scopes.push(
            json!({"scope":scope.scope,"label":scope.label,"basename_hint_only":scope.location}),
        );
    }
    Ok(
        json!({"id":row.id,"source_id":row.source_id,"source_url":row.source_url,
        "observed_unix":row.observed_unix,"mod_version":row.mod_version,
        "game_build_evidence":row.applicable_build,"risk":v.risk,"preserve":scopes}),
    )
}

impl Store {
    /// Reports only posted advisories for this exact mod version and action.
    /// Caller-supplied version is not an assertion of a local installation.
    /// Absence, withdrawal, stale or invalid evidence never means "safe."
    pub fn maintenance_report(
        &self,
        entity: &str,
        version: &str,
        action: MaintenanceAction,
        as_of: i64,
        max_age: i64,
    ) -> Result<Value> {
        if !safe_copy(entity, 160)
            || !safe_copy(version, 96)
            || version.trim() != version
            || ["unverified", "unknown", "unspecified", "n/a", "*"]
                .iter()
                .any(|s| version.eq_ignore_ascii_case(s))
            || as_of < 0
            || max_age < 0
        {
            return Err(
                "Exact project, mod version and nonnegative evidence window required".into(),
            );
        }
        const MAX: usize = 512;
        let mut query = self.db.prepare(
            "SELECT o.id,o.entity_id,o.source_id,o.source_url,o.predicate,o.value,
                    o.observed_unix,o.applicable_build,o.mod_version,o.completeness,o.policy_revision
               FROM observations o JOIN sources s ON s.id=o.source_id
              WHERE o.entity_id=?1 AND o.mod_version=?2 AND o.predicate=?3
                AND o.observed_unix<=?4 AND s.can_retain=1 AND s.can_publish=1
                AND o.policy_revision=s.policy_revision
              ORDER BY o.observed_unix DESC,o.id LIMIT 513",
        )?;
        let rows = query.query_map(
            params![entity, version, action.predicate(), as_of],
            observation_from_row,
        )?;
        let mut advisories = Vec::new();
        let mut seen = BTreeSet::new();
        let mut invalid = 0;
        let mut stale = 0;
        let mut unreviewed = 0;
        for (idx, row) in rows.enumerate() {
            if idx == MAX {
                return Err(
                    "Maintenance evidence bound exceeded; no partial decision emitted".into(),
                );
            }
            let row = row?;
            let Ok(note) = evidence(&row, action) else {
                invalid += 1;
                continue;
            };
            if row.completeness == "unknown" {
                unreviewed += 1;
                continue;
            }
            if as_of.saturating_sub(row.observed_unix) > max_age {
                stale += 1;
                continue;
            }
            // Exact identical historical claims with the same source URL and
            // build scope are shown once; differing assertions remain visible.
            if seen.insert((
                row.source_id,
                row.source_url,
                row.applicable_build,
                row.value,
            )) {
                advisories.push(note);
            }
        }
        Ok(
            json!({"schema":"athena.mod-maintenance-advisories.v1","entity_id":entity,
            "requested_mod_version":version,"requested_action":action.key(),"as_of_unix":as_of,
            "coverage":if advisories.is_empty(){"unknown"} else {"reviewed_advisories"},
            "invalid_rows":invalid,"stale_rows":stale,"unreviewed_rows":unreviewed,
            "local_installation_verified":false,"safe_to_apply":null,"advisories":advisories}),
        )
    }
}
