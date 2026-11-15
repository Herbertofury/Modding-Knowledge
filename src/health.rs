//! Scoped evidence query, not a compatibility oracle or automated reputation score.
use crate::{Observation, Result, Store};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Import a structured, attributed assertion. Authority is a reviewed adapter claim,
/// not inferred from a username, popularity, or the mere presence of metadata.
pub fn validate_claim(row: &Observation) -> Result<Value> {
    if row.predicate != "health_claim_v1" {
        return Err("Not a health claim".into());
    }
    let v: Value = serde_json::from_str(&row.value)?;
    for key in ["assertion", "authority", "lineage", "context", "status"] {
        if v[key].as_str().is_none_or(|s| s.trim().is_empty()) {
            return Err(format!("Missing claim {key}").into());
        }
    }
    if !["works", "broken", "warning", "unknown"].contains(&v["assertion"].as_str().unwrap())
        || ![
            "official_release",
            "maintainer_report",
            "community_report",
            "independent_test",
        ]
        .contains(&v["authority"].as_str().unwrap())
        || !["active", "retracted", "superseded"].contains(&v["status"].as_str().unwrap())
        || v["reported_unix"]
            .as_i64()
            .is_none_or(|n| n < 0 || n > row.observed_unix)
        || row.applicable_build.trim().is_empty()
        || row.mod_version.trim().is_empty()
    {
        return Err("Invalid claim vocabulary, date or version scope".into());
    }
    // Deliberately no free-form comments or public reporter identities in this contract.
    let allowed = [
        "assertion",
        "authority",
        "lineage",
        "context",
        "status",
        "reported_unix",
    ];
    if v.as_object()
        .is_none_or(|o| o.keys().any(|k| !allowed.contains(&k.as_str())))
    {
        return Err("Unsupported claim fields".into());
    }
    Ok(v)
}
impl Store {
    pub fn import_health_claims(&mut self, rows: &[Observation]) -> Result<()> {
        for row in rows {
            validate_claim(row)?;
        }
        self.import(rows)
    }
    /// Exact build/version/runtime context only. Unscoped observations remain available
    /// in the general API, never promoted to compatibility evidence here. All history is
    /// returned up to a hard bound; exceeding it fails explicitly, never silently truncates.
    pub fn health_report(
        &self,
        entity: &str,
        build: &str,
        version: &str,
        context: &str,
        now: i64,
        max_age: i64,
    ) -> Result<Value> {
        if [build, version, context]
            .iter()
            .any(|s| s.trim().is_empty())
            || now < 0
            || max_age < 0
        {
            return Err("Explicit scope and nonnegative freshness required".into());
        }
        let mut rows = Vec::new();
        let mut offset = 0;
        loop {
            let page = self.observations_page(entity, offset)?;
            let len = page.len();
            rows.extend(page);
            if rows.len() > 10_000 {
                return Err(
                    "Evidence query exceeds 10000 observations; narrow entity scope".into(),
                );
            }
            if len < 1000 {
                break;
            }
            offset += 1000;
        }
        let mut claims = Vec::new();
        let mut assertions = BTreeSet::new();
        let mut lineage = BTreeSet::new();
        let mut stale = 0;
        let mut invalid = 0;
        for row in rows {
            if row.predicate != "health_claim_v1"
                || row.applicable_build != build
                || row.mod_version != version
            {
                continue;
            }
            let Ok(v) = validate_claim(&row) else {
                invalid += 1;
                continue;
            };
            if v["context"] != context {
                continue;
            }
            let reported = v["reported_unix"].as_i64().unwrap();
            let fresh = row.observed_unix <= now && reported <= now && now - reported <= max_age;
            let active = v["status"] == "active";
            if active && fresh {
                assertions.insert(v["assertion"].as_str().unwrap().to_owned());
                lineage.insert(v["lineage"].as_str().unwrap().to_owned());
            } else if active {
                stale += 1;
            }
            claims.push(json!({"id":row.id,"source_id":row.source_id,"source_url":row.source_url,
                "observed_unix":row.observed_unix,"game_build":row.applicable_build,"mod_version":row.mod_version,
                "completeness":row.completeness,"fresh":fresh,"claim":v}));
        }
        let conflict = assertions.contains("works")
            && (assertions.contains("broken") || assertions.contains("warning"));
        let state = if conflict {
            "conflicting"
        } else if !assertions.is_empty() {
            "partial_evidence"
        } else if stale > 0 {
            "stale"
        } else {
            "unknown"
        };
        Ok(
            json!({"entity_id":entity,"game_build":build,"mod_version":version,"context":context,
            "as_of_unix":now,"max_age_seconds":max_age,"state":state,"compatibility_verdict":"not_determined",
            "distinct_report_lineages":lineage.len(),"invalid_claims":invalid,"claims":claims,
            "coverage":"source_bounded_not_exhaustive","authority_note":"Official release authority is not proof of compatibility; lineages are adapter-reviewed, not votes."}),
        )
    }
}
