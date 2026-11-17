//! Read-only lifecycle evidence. Adapters verify authorship; this module never fetches,
//! changes a source, downloads, installs, or infers abandonment from silence.
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dimension {
    Availability,
    ProjectSupport,
    CreatorActivity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Available,
    Missing,
    AccessBlocked,
    RateLimited,
    RetrievalFailed,
    Removed,
    Deleted,
    Supported,
    Discontinued,
    Paused,
    ActivityObserved,
    InactiveDeclared,
}
impl State {
    fn dimension(self) -> Dimension {
        match self {
            Self::Supported | Self::Discontinued | Self::Paused => Dimension::ProjectSupport,
            Self::ActivityObserved | Self::InactiveDeclared => Dimension::CreatorActivity,
            _ => Dimension::Availability,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Authority {
    Retrieval,
    ProviderStatement,
    CreatorStatement,
    VerifiedPublicActivity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Active,
    Retracted,
    Superseded,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub id: String,
    /// Exact provider project URL for Availability; stable project/creator ID otherwise.
    pub subject: String,
    pub state: State,
    pub authority: Authority,
    pub source_url: String,
    /// Original statement/activity time when known, never replaced by fetch time.
    pub event_unix: Option<i64>,
    pub observed_unix: i64,
    pub review_after_unix: i64,
    pub lifecycle: Lifecycle,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Projection<'a> {
    /// None means unknown, stale-only, or conflict, never "abandoned".
    pub current: Option<State>,
    pub conflict: bool,
    pub has_stale_evidence: bool,
    pub last_checked_unix: Option<i64>,
    /// Full scoped history including withdrawn, superseded and stale evidence.
    pub history: Vec<&'a Evidence>,
    pub current_evidence: Vec<&'a Evidence>,
}
fn validate(e: &Evidence, now: i64) -> crate::Result<()> {
    if e.id.trim().is_empty()
        || e.subject.trim().is_empty()
        || e.observed_unix < 0
        || e.observed_unix > now
        || e.review_after_unix <= e.observed_unix
    {
        return Err("Invalid identity or evidence timestamps".into());
    }
    if e.event_unix.is_some_and(|t| t < 0 || t > e.observed_unix)
        || (e.state == State::ActivityObserved && e.event_unix.is_none())
    {
        return Err("Invalid or missing activity/event timestamp".into());
    }
    crate::public_url(&e.source_url)?;
    if e.state.dimension() == Dimension::Availability {
        crate::public_url(&e.subject)?;
    }
    let valid = match e.state {
        State::Available
        | State::Missing
        | State::AccessBlocked
        | State::RateLimited
        | State::RetrievalFailed => e.authority == Authority::Retrieval,
        State::Deleted | State::Removed => matches!(
            e.authority,
            Authority::ProviderStatement | Authority::CreatorStatement
        ),
        State::Supported | State::Discontinued | State::Paused | State::InactiveDeclared => {
            e.authority == Authority::CreatorStatement
        }
        State::ActivityObserved => e.authority == Authority::VerifiedPublicActivity,
    };
    if !valid {
        return Err("Authority cannot establish this lifecycle state".into());
    }
    Ok(())
}
/// Conflicting fresh active states remain unresolved until an adapter records an
/// explicit correction/retraction/supersession. Newest timestamp alone cannot win.
/// Availability is source-specific, not a claim that a mod vanished everywhere.
pub fn project<'a>(
    subject: &str,
    dimension: Dimension,
    evidence: &'a [Evidence],
    now: i64,
) -> crate::Result<Projection<'a>> {
    if now < 0 || subject.trim().is_empty() {
        return Err("Invalid query".into());
    }
    if dimension == Dimension::Availability {
        crate::public_url(subject)?;
    }
    let mut ids = HashSet::new();
    for e in evidence {
        validate(e, now)?;
        if !ids.insert(&e.id) {
            return Err("Duplicate evidence ID".into());
        }
    }
    let mut history: Vec<_> = evidence
        .iter()
        .filter(|e| e.subject == subject && e.state.dimension() == dimension)
        .collect();
    history.sort_by(|a, b| a.observed_unix.cmp(&b.observed_unix).then(a.id.cmp(&b.id)));
    let current_evidence: Vec<_> = history
        .iter()
        .copied()
        .filter(|e| e.lifecycle == Lifecycle::Active && now < e.review_after_unix)
        .collect();
    let first = current_evidence.first().map(|e| e.state);
    let conflict = current_evidence.iter().any(|e| Some(e.state) != first);
    let stale = history
        .iter()
        .any(|e| e.lifecycle == Lifecycle::Active && now >= e.review_after_unix);
    Ok(Projection {
        current: if conflict { None } else { first },
        conflict,
        has_stale_evidence: stale,
        last_checked_unix: history.last().map(|e| e.observed_unix),
        history,
        current_evidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn e(state: State, authority: Authority) -> Evidence {
        Evidence {
            id: "observation:1".into(),
            subject: "https://mods.example/project/1".into(),
            state,
            authority,
            source_url: "https://mods.example/notice/1".into(),
            event_unix: Some(90),
            observed_unix: 100,
            review_after_unix: 200,
            lifecycle: Lifecycle::Active,
        }
    }
    fn p(es: &[Evidence], d: Dimension, now: i64) -> crate::Result<Projection<'_>> {
        project("https://mods.example/project/1", d, es, now)
    }
    #[test]
    fn missing_is_not_deleted() {
        let es = [e(State::Missing, Authority::Retrieval)];
        assert_eq!(
            p(&es, Dimension::Availability, 150).unwrap().current,
            Some(State::Missing)
        );
        assert!(
            p(
                &[e(State::Deleted, Authority::Retrieval)],
                Dimension::Availability,
                150
            )
            .is_err()
        );
    }
    #[test]
    fn errors_preserve_history_not_abandonment() {
        for s in [
            State::AccessBlocked,
            State::RateLimited,
            State::RetrievalFailed,
        ] {
            let es = [e(s, Authority::Retrieval)];
            assert_eq!(
                p(&es, Dimension::Availability, 150).unwrap().current,
                Some(s)
            );
            assert_eq!(
                p(&es, Dimension::ProjectSupport, 150).unwrap().current,
                None
            );
        }
    }
    #[test]
    fn activity_does_not_prove_mod_support() {
        let es = [e(
            State::ActivityObserved,
            Authority::VerifiedPublicActivity,
        )];
        assert_eq!(
            p(&es, Dimension::ProjectSupport, 150).unwrap().current,
            None
        );
        assert!(
            p(
                &[e(State::Discontinued, Authority::VerifiedPublicActivity)],
                Dimension::ProjectSupport,
                150
            )
            .is_err()
        );
    }
    #[test]
    fn explicit_support_and_inactivity_require_creator() {
        for s in [
            State::Supported,
            State::Paused,
            State::Discontinued,
            State::InactiveDeclared,
        ] {
            assert!(p(&[e(s, Authority::ProviderStatement)], s.dimension(), 150).is_err());
            assert_eq!(
                p(&[e(s, Authority::CreatorStatement)], s.dimension(), 150)
                    .unwrap()
                    .current,
                Some(s)
            );
        }
    }
    #[test]
    fn stale_is_not_current() {
        let es = [e(State::Discontinued, Authority::CreatorStatement)];
        let r = p(&es, Dimension::ProjectSupport, 200).unwrap();
        assert!(r.has_stale_evidence);
        assert_eq!(r.current, None);
        assert_eq!(r.history.len(), 1);
    }
    #[test]
    fn conflict_is_order_independent() {
        let mut a = e(State::Discontinued, Authority::CreatorStatement);
        let mut b = e(State::Supported, Authority::CreatorStatement);
        b.id = "observation:2".into();
        b.observed_unix = 110;
        for es in [vec![a.clone(), b.clone()], vec![b.clone(), a.clone()]] {
            let r = p(&es, Dimension::ProjectSupport, 150).unwrap();
            assert!(r.conflict);
            assert_eq!(r.current, None);
        }
        a.lifecycle = Lifecycle::Superseded;
        let es = [a, b];
        let r = p(&es, Dimension::ProjectSupport, 150).unwrap();
        assert_eq!(r.current, Some(State::Supported));
        assert_eq!(r.history.len(), 2);
    }
    #[test]
    fn retracted_and_other_sources_do_not_count() {
        let mut a = e(State::Deleted, Authority::ProviderStatement);
        a.lifecycle = Lifecycle::Retracted;
        let mut b = e(State::Available, Authority::Retrieval);
        b.id = "other".into();
        b.subject = "https://other.example/mod/1".into();
        let es = [a, b];
        let r = p(&es, Dimension::Availability, 150).unwrap();
        assert_eq!(r.current, None);
        assert_eq!(r.history.len(), 1);
    }
    #[test]
    fn bad_evidence_is_rejected() {
        let base = e(State::Available, Authority::Retrieval);
        for url in [
            "http://mods.example/a",
            "https://user:pass@mods.example/a",
            "https://mods.example/a?token=secret",
        ] {
            let mut a = base.clone();
            a.source_url = url.into();
            assert!(p(&[a], Dimension::Availability, 150).is_err());
        }
        assert!(p(&[base.clone(), base.clone()], Dimension::Availability, 150).is_err());
        assert!(p(&[base], Dimension::Availability, 99).is_err());
        assert!(project("", Dimension::ProjectSupport, &[], 150).is_err());
    }
    #[test]
    fn activity_date_required_and_stale_history_is_not_current_freshness() {
        let mut a = e(State::ActivityObserved, Authority::VerifiedPublicActivity);
        a.event_unix = None;
        assert!(p(&[a.clone()], Dimension::CreatorActivity, 150).is_err());
        a.event_unix = Some(151);
        assert!(p(&[a], Dimension::CreatorActivity, 150).is_err());
        let old = e(State::Supported, Authority::CreatorStatement);
        let mut fresh = old.clone();
        fresh.id = "fresh".into();
        fresh.observed_unix = 180;
        fresh.review_after_unix = 300;
        let es = [old, fresh];
        let r = p(&es, Dimension::ProjectSupport, 200).unwrap();
        assert!(r.has_stale_evidence);
        assert_eq!(r.current, Some(State::Supported));
        assert_eq!(r.current_evidence.len(), 1);
    }
    #[test]
    fn no_evidence_means_unknown() {
        let r = p(&[], Dimension::ProjectSupport, 150).unwrap();
        assert_eq!(r.current, None);
        assert!(!r.has_stale_evidence);
        assert!(!r.conflict);
        assert_eq!(r.last_checked_unix, None);
    }
}
