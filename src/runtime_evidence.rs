//! Scoped read-only native runtime evidence. No detection or installation.
use crate::{Result, Store, observation_from_row, public_url};
use rusqlite::params;
use serde_json::{Value, json};

fn build_parts(value: &str) -> Option<Vec<u32>> {
    if value.len() > 43 {
        return None;
    }
    let parts: Vec<_> = value.split('.').collect();
    if !(3..=4).contains(&parts.len()) {
        return None;
    }
    let mut numbers: Vec<_> = parts
        .iter()
        .map(|part| {
            if part.is_empty()
                || (part.len() > 1 && part.starts_with('0'))
                || !part.bytes().all(|b| b.is_ascii_digit())
            {
                return None;
            }
            part.parse::<u32>().ok()
        })
        .collect::<Option<_>>()?;
    // Bethesda/Silverlock display the final zero in some sources only.
    if numbers.len() == 4 && numbers[3] == 0 {
        numbers.pop();
    }
    Some(numbers)
}

impl Store {
    /// Exact reviewed Skyrim tuples; no fallback from one storefront to another.
    pub fn skyrim_runtime_evidence(
        &self,
        game: &str,
        storefront: &str,
        runtime: &str,
    ) -> Result<Value> {
        let known_lane = match game {
            "skyrim-se" => matches!(
                storefront,
                "Steam"
                    | "GOG"
                    | "Legacy desktop"
                    | "Windows Store / Game Pass"
                    | "Epic Games Store"
            ),
            "skyrim-vr" => storefront == "Skyrim VR",
            _ => false,
        };
        if !known_lane {
            return Err("Expected an explicit reviewed Skyrim game/storefront lane".into());
        }
        let build =
            build_parts(runtime).ok_or("Expected an exact numeric game executable version")?;
        let mut query=self.db.prepare(
            "SELECT o.id,o.entity_id,o.source_id,o.source_url,o.predicate,o.value,o.observed_unix,o.applicable_build,o.mod_version,o.completeness,o.policy_revision
            FROM observations o JOIN entities e ON e.id=o.entity_id JOIN sources s ON s.id=o.source_id
            WHERE e.game_id=?1 AND o.predicate IN ('runtime_match_v1','storefront_support_v1')
            AND s.can_publish=1 AND s.can_retain=1 AND s.policy_revision=o.policy_revision
            AND o.policy_revision='skyrim-native-runtime-primary-2026-11-25'
            AND o.source_id IN ('skse-original','skyrim-maintainer-github')
            ORDER BY o.id LIMIT 257")?;
        let rows = query.query_map(params![game], observation_from_row)?;
        let mut evidence = Vec::new();
        let mut unsupported = Vec::new();
        for (idx, row) in rows.enumerate() {
            if idx == 256 {
                return Err("Skyrim evidence exceeds scan limit; no partial result".into());
            }
            let row = row?;
            let Ok(value) = serde_json::from_str::<Value>(&row.value) else {
                continue;
            };
            if value["storefront"] != storefront || public_url(&row.source_url).is_err() {
                continue;
            }
            let official_site = row.source_id == "skse-original"
                && row.source_url == "https://skse.silverlock.org/";
            let original_release = row.source_id == "skyrim-maintainer-github"
                && storefront == "Steam"
                && row.source_url == "https://github.com/ianpatt/skse64/releases/tag/v2.2.6";
            if !official_site && !original_release {
                continue;
            }
            let scoped = match game {
                "skyrim-vr" => row.entity_id == "silverlock:sksevr",
                "skyrim-se" => row.entity_id == "silverlock:skse64",
                _ => false,
            };
            if !scoped {
                continue;
            }
            let provenance = json!({"observation_id":row.id,"source_url":row.source_url,"as_of_unix":row.observed_unix});
            if row.predicate == "storefront_support_v1"
                && official_site
                && matches!(storefront, "Windows Store / Game Pass" | "Epic Games Store")
            {
                unsupported.push(provenance);
            } else if row.predicate == "runtime_match_v1"
                && value["game_runtime"]
                    .as_str()
                    .and_then(build_parts)
                    .as_ref()
                    == Some(&build)
                && build_parts(&row.applicable_build).as_ref() == Some(&build)
                && value["skse_version"] == row.mod_version
            {
                evidence.push(json!({"skse_version":row.mod_version,"game_runtime_as_reported":value["game_runtime"],"source":provenance,
                    "historical_source":original_release,"runtime_tested_by_catalog":false}));
            }
        }
        let status = if !unsupported.is_empty() {
            "unsupported_by_source"
        } else if evidence.is_empty() {
            "no_exact_reviewed_match"
        } else {
            "exact_reviewed_match"
        };
        Ok(
            json!({"status":status,"game":game,"storefront":storefront,"query_runtime":runtime,
            "evidence":evidence,"storefront_support_evidence":unsupported,
            "scope":"Exact reviewed maintainer records only; absence is unknown. No executable detection, download, compatibility guarantee or install."}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Observation;

    #[test]
    fn exact_tuple_cannot_cross_storefront_or_change_revision_component() {
        let mut store = Store::open(":memory:").unwrap();
        store.register_game("skyrim-se", "Desktop").unwrap();
        store
            .register_entity("silverlock:skse64", "skyrim-se", "mod", "SKSE")
            .unwrap();
        let policy = "skyrim-native-runtime-primary-2026-11-25";
        let url = "https://skse.silverlock.org/";
        store
            .set_source_policy("skse-original", url, true, true, policy)
            .unwrap();
        let observation = Observation {
            id: "gog".into(),
            entity_id: "silverlock:skse64".into(),
            source_id: "skse-original".into(),
            source_url: url.into(),
            predicate: "runtime_match_v1".into(),
            value: json!({"storefront":"GOG","game_runtime":"1.6.1179","skse_version":"2.2.6"})
                .to_string(),
            observed_unix: 1,
            applicable_build: "1.6.1179".into(),
            mod_version: "2.2.6".into(),
            completeness: "partial".into(),
            policy_revision: policy.into(),
        };
        store.import(&[observation]).unwrap();
        assert_eq!(
            store
                .skyrim_runtime_evidence("skyrim-se", "GOG", "1.6.1179.0")
                .unwrap()["evidence"][0]["skse_version"],
            "2.2.6"
        );
        for (lane, v) in [
            ("Steam", "1.6.1179"),
            ("GOG", "1.6.1170"),
            ("GOG", "1.6.1179.1"),
        ] {
            assert_eq!(
                store.skyrim_runtime_evidence("skyrim-se", lane, v).unwrap()["status"],
                "no_exact_reviewed_match"
            );
        }
        assert!(
            store
                .skyrim_runtime_evidence("skyrim-se", "GOG", "1.6.x")
                .is_err()
        );
        store
            .set_source_policy("skse-original", url, true, false, policy)
            .unwrap();
        assert_eq!(
            store
                .skyrim_runtime_evidence("skyrim-se", "GOG", "1.6.1179")
                .unwrap()["status"],
            "no_exact_reviewed_match"
        );
    }
}
