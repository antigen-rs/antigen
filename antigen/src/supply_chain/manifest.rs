//! Minimal Cargo manifest reader for supply-chain witness evaluation.
//!
//! Per ADR-002 Amendment 2 ("compete where antigen cohesion serves"):
//! we hand-roll a line-based scanner rather than pull in `toml` as a
//! workspace dependency. The scanner handles the common cases needed for
//! the supply-chain audit hints — full TOML structural correctness is
//! NOT the goal (Cargo itself enforces that).
//!
//! ## Scope
//!
//! - Identify entries under `[dependencies]`, `[dev-dependencies]`,
//!   `[build-dependencies]`, and `[target.<cfg>.dependencies]` sections.
//! - Distinguish exact-pin (`=X.Y.Z`) from caret (`^X.Y.Z` or bare
//!   `X.Y.Z`), tilde (`~X.Y.Z`), wildcard (`*`), and `?` ranges.
//! - Extract the version string for table-form entries
//!   (`foo = { version = "1.0", features = [...] }`).
//!
//! ## Out of scope (use a real toml parser if needed in v0.3+)
//!
//! - Complex table-array forms
//! - Multi-line entries spanning > 1 line in odd shapes
//! - Inheritance via `workspace = true` (treated as "version not inline")
//! - Path/git dependencies (treated as non-applicable: not version-pinned)

use std::path::Path;

/// A dependency entry extracted from a Cargo manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepEntry {
    /// Crate name as it appears in `Cargo.toml`.
    pub name: String,
    /// Version specifier, if inlined. `None` for `workspace = true`,
    /// `path = "..."`, `git = "..."`, or other non-version-string forms.
    pub version: Option<String>,
    /// The section the entry was found in: `"dependencies"`,
    /// `"dev-dependencies"`, `"build-dependencies"`, or the rendered
    /// target-cfg form.
    pub section: String,
}

impl DepEntry {
    /// Returns true if the version specifier exact-pins (`=X.Y.Z`).
    /// `None` (workspace/path/git) returns true (vacuously — not a
    /// version-string entry).
    #[must_use]
    pub fn is_exact_pinned(&self) -> bool {
        self.version
            .as_ref()
            .is_none_or(|v| is_exact_pin_specifier(v.trim()))
    }

    /// Returns true if the version specifier uses wildcard (`*`) or
    /// unbounded forms.
    #[must_use]
    pub fn is_wildcard(&self) -> bool {
        matches!(self.version.as_deref().map(str::trim), Some("*" | "?"))
    }
}

/// Whether a version-string specifier is an exact pin (`=X.Y.Z`).
///
/// Per Cargo semantics, the bare form `"1.2.3"` is a caret requirement
/// (`^1.2.3`) — NOT an exact pin. Only `=1.2.3` exact-pins.
#[must_use]
pub fn is_exact_pin_specifier(s: &str) -> bool {
    s.starts_with('=')
}

/// Scan a `Cargo.toml` and return all dependency entries across
/// `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, and
/// `[target.<cfg>.<deps-table>]` sections.
///
/// IO errors return an empty list; the CLI surfaces missing-manifest
/// separately.
#[must_use]
pub fn read_manifest_deps(manifest_path: &Path) -> Vec<DepEntry> {
    let Ok(content) = std::fs::read_to_string(manifest_path) else {
        return Vec::new();
    };
    parse_manifest_deps(&content)
}

/// Parse dep entries out of a Cargo.toml-shaped string.
///
/// Exposed for unit tests; callers should prefer [`read_manifest_deps`].
#[must_use]
pub fn parse_manifest_deps(content: &str) -> Vec<DepEntry> {
    let mut entries = Vec::new();
    let mut current_section: Option<String> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header?
        if let Some(stripped) = line.strip_prefix('[') {
            if let Some(header) = stripped.strip_suffix(']') {
                current_section = section_kind(header).map(str::to_string);
                continue;
            }
        }

        // We're inside a dep section?
        let Some(section) = current_section.clone() else {
            continue;
        };

        // Skip inheritance sub-tables like `[dependencies.foo]` — those
        // appear as their own section headers in TOML; we don't follow
        // them in v0.2 (would require a real parser to do well).
        // The most common form is `name = "version"` or
        // `name = { version = "...", ... }`.
        if let Some((name, value)) = split_kv(line) {
            let version = extract_version(value);
            entries.push(DepEntry {
                name: name.to_string(),
                version,
                section: section.clone(),
            });
        }
    }

    entries
}

/// Map a `[section]` header to the dep-section kind, or `None` for
/// non-dep sections.
fn section_kind(header: &str) -> Option<&str> {
    // Bare sections
    match header {
        "dependencies" => Some("dependencies"),
        "dev-dependencies" => Some("dev-dependencies"),
        "build-dependencies" => Some("build-dependencies"),
        _ => {
            // Target-conditional forms like
            // `target.'cfg(unix)'.dependencies`
            if header.starts_with("target.") && header.contains(".dependencies") {
                Some("target-dependencies")
            } else if header.starts_with("target.") && header.contains(".dev-dependencies") {
                Some("target-dev-dependencies")
            } else if header.starts_with("target.") && header.contains(".build-dependencies") {
                Some("target-build-dependencies")
            } else {
                None
            }
        },
    }
}

/// Split `key = value` on the first `=`. Strips surrounding whitespace
/// AND trailing `# ...` comments from the value.
fn split_kv(line: &str) -> Option<(&str, &str)> {
    let idx = line.find('=')?;
    let key = line[..idx].trim();
    let raw_value = line[idx + 1..].trim();
    // Names may not contain `[`, which signals a sub-table form.
    if key.contains('[') || key.contains(']') {
        return None;
    }
    // Strip trailing `# comment` — only if the `#` is not inside a quoted
    // string. v0.2 approximation: split on `#` only when there's a
    // balanced quote pair before it.
    let value = strip_trailing_comment(raw_value);
    Some((key, value))
}

/// Strip a trailing ` # ...` comment from a value, respecting balanced
/// double-quotes. A `#` that appears inside an unbalanced pair of `"`
/// quotes is treated as content, not a comment marker.
fn strip_trailing_comment(s: &str) -> &str {
    let mut in_quote = false;
    let mut cut_at: Option<usize> = None;
    for (i, ch) in s.char_indices() {
        match ch {
            '"' => in_quote = !in_quote,
            '#' if !in_quote => {
                cut_at = Some(i);
                break;
            },
            _ => {},
        }
    }
    cut_at.map_or(s, |i| s[..i].trim_end())
}

/// Extract the version string from a dep value, handling both shorthand
/// `"X.Y.Z"` and table form `{ version = "X.Y.Z", ... }`.
///
/// Returns `None` for `workspace = true`, `path = ...`, `git = ...`, or
/// any other form that doesn't expose a version literal.
fn extract_version(value: &str) -> Option<String> {
    // Shorthand: `"X.Y.Z"` or `'X.Y.Z'`
    if let Some(v) = strip_quotes(value) {
        return Some(v.to_string());
    }

    // Table form: `{ version = "...", ... }` — scan for `version =`.
    let inner = value
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(value);

    // Crude key=value split inside the braces.
    for part in inner.split(',') {
        let part = part.trim();
        if let Some(rest) = part
            .strip_prefix("version")
            .and_then(|s| s.trim_start().strip_prefix('='))
        {
            if let Some(v) = strip_quotes(rest.trim()) {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Strip a single pair of `"..."` or `'...'` from a value, returning the
/// inner string. Returns `None` if the value isn't a quoted string.
fn strip_quotes(s: &str) -> Option<&str> {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return Some(inner);
    }
    if let Some(inner) = s.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        return Some(inner);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorthand_caret_not_exact_pinned() {
        let entries = parse_manifest_deps(
            r#"
[dependencies]
serde = "1.0"
"#,
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "serde");
        assert_eq!(entries[0].version.as_deref(), Some("1.0"));
        assert!(!entries[0].is_exact_pinned());
    }

    #[test]
    fn shorthand_exact_pinned() {
        let entries = parse_manifest_deps(
            r#"
[dependencies]
serde = "=1.0.197"
"#,
        );
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_exact_pinned());
    }

    #[test]
    fn table_form_extracts_version() {
        let entries = parse_manifest_deps(
            r#"
[dependencies]
serde = { version = "=1.0.197", features = ["derive"] }
"#,
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version.as_deref(), Some("=1.0.197"));
        assert!(entries[0].is_exact_pinned());
    }

    #[test]
    fn workspace_inheritance_treated_as_no_version() {
        let entries = parse_manifest_deps(
            r#"
[dependencies]
serde = { workspace = true, features = ["derive"] }
"#,
        );
        assert_eq!(entries.len(), 1);
        assert!(entries[0].version.is_none());
        // Vacuously exact-pinned (no version string to check)
        assert!(entries[0].is_exact_pinned());
    }

    #[test]
    fn wildcard_specifier_detected() {
        let entries = parse_manifest_deps(
            r#"
[dependencies]
loose-dep = "*"
"#,
        );
        assert!(entries[0].is_wildcard());
        assert!(!entries[0].is_exact_pinned());
    }

    #[test]
    fn dev_dependencies_section_picked_up() {
        let entries = parse_manifest_deps(
            r#"
[dependencies]
serde = "=1.0.197"

[dev-dependencies]
proptest = "1.0"
"#,
        );
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].section, "dev-dependencies");
        assert_eq!(entries[1].name, "proptest");
    }

    #[test]
    fn build_dependencies_section_picked_up() {
        let entries = parse_manifest_deps(
            r#"
[build-dependencies]
cc = "=1.0.79"
"#,
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].section, "build-dependencies");
    }

    #[test]
    fn target_cfg_dependencies_picked_up() {
        let entries = parse_manifest_deps(
            r#"
[target.'cfg(unix)'.dependencies]
libc = "=0.2.0"
"#,
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].section, "target-dependencies");
    }

    #[test]
    fn comments_and_blank_lines_skipped() {
        let entries = parse_manifest_deps(
            r#"
# This is the dep list
[dependencies]

serde = "=1.0.197"  # workspace baseline
"#,
        );
        // Note: the inline `# workspace baseline` is currently part of
        // the value — our scanner doesn't strip inline comments. The
        // version-extract code is a quoted-string scan, so trailing
        // comments after the quoted string don't affect version parsing.
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version.as_deref(), Some("=1.0.197"));
    }

    #[test]
    fn non_dep_section_ignored() {
        let entries = parse_manifest_deps(
            r#"
[package]
name = "foo"
version = "0.1.0"

[lib]
name = "foo"
"#,
        );
        assert_eq!(entries.len(), 0);
    }
}
