//! Read-only local catalog management. Never merges identities or mutates snapshots.
use crate::{Result, Store};
use rusqlite::params;
use serde_json::{Value, json};

const PUBLIC: &str = "s.can_retain=1 AND s.can_publish=1 AND s.policy_revision=o.policy_revision";
impl Store {
    /// Stable ID cursor, bounded page, literal substring matching (not SQL wildcards).
    /// Only entities with currently publishable observations are exposed.
    pub fn browse_catalog(&self, query: &str, after_id: &str, limit: u32) -> Result<Value> {
        if !(1..=200).contains(&limit) || query.len() > 1024 || after_id.len() > 4096 {
            return Err("Expected limit 1..200, query <=1024 bytes and cursor <=4096 bytes".into());
        }
        let sql = format!(
            "SELECT e.id,e.name,e.game_id,e.kind FROM entities e WHERE e.id>?1 AND (instr(lower(e.name),lower(?2))>0 OR instr(lower(e.id),lower(?2))>0) AND EXISTS(SELECT 1 FROM observations o JOIN sources s ON s.id=o.source_id WHERE o.entity_id=e.id AND {PUBLIC}) ORDER BY e.id LIMIT ?3"
        );
        let mut stmt = self.db.prepare(&sql)?;
        let mut rows: Vec<Value> = stmt.query_map(params![after_id,query,limit+1], |r| {
            Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"game_id":r.get::<_,String>(2)?,"kind":r.get::<_,String>(3)?}))
        })?.collect::<rusqlite::Result<_>>()?;
        let more = rows.len() > limit as usize;
        rows.truncate(limit as usize);
        let next = if more {
            rows.last().and_then(|r| r.get("id")).cloned()
        } else {
            None
        };
        Ok(
            json!({"entities":rows,"next_cursor":next,"limit":limit,"scope":"currently publishable observations only"}),
        )
    }

    /// Local storage metrics are counts, not permission to distribute the raw file.
    pub fn catalog_status(&self) -> Result<Value> {
        let count = |sql: &str| -> Result<i64> { Ok(self.db.query_row(sql, [], |r| r.get(0))?) };
        let public = count(&format!(
            "SELECT count(*) FROM observations o JOIN sources s ON s.id=o.source_id WHERE {PUBLIC}"
        ))?;
        let all = count("SELECT count(*) FROM observations")?;
        let pages = count("PRAGMA page_count")?;
        let free = count("PRAGMA freelist_count")?;
        let size = count("PRAGMA page_size")?;
        Ok(
            json!({"schema":1,"entities":count("SELECT count(*) FROM entities")?,"sources":count("SELECT count(*) FROM sources")?,"observations":all,"public_observations":public,"withheld_observations":all-public,"allocated_page_bytes":pages*size,"reclaimable_page_bytes_estimate":free*size,"page_size":size,"note":"Page metrics exclude WAL/SHM and compression. No VACUUM, deletion or automatic optimization performed."}),
        )
    }

    pub fn catalog_integrity(&self) -> Result<Value> {
        let mut stmt = self.db.prepare("PRAGMA integrity_check(100)")?;
        let messages: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        let fk: i64 =
            self.db
                .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
                    r.get(0)
                })?;
        Ok(
            json!({"ok":messages==["ok"] && fk==0,"integrity_messages":messages,"foreign_key_violations":fk,"scope":"SQLite structural integrity, not factual correctness or compatibility"}),
        )
    }

    /// Same-name candidates are NOT verified duplicate identities. Preserve all rows.
    pub fn duplicate_candidates(&self) -> Result<Value> {
        let sql = format!(
            "SELECT e.game_id,e.kind,lower(trim(e.name)),count(*) FROM entities e WHERE trim(e.name)<>'' AND EXISTS(SELECT 1 FROM observations o JOIN sources s ON s.id=o.source_id WHERE o.entity_id=e.id AND {PUBLIC}) GROUP BY e.game_id,e.kind,lower(trim(e.name)) HAVING count(*)>1 ORDER BY count(*) DESC,e.game_id,e.kind,lower(trim(e.name)) LIMIT 101"
        );
        let mut stmt = self.db.prepare(&sql)?;
        let mut rows:Vec<Value> = stmt.query_map([], |r|Ok(json!({"game_id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"normalized_name":r.get::<_,String>(2)?,"candidate_count":r.get::<_,i64>(3)?})))?.collect::<rusqlite::Result<_>>()?;
        let truncated = rows.len() > 100;
        rows.truncate(100);
        Ok(
            json!({"candidates":rows,"truncated":truncated,"note":"ASCII case-folded same-name hints within game/kind only; no merge, identity verdict or evidence deletion."}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Observation;
    fn fixture() -> Store {
        let mut s = Store::open(":memory:").unwrap();
        s.register_game("fixture", "Fixture").unwrap();
        s.set_source_policy("fixture", "https://example.org/terms", true, true, "v1")
            .unwrap();
        for id in ["a", "b", "c"] {
            s.register_entity(
                id,
                "fixture",
                "mod",
                if id == "c" { "100%" } else { "Same" },
            )
            .unwrap();
            s.import(&[Observation {
                id: id.into(),
                entity_id: id.into(),
                source_id: "fixture".into(),
                source_url: "https://example.org/item".into(),
                predicate: "name".into(),
                value: "fixture".into(),
                observed_unix: 1,
                applicable_build: "unknown".into(),
                mod_version: "unknown".into(),
                completeness: "unknown".into(),
                policy_revision: "v1".into(),
            }])
            .unwrap();
        }
        s
    }
    #[test]
    fn browse_cursor_literal_search_and_bounds() {
        let s = fixture();
        let first = s.browse_catalog("", "", 2).unwrap();
        assert_eq!(first["next_cursor"], "b");
        let last = s.browse_catalog("", "b", 2).unwrap();
        assert_eq!(last["entities"].as_array().unwrap().len(), 1);
        assert!(last["next_cursor"].is_null());
        assert_eq!(
            s.browse_catalog("%", "", 20).unwrap()["entities"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            s.browse_catalog("' OR 1=1--", "", 20).unwrap()["entities"],
            json!([])
        );
        assert!(s.browse_catalog("", "", 201).is_err());
        assert!(s.browse_catalog("", "", 0).is_err());
    }
    #[test]
    fn policy_withdrawal_hides_browse_and_candidates_not_history() {
        let s = fixture();
        assert_eq!(
            s.duplicate_candidates().unwrap()["candidates"][0]["candidate_count"],
            2
        );
        s.set_source_policy("fixture", "https://example.org/terms", true, false, "v1")
            .unwrap();
        assert_eq!(s.browse_catalog("", "", 20).unwrap()["entities"], json!([]));
        assert_eq!(s.duplicate_candidates().unwrap()["candidates"], json!([]));
        assert_eq!(s.catalog_status().unwrap()["withheld_observations"], 3);
        assert_eq!(s.catalog_integrity().unwrap()["ok"], true);
    }
}
