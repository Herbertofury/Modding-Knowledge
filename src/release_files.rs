//! Read-only identity projection for the reviewed Skyrim GitHub release sidecars.
use crate::{Observation, Result, Store, observation_from_row};
use rusqlite::params;
use serde_json::{Value, json};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Write,
};

const LIMIT: usize = 128;
const PREDICATE: &str = "project_release_evidence_v1";
fn component(label: &str, max: usize) -> bool {
    !label.is_empty()
        && label.len() <= max
        && label.as_bytes()[0].is_ascii_alphanumeric()
        && label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}
fn repository_for(entity: &str) -> Option<&str> {
    let name = match entity {
        "silverlock:skse64" => "ianpatt/skse64",
        "nexus:skyrim-se:project:58101" => "alandtse/skyrim_vr_address_library",
        _ => entity.strip_prefix("github:")?,
    };
    let (owner, repo) = name.split_once('/')?;
    (component(owner, 100) && component(repo, 100)).then_some(name)
}
fn sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn project_evidence(row: &Observation, repository: &str) -> Result<Value> {
    let v: Value = serde_json::from_str(&row.value)?;
    let tag = v["tag"].as_str().ok_or("Release tag missing")?;
    if v["repository"] != repository
        || !component(tag, 100)
        || row.mod_version != tag
        || row.source_url != format!("https://github.com/{repository}/releases/tag/{tag}")
    {
        return Err("Release project, tag and source URL did not agree".into());
    }
    let entries = v["files"].as_array().ok_or("Missing reviewed file list")?;
    if entries.len() > 96 {
        return Err("Release evidence exceeds bounded file limit".into());
    }
    let mut assets = Vec::new();
    let mut ids = HashSet::new();
    for f in entries {
        let id = f["provider_asset_id"]
            .as_u64()
            .filter(|x| *x != 0)
            .ok_or("Invalid file identity")?;
        let name = f["name"]
            .as_str()
            .filter(|x| component(x, 200))
            .ok_or("Unsupported asset label")?;
        let size = f["size"]
            .as_u64()
            .filter(|n| *n != 0)
            .ok_or("Invalid reported byte size")?;
        if !ids.insert(id) {
            return Err("Duplicate provider asset ID within one release".into());
        }
        let digest = match &f["provider_sha256"] {
            Value::Null => None,
            Value::String(value) if sha256(value) => Some(value),
            _ => return Err("Invalid provider digest".into()),
        };
        let recorded_url = f["official_download_url"].as_str().filter(|v| {
            *v == format!("https://github.com/{repository}/releases/download/{tag}/{name}")
        });
        assets.push(json!({
            "provider_asset_id":id,"name":name,"bytes_reported":size,"provider_sha256":digest,
            "official_download_url":recorded_url,"role_as_recorded":f["role"],
            "archive_bytes_verified":false,
        }));
    }
    Ok(json!({
        "tag":tag,"provider_release_id":v["provider_release_id"].as_u64(),
        "release_page":row.source_url,"published_at_as_recorded":v["published_at"],
        "release_type_as_recorded":v["release_type"],"provider_prerelease_flag":v["provider_prerelease_flag"],
        "files":assets,"observation_id":row.id,"observed_unix":row.observed_unix,
    }))
}
impl Store {
    /// Exact publisher file IDs. Does not derive unrecorded or mismatched file links.
    pub fn skyrim_release_files(&self, entity: &str) -> Result<Value> {
        let repository = repository_for(entity)
            .ok_or("Expected an exact reviewed GitHub project ID or known Skyrim alias")?;
        let mut q = self.db.prepare(
            "SELECT o.id,o.entity_id,o.source_id,o.source_url,o.predicate,
              CASE WHEN length(CAST(o.value AS BLOB))<=65536 THEN o.value ELSE '' END,
              o.observed_unix,o.applicable_build,o.mod_version,o.completeness,o.policy_revision
             FROM observations o JOIN sources s ON s.id=o.source_id JOIN entities e ON e.id=o.entity_id
             WHERE o.entity_id=?1 AND o.predicate=?2 AND e.game_id IN ('skyrim-se','skyrim-vr')
              AND s.can_retain=1 AND s.can_publish=1 AND s.policy_revision=o.policy_revision
              AND ((o.source_id='skyrim-maintainer-github' AND o.policy_revision='skyrim-native-runtime-primary-2026-11-25')
              OR (o.source_id='skyrim-distributor-authors' AND o.policy_revision='skyrim-distributor-release-evidence-2026-11-25'))
             ORDER BY o.observed_unix DESC,o.id LIMIT ?3",
        )?;
        let mut releases = Vec::new();
        let mut invalid = 0;
        for (i, row) in q
            .query_map(
                params![entity, PREDICATE, (LIMIT + 1) as i64],
                observation_from_row,
            )?
            .enumerate()
        {
            if i == LIMIT {
                return Err(
                    "Review query exceeds bounded release limit; no partial receipt".into(),
                );
            }
            match project_evidence(&row?, repository) {
                Ok(release) => releases.push(release),
                Err(_) => invalid += 1,
            }
        }
        releases.sort_by(|a, b| {
            b["published_at_as_recorded"]
                .as_str()
                .cmp(&a["published_at_as_recorded"].as_str())
                .then_with(|| a["tag"].as_str().cmp(&b["tag"].as_str()))
        });
        let files = releases
            .iter()
            .filter_map(|r| r["files"].as_array())
            .map(Vec::len)
            .sum::<usize>();
        Ok(json!({
            "entity_id":entity,"repository":repository,"releases":releases,"file_count":files,
            "invalid_reviewed_rows":invalid,
            "scope":"Reviewed file identity only. Missing is unknown. No archive verification, compatibility claim, download, or install."
        }))
    }
}

fn exact_release<'a>(report: &'a Value, tag: &str) -> Result<&'a Value> {
    let releases = report["releases"]
        .as_array()
        .ok_or("Missing reviewed releases")?;
    let mut matches = releases.iter().filter(|row| row["tag"] == tag);
    let result = matches
        .next()
        .ok_or("Exact release tag was not found in retained observations")?;
    if matches.next().is_some() {
        return Err(
            "Multiple observations share this tag; review the source before comparing".into(),
        );
    }
    Ok(result)
}

impl Store {
    /// Compare retained metadata for two explicit tags, without inferring replacement files.
    pub fn compare_skyrim_release_files(
        &self,
        entity: &str,
        first_tag: &str,
        second_tag: &str,
    ) -> Result<Value> {
        if !component(first_tag, 100) || !component(second_tag, 100) || first_tag == second_tag {
            return Err("Choose two distinct exact release tags".into());
        }
        let report = self.skyrim_release_files(entity)?;
        if report["invalid_reviewed_rows"] != 0 {
            return Err("Invalid retained rows require review before comparison".into());
        }
        let first = exact_release(&report, first_tag)?;
        let second = exact_release(&report, second_tag)?;
        let before = first["files"].as_array().ok_or("Missing reference files")?;
        let after = second["files"].as_array().ok_or("Missing compared files")?;
        let (mut ids, mut names, mut digests) = (HashMap::new(), HashMap::new(), HashMap::new());
        for (index, file) in after.iter().enumerate() {
            ids.insert(file["provider_asset_id"].as_u64(), index);
            names
                .entry(file["name"].as_str())
                .or_insert_with(Vec::new)
                .push(index);
            if let Some(digest) = file["provider_sha256"].as_str() {
                digests.entry(digest).or_insert_with(Vec::new).push(index);
            }
        }
        let (mut pairs, mut reference_without_candidate, mut matched_second) =
            (Vec::new(), Vec::new(), HashSet::new());
        for file in before {
            let first_id = file["provider_asset_id"].as_u64();
            let a_digest = file["provider_sha256"].as_str();
            let mut candidates = BTreeSet::<usize>::new();
            candidates.extend(ids.get(&first_id));
            if let Some(indices) = names.get(&file["name"].as_str()) {
                candidates.extend(indices);
            }
            if let Some(indices) = a_digest.and_then(|digest| digests.get(digest)) {
                candidates.extend(indices);
            }
            if candidates.is_empty() {
                reference_without_candidate.push(first_id);
            }
            for index in candidates {
                let other = &after[index];
                let b_digest = other["provider_sha256"].as_str();
                let digest_observation = match (a_digest, b_digest) {
                    (Some(a), Some(b)) if a == b => "reported_sha256_equal",
                    (Some(_), Some(_)) => "reported_sha256_different",
                    _ => "reported_sha256_unavailable",
                };
                matched_second.insert(index);
                pairs.push(json!({
                    "reference_asset_id":first_id,
                    "compared_asset_id":other["provider_asset_id"],
                    "same_provider_asset_id":file["provider_asset_id"]==other["provider_asset_id"],
                    "same_exact_filename":file["name"]==other["name"],
                    "reported_digest_observation":digest_observation,
                }));
            }
        }
        let compared_without_candidate = after
            .iter()
            .enumerate()
            .filter(|(index, _)| !matched_second.contains(index))
            .map(|(_, equality)| &equality["provider_asset_id"])
            .collect::<Vec<_>>();
        Ok(json!({
            "entity_id":entity,"repository":report["repository"],
            "reference":first,"compared":second,"metadata_pairs":pairs,
            "reference_without_metadata_pair":reference_without_candidate,
            "compared_without_metadata_pair":compared_without_candidate,
            "scope":"Exact retained observations only. Filename equality is not file identity. Reported hashes were not verified against archive bytes. No replacement, compatibility, availability, or install verdict."
        }))
    }
}

fn escape(value: &Value) -> String {
    value
        .as_str()
        .unwrap_or("Unknown")
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
pub fn html(report: &Value) -> Result<String> {
    let mut s = String::from(
        "<!doctype html><html lang=en><meta charset=utf-8><meta name=viewport content=\"width=device-width,initial-scale=1\"><meta name=referrer content=no-referrer><meta http-equiv=Content-Security-Policy content=\"default-src 'none';style-src 'unsafe-inline';base-uri 'none';form-action 'none'\"><title>Reviewed Skyrim release files</title><style>:root{color-scheme:dark;background:#000;color:#eee;font:16px/1.55 system-ui}main{max-width:940px;margin:auto;padding:24px}a{color:#b4d9fd}article{border:1px solid #555;border-radius:12px;padding:16px;margin:16px 0;overflow-wrap:anywhere}code{display:block;overflow-wrap:anywhere}small{color:#bbb}</style><main>",
    );
    write!(
        s,
        "<h1>{}</h1><p>{} file records · {} malformed reviewed rows omitted</p><p>{}</p>",
        escape(&report["repository"]),
        report["file_count"],
        report["invalid_reviewed_rows"],
        escape(&report["scope"])
    )?;
    for release in report["releases"].as_array().ok_or("Missing releases")? {
        write!(
            s,
            "<article><h2><a href=\"{}\">{}</a></h2><p>{} · {}</p><ul>",
            escape(&release["release_page"]),
            escape(&release["tag"]),
            escape(&release["release_type_as_recorded"]),
            escape(&release["published_at_as_recorded"])
        )?;
        for file in release["files"].as_array().ok_or("Missing files")? {
            write!(
                s,
                "<li><strong>{}</strong> · Asset #{} · {} bytes<code>{}</code><small>{}</small>",
                escape(&file["name"]),
                file["provider_asset_id"],
                file["bytes_reported"],
                escape(&file["provider_sha256"]),
                escape(&file["role_as_recorded"])
            )?;
            if !file["official_download_url"].is_null() {
                write!(
                    s,
                    " <a href=\"{}\">Publisher download</a>",
                    escape(&file["official_download_url"])
                )?;
            }
            s.push_str("</li>");
        }
        s.push_str("</ul></article>");
    }
    s.push_str("</main></html>");
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn evidence(id: u64, tag: &str, link: Option<&str>) -> Observation {
        Observation {
            id: format!("sample:{id}"),
            entity_id: "github:powerof3/Keyword-Item-Distributor".into(),
            source_id: "skyrim-distributor-authors".into(),
            source_url: format!("https://github.com/powerof3/Keyword-Item-Distributor/releases/tag/{tag}"),
            predicate: PREDICATE.into(),
            value: json!({"tag":tag,"repository":"powerof3/Keyword-Item-Distributor","files":[
              {"provider_asset_id":id,"name":"FOMOD.7z","size":100,"official_download_url":link,"provider_sha256":null,"role":"<script>inert</script>"}]}).to_string(),
            observed_unix: 2,
            applicable_build: "unverified".into(),
            mod_version: tag.into(),
            completeness: "partial".into(),
            policy_revision: "skyrim-distributor-release-evidence-2026-11-25".into(),
        }
    }
    #[test]
    fn source_policy_variant_identity_and_links_are_independent() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let a = evidence(
            1,
            "v4.1.0",
            Some(
                "https://github.com/powerof3/Keyword-Item-Distributor/releases/download/v4.1.0/FOMOD.7z",
            ),
        );
        let b = evidence(
            2,
            "v4.0.6",
            Some("https://example.invalid/outside/FOMOD.7z"),
        );
        let mut bad = evidence(3, "v4.0.5", None);
        bad.source_url.push_str("-mismatch");
        store.register_game("skyrim-se", "Skyrim Special Edition")?;
        store.set_source_policy(
            &a.source_id,
            "https://github.com/powerof3",
            true,
            true,
            &a.policy_revision,
        )?;
        store.register_entity(&a.entity_id, "skyrim-se", "mod", "KID")?;
        store.import(&[evidence(1,"v4.1.0",Some("https://github.com/powerof3/Keyword-Item-Distributor/releases/download/v4.1.0/FOMOD.7z")), b, bad])?;
        let r = store.skyrim_release_files(&a.entity_id)?;
        assert_eq!(r["file_count"], 2);
        assert_eq!(r["invalid_reviewed_rows"], 1);
        let releases = r["releases"].as_array().unwrap();
        assert_eq!(
            releases
                .iter()
                .map(|r| r["files"][0]["provider_asset_id"].as_u64().unwrap())
                .collect::<HashSet<_>>(),
            HashSet::from([1, 2])
        );
        let text = html(&r)?;
        assert!(!text.contains("example.invalid"));
        assert!(!text.contains("<script>"));
        assert!(text.contains("&lt;script&gt;inert"));
        assert_eq!(text.matches("Publisher download").count(), 1);
        store.set_source_policy(
            &a.source_id,
            "https://github.com/powerof3",
            true,
            false,
            &a.policy_revision,
        )?;
        let withheld = store.skyrim_release_files(&a.entity_id)?;
        assert_eq!(withheld["file_count"], 0);
        assert_eq!(withheld["invalid_reviewed_rows"], 0);
        Ok(())
    }

    #[test]
    fn compare_keeps_identity_hash_and_name_independent_and_refuses_ambiguous_tags() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let mut a = evidence(11, "v4.0.6", None);
        let mut b = evidence(22, "v4.1.0", None);
        let identical = "f".repeat(64);
        let file = |id, name, digest| {
            json!({
                "provider_asset_id":id,"name":name,"size":100,"official_download_url":null,"provider_sha256":digest
            })
        };
        for (row, files) in [
            (
                &mut a,
                vec![
                    file(11, "Shared.7z", Value::Null),
                    file(12, "Before.7z", json!(identical)),
                    file(13, "Original.7z", Value::Null),
                    file(14, "First-Only.7z", Value::Null),
                ],
            ),
            (
                &mut b,
                vec![
                    file(22, "Shared.7z", Value::Null),
                    file(23, "After.7z", json!(identical)),
                    file(13, "Renamed.7z", Value::Null),
                    file(25, "Second-Only.7z", Value::Null),
                ],
            ),
        ] {
            let mut body: Value = serde_json::from_str(&row.value)?;
            body["files"] = json!(files);
            row.value = body.to_string();
        }
        store.register_game("skyrim-se", "Skyrim Special Edition")?;
        store.set_source_policy(
            &a.source_id,
            "https://github.com/powerof3",
            true,
            true,
            &a.policy_revision,
        )?;
        store.register_entity(&a.entity_id, "skyrim-se", "mod", "KID")?;
        store.import(std::slice::from_ref(&a))?;
        store.import(std::slice::from_ref(&b))?;
        let r = store.compare_skyrim_release_files(&a.entity_id, "v4.0.6", "v4.1.0")?;
        assert_eq!(r["reference_without_metadata_pair"], json!([14]));
        assert_eq!(r["compared_without_metadata_pair"], json!([25]));
        let rows = r["metadata_pairs"].as_array().unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["same_exact_filename"], true);
        assert_eq!(rows[0]["same_provider_asset_id"], false);
        assert_eq!(
            rows[0]["reported_digest_observation"],
            "reported_sha256_unavailable"
        );
        assert_eq!(rows[1]["same_exact_filename"], false);
        assert_eq!(rows[1]["same_provider_asset_id"], false);
        assert_eq!(
            rows[1]["reported_digest_observation"],
            "reported_sha256_equal"
        );
        assert_eq!(rows[2]["same_exact_filename"], false);
        assert_eq!(rows[2]["same_provider_asset_id"], true);
        assert!(
            store
                .compare_skyrim_release_files(&a.entity_id, "v4.1.0", "v4.1.0")
                .is_err()
        );
        assert!(
            store
                .compare_skyrim_release_files(&a.entity_id, "v4.1.0", "v4")
                .is_err()
        );
        b.id = "duplicate".into();
        store.import(&[b])?;
        assert!(
            store
                .compare_skyrim_release_files(&a.entity_id, "v4.0.6", "v4.1.0")
                .is_err()
        );
        Ok(())
    }
}
