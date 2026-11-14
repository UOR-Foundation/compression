//! UC-FIG-01: the release ships no number the Prism model does not account
//! for. The check runs on this repository, and planted figures in each kind
//! of shipped source are rejected.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_shipped_sources_hold_no_figure_beyond_the_register_uc_fig_01() {
    let root = root();
    let register = figures::load(&root)
        .expect("the register reads")
        .expect("model/figures.toml exists");
    let out = figures::check(&root, Some(&register)).expect("the check runs");
    assert!(out.rejected.is_empty(), "{}", out.rejected.join("\n"));
    assert!(out.stale.is_empty(), "{}", out.stale.join("\n"));
    let kinds = |suffix: &str| out.entries.iter().any(|e| e.file.ends_with(suffix));
    assert!(
        kinds(".rs") && kinds(".js") && kinds(".lex.tex"),
        "every kind of shipped source is read"
    );
}

#[test]
fn a_planted_figure_is_rejected_in_every_kind_of_source_uc_fig_01() {
    let root = root();
    let register = figures::load(&root).unwrap().unwrap();
    for (file, kind) in [
        ("crates/uorc/src/lib.rs", "rs"),
        ("web/uorc.js", "js"),
        ("client/src/Uorc/Codec/Table.lex.tex", "lex.tex"),
    ] {
        let mut scan = figures::Scan::default();
        let text = std::fs::read_to_string(root.join(file)).unwrap();
        let planted = match kind {
            "rs" => format!("{text}\nfn planted() -> usize {{ 4093 }}\n"),
            "js" => format!("{text}\nconst PLANTED = 4093;\n"),
            _ => text.replacen("definition{", "definition{\n  name: planted,\n  parameters: [],\n  result: Nat,\n  body: (prim add 4093 2 : Nat)\n}\n\ndefinition{", 1),
        };
        match kind {
            "rs" => figures::scan_source(file, &planted, &mut scan).unwrap(),
            "js" => figures::sources::scan_js(file, &planted, &mut scan),
            _ => figures::sources::scan_lean(file, "Uorc.Codec.Table", &planted, &mut scan),
        }
        assert!(
            scan.sites.iter().any(|s| s.value == "4093"),
            "{file}: the planted figure is a site"
        );
        let listed: u64 = register
            .items
            .iter()
            .filter(|e| e.file == file)
            .filter_map(|e| e.values.get("4093"))
            .sum();
        assert_eq!(listed, 0, "{file}: the register admits no 4093");
    }
}
