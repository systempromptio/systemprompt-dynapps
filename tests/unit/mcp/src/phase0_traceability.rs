//! The Phase 0 requirement traceability gate.
//!
//! Every requirement id in
//! `tests/fixtures/project-context/PHASE0-REQUIREMENTS.md` must have a row in
//! `TRACEABILITY.md` beside it; every test that row cites must exist as a `fn`
//! under `tests/**/src/`; and a `BLOCKED` row must say who or what blocks it. A
//! requirement is "confirmed done" here only if a named test proves it — a row
//! can cite nothing that does not compile.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("tests/fixtures/project-context").is_dir() && dir.join("tests").is_dir() {
            return dir;
        }
        assert!(
            dir.pop(),
            "repository root not found above CARGO_MANIFEST_DIR"
        );
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn is_id_char(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-'
}

// Every BR-n / FR-XXX-n / NFR-XXX-n token in the text.
fn requirement_ids(text: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if is_id_char(bytes[i]) && (i == 0 || !is_id_char(bytes[i - 1])) {
            let mut j = i;
            while j < bytes.len() && is_id_char(bytes[j]) {
                j += 1;
            }
            let token: String = bytes[i..j].iter().collect();
            let mut parts = token.split('-');
            let head = parts.next().unwrap_or_default();
            let tail: Vec<&str> = parts.collect();
            let well_formed = match head {
                "BR" => tail.len() == 1,
                "FR" | "NFR" => tail.len() == 2 && tail[0].chars().all(|c| c.is_ascii_uppercase()),
                _ => false,
            } && tail
                .last()
                .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
            if well_formed {
                ids.insert(token);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    ids
}

struct Row {
    status: String,
    tests: Vec<String>,
}

fn traceability_rows(text: &str) -> BTreeMap<String, Row> {
    let mut rows = BTreeMap::new();
    for line in text.lines().filter(|l| l.starts_with("| ")) {
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() != 4 || cells[0] == "ID" || cells[0].starts_with("---") {
            continue;
        }
        let tests = cells[3]
            .split('`')
            .skip(1)
            .step_by(2)
            .map(str::to_owned)
            .filter(|t| !t.is_empty())
            .collect();
        rows.insert(
            cells[0].to_owned(),
            Row {
                status: cells[1].to_owned(),
                tests,
            },
        );
    }
    rows
}

fn test_functions(root: &Path) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut stack = vec![root.join("tests")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                for line in read(&path).lines() {
                    let line = line.trim_start();
                    let Some(rest) = line
                        .strip_prefix("async fn ")
                        .or_else(|| line.strip_prefix("fn "))
                    else {
                        continue;
                    };
                    if let Some(name) = rest.split('(').next() {
                        names.insert(name.trim().to_owned());
                    }
                }
            }
        }
    }
    names
}

#[test]
fn every_phase0_requirement_has_a_traceability_row_citing_a_real_test() {
    let root = repo_root();
    let pack = root.join("tests/fixtures/project-context");
    let required = requirement_ids(&read(&pack.join("PHASE0-REQUIREMENTS.md")));
    assert!(
        required.len() >= 50,
        "the requirements doc lists {} ids",
        required.len()
    );
    let rows = traceability_rows(&read(&pack.join("TRACEABILITY.md")));
    let known_tests = test_functions(&root);

    let mut problems = Vec::new();
    for id in &required {
        let Some(row) = rows.get(id) else {
            problems.push(format!("{id}: no row in TRACEABILITY.md"));
            continue;
        };
        let status = row.status.as_str();
        let recognised = status == "DONE"
            || status == "DESIGN"
            || status
                .strip_prefix("BLOCKED:")
                .is_some_and(|reason| reason.trim().len() > 10);
        if !recognised {
            problems.push(format!(
                "{id}: status must be DONE, DESIGN or BLOCKED: <reason>, got '{status}'"
            ));
        }
        if row.tests.is_empty() {
            problems.push(format!("{id}: cites no test"));
        }
        for test in &row.tests {
            if !known_tests.contains(test) {
                problems.push(format!(
                    "{id}: cites `{test}`, which is not a test function"
                ));
            }
        }
    }
    for id in rows.keys() {
        if !required.contains(id) {
            problems.push(format!("{id}: row without a requirement"));
        }
    }
    assert!(
        problems.is_empty(),
        "traceability drift:\n{}",
        problems.join("\n")
    );
}

#[test]
fn the_id_scanner_reads_bold_table_cells_and_ignores_prose_lookalikes() {
    let ids = requirement_ids("| **FR-RET-3** | see O-2 and REQ-052; NFR-COST-1 | BR-7 | FR-X |");
    let expected: BTreeSet<String> = ["FR-RET-3", "NFR-COST-1", "BR-7"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert_eq!(ids, expected);
}
