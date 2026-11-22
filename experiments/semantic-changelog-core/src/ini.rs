//! Conservative, read-only INI lexical settings projection, not engine resolution.
//! Duplicate identities are positional occurrences; values never appear in Debug
//! or default reports. Section/key labels can also be private: keep reports local.
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub section: String,
    pub section_occurrence: usize,
    pub key: String,
    pub key_occurrence: usize,
}
pub struct Document {
    sections: Vec<String>,
    entries: Vec<Entry>,
}
pub struct Entry {
    identity: Key,
    line: usize,
    value: String,
}
impl Document {
    pub fn sections(&self) -> &[String] {
        &self.sections
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
}
impl Entry {
    pub fn identity(&self) -> &Key {
        &self.identity
    }
    pub fn line(&self) -> usize {
        self.line
    }

    /// Private local value for an explicitly authorized UI reveal, never logging.
    pub fn value(&self) -> &str {
        &self.value
    }
}
impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IniDocument")
            .field("entry_count", &self.entries.len())
            .field("values", &"redacted")
            .finish()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Change {
    pub key: Key,
    pub before_line: Option<usize>,
    pub after_line: Option<usize>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    pub changes: Vec<Change>,
    pub section_order_changed: bool,
    pub key_order_changed: bool,
    pub values_redacted: bool,
}
fn decode(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("INI input exceeds4MiB".into());
    }
    let text = if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        if !bytes.len().is_multiple_of(2) {
            return Err("truncated UTF16".into());
        }
        let le = bytes[0] == 0xff;
        let words: Vec<u16> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| {
                if le {
                    u16::from_le_bytes([p[0], p[1]])
                } else {
                    u16::from_be_bytes([p[0], p[1]])
                }
            })
            .collect();
        String::from_utf16(&words).map_err(|_| "invalid UTF16")?
    } else {
        String::from_utf8(
            bytes
                .strip_prefix(&[0xef, 0xbb, 0xbf])
                .unwrap_or(bytes)
                .to_vec(),
        )
        .map_err(|_| "unsupported encoding; require UTF8 or BOM UTF16")?
    };
    if text.contains('\0') {
        return Err("embedded NUL unsupported".into());
    }
    Ok(text)
}
pub fn parse(bytes: &[u8], cancel: impl Fn() -> bool) -> Result<Document, String> {
    if cancel() {
        return Err("cancelled".into());
    }
    let text = decode(bytes)?;
    let mut doc = Document {
        sections: vec![],
        entries: vec![],
    };
    let mut section = String::new();
    let mut occurrence = 0;
    let mut projection_bytes = 0usize;
    let mut sections: BTreeMap<String, usize> = BTreeMap::new();
    let mut keys: BTreeMap<String, usize> = BTreeMap::new();
    for (i, raw) in text.lines().enumerate() {
        if cancel() {
            return Err("cancelled".into());
        }
        if i >= 100_000 {
            return Err("INI line budget exceeded".into());
        }
        let line = raw.trim_matches([' ', '\t']);
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.len() > 65536 || line.chars().any(|c| c.is_control() && c != '\t') {
            return Err(format!("unsupported line shape at{}", i + 1));
        }
        if line.starts_with('[') {
            let name = line
                .strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .ok_or_else(|| format!("unsupported section syntax at{}", i + 1))?
                .trim_matches([' ', '\t']);
            if name.is_empty() || name.contains(['[', ']']) {
                return Err(format!("invalid section at{}", i + 1));
            }
            section = name.into();
            occurrence = *sections.entry(section.clone()).or_default();
            *sections.get_mut(&section).unwrap() += 1;
            keys.clear();
            doc.sections.push(section.clone());
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("unsupported assignment at{}", i + 1))?;
        let key = key.trim_matches([' ', '\t']);
        if key.is_empty() {
            return Err(format!("empty key at{}", i + 1));
        }
        projection_bytes = projection_bytes
            .checked_add(256 + section.len() + key.len() + value.len())
            .ok_or("INI projection size overflow")?;
        if projection_bytes > 16 * 1024 * 1024 {
            return Err("INI projection allocation budget exceeded".into());
        }
        let n = keys.entry(key.into()).or_default();
        let identity = Key {
            section: section.clone(),
            section_occurrence: occurrence,
            key: key.into(),
            key_occurrence: *n,
        };
        *n += 1;
        doc.entries.push(Entry {
            identity,
            line: i + 1,
            value: value.trim_matches([' ', '\t']).into(),
        });
    }
    Ok(doc)
}
/// Exact lexical value comparison, case sensitive; no bool/numeric coercion,
/// inline-comment stripping, interpolation or game-specific precedence inferred.
pub fn compare(a: &Document, b: &Document) -> Report {
    let old: BTreeMap<_, _> = a.entries.iter().map(|e| (&e.identity, e)).collect();
    let new: BTreeMap<_, _> = b.entries.iter().map(|e| (&e.identity, e)).collect();
    let ids: std::collections::BTreeSet<_> = old.keys().chain(new.keys()).copied().collect();
    let mut changes = vec![];
    for key in ids {
        let x = old.get(key);
        let y = new.get(key);
        if x.map(|e| e.value.as_str()) != y.map(|e| e.value.as_str()) {
            changes.push(Change {
                key: key.clone(),
                before_line: x.map(|e| e.line),
                after_line: y.map(|e| e.line),
            });
        }
    }
    Report {
        changes,
        section_order_changed: a.sections != b.sections,
        key_order_changed: a
            .entries
            .iter()
            .map(|e| &e.identity)
            .ne(b.entries.iter().map(|e| &e.identity)),
        values_redacted: true,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicates_retained() {
        let a = parse(b"[S]\nx=1\nx=2\n[S]\nx=3", || false).unwrap();
        assert_eq!(a.entries.len(), 3);
        assert_eq!(a.entries[1].identity.key_occurrence, 1);
        assert_eq!(a.entries[2].identity.section_occurrence, 1);
        let b = parse(b"[S]\nx=1\nx=4\n[S]\nx=3", || false).unwrap();
        let r = compare(&a, &b);
        assert_eq!(r.changes.len(), 1);
        assert_eq!(r.changes[0].key.key_occurrence, 1);
    }
    #[test]
    fn formatting_not_setting_change() {
        let a = parse(b"\xef\xbb\xbf[S]\r\nx = 1\r\n", || false).unwrap();
        let b = parse(b";comment\n[S]\nx=1\n", || false).unwrap();
        assert!(compare(&a, &b).changes.is_empty());
    }
    #[test]
    fn reorder_and_empty_section_visible() {
        let a = parse(b"[A]\nx=1\ny=2", || false).unwrap();
        let b = parse(b"[A]\ny=2\nx=1\n[Empty]", || false).unwrap();
        let r = compare(&a, &b);
        assert!(r.changes.is_empty());
        assert!(r.key_order_changed && r.section_order_changed);
    }
    #[test]
    fn utf16_and_invalid_encoding() {
        for le in [true, false] {
            let mut b = if le { vec![255, 254] } else { vec![254, 255] };
            for w in "[S]\nx=1".encode_utf16() {
                b.extend(if le { w.to_le_bytes() } else { w.to_be_bytes() });
            }
            assert_eq!(parse(&b, || false).unwrap().entries[0].value(), "1");
        }
        assert!(parse(&[0xff], || false).is_err());
        assert!(parse(&[255, 254, 0], || false).is_err());
    }
    #[test]
    fn malformed_and_cancelled_explicit() {
        for b in [b"[bad".as_slice(), b"keyonly", b"=x", b"x=\0"] {
            assert!(parse(b, || false).is_err());
        }
        assert!(parse(b"", || true).is_err());
    }
    #[test]
    fn values_never_in_default_debug() {
        let a = parse(b"password=super-secret-A", || false).unwrap();
        let b = parse(b"password=super-secret-B", || false).unwrap();
        assert!(!format!("{a:?}{:?}", compare(&a, &b)).contains("super-secret"));
    }
    #[test]
    fn inline_markers_literal_and_case_distinct() {
        let a = parse(b"[S]\nx=a;b#c\nX=other", || false).unwrap();
        assert_eq!(a.entries[0].value(), "a;b#c");
        assert_eq!(a.entries[1].identity.key, "X");
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn repeated_long_section_cannot_amplify_memory_unbounded() {
        let data = format!("[{}]\n{}", "s".repeat(60000), "x=1\n".repeat(300));
        assert!(
            parse(data.as_bytes(), || false)
                .unwrap_err()
                .contains("allocation budget")
        );
    }
    #[test]
    fn unicode_whitespace_is_not_silently_trimmed() {
        let a = parse("x=\u{a0}1".as_bytes(), || false).unwrap();
        assert_eq!(a.entries[0].value(), "\u{a0}1");
    }
}
