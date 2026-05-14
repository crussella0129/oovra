//! End-to-end pipeline tests: parse, compose compounds from atoms, compose
//! deeper compounds from compounds, decompose one level, decompose --full,
//! compare.

use std::path::Path;

use oovra::decompose::{decompose, decompose_full};
use oovra::diff::{compare, DiffReport};
use oovra::element::{parse_file, write};
use oovra::library::Library;
use oovra::migrate::migrate_library;
use oovra::render::{compose, render_text, ComposeRequest};

/// Path to the sample library shipped in this repo.
fn elements_dir() -> &'static Path {
    Path::new("elements")
}

#[test]
fn create_with_invalid_id_does_not_leave_orphan_file() {
    // Regression: Create used to fs::write before validating, so a bad id
    // left an unparseable file on disk. After the in-memory pre-validation
    // fix, no file should be written.
    use oovra::create::{scaffold, ScaffoldArgs};
    let tmp = tempdir_for_test("orphan-check");
    let result = scaffold(ScaffoldArgs {
        library_dir: tmp.clone(),
        id: "BadID".into(), // not kebab-case
        name: None,
        version: "1.0.0".into(),
        meta: String::new(),
    });
    assert!(result.is_err(), "expected scaffold to reject 'BadID'");
    let entries: Vec<_> = std::fs::read_dir(&tmp)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        entries.is_empty(),
        "expected no files written, found {:?}",
        entries.iter().map(|e| e.path()).collect::<Vec<_>>()
    );
}

#[test]
fn parse_serialize_round_trip_is_idempotent() {
    // Load each shipped order-0 element, serialize, parse, serialize again,
    // and assert the second serialization equals the first. This catches
    // serializer non-determinism and parse-write drift.
    let library = Library::load(elements_dir()).unwrap();
    for element in library.elements.values() {
        let s1 = oovra::element::serialize(element).unwrap();
        let parsed = oovra::element::parse(&s1, std::path::Path::new("<test>")).unwrap();
        let s2 = oovra::element::serialize(&parsed).unwrap();
        assert_eq!(
            s1, s2,
            "non-idempotent round-trip for '{}'",
            element.header.id
        );
    }
}

#[test]
fn compose_round_trip_preserves_recipe_and_decomposes_lossless() {
    // Compose -> serialize -> parse -> decompose -> compare each leaf to
    // original. Catches any byte drift that would break decompose.
    let library = Library::load(elements_dir()).unwrap();
    let composed = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "round-trip".into(),
        output_name: "Round Trip Test".into(),
        output_version: "1.0.0".into(),
        output_meta: "ensures byte-stable round-trip".into(),
    })
    .unwrap();

    let serialized = oovra::element::serialize(&composed).unwrap();
    let reparsed = oovra::element::parse(&serialized, std::path::Path::new("<rt>")).unwrap();

    // Header round-trips exactly.
    assert_eq!(reparsed.header.id, composed.header.id);
    assert_eq!(reparsed.header.kind, composed.header.kind);
    assert_eq!(reparsed.header.body_level, composed.header.body_level);
    assert_eq!(reparsed.header.composed_of, composed.header.composed_of);

    // Decompose recovers the original inputs byte-for-byte.
    let recovered = decompose(&reparsed).unwrap();
    for original_id in &["role-declaration", "refusal-policy-strict", "tone-direct"] {
        let original = library.get(original_id).unwrap();
        let rec = recovered
            .iter()
            .find(|e| e.header.id == *original_id)
            .unwrap();
        assert_eq!(rec.body, original.body, "body drift for {original_id}");
        assert_eq!(rec.header.name, original.header.name);
        assert_eq!(rec.header.version, original.header.version);
        assert_eq!(rec.header.meta, original.header.meta);
    }
}

#[test]
fn library_loads_five_atoms() {
    let library = Library::load(elements_dir()).unwrap();
    assert_eq!(library.len(), 5);
    for element in library.elements.values() {
        assert_eq!(
            element.header.kind,
            oovra::header::PromptElementKind::Atom,
            "{}",
            element.header.id
        );
    }
}

#[test]
fn compose_three_atoms_into_one_compound() {
    let library = Library::load(elements_dir()).unwrap();
    let req = ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "coding-agent-strict".into(),
        output_name: "Coding Agent (Strict)".into(),
        output_version: "1.0.0".into(),
        output_meta: "Three-element strict coding-agent prompt".into(),
    };
    let composed = compose(req).unwrap();
    assert_eq!(
        composed.header.kind,
        oovra::header::PromptElementKind::Compound
    );
    assert_eq!(composed.header.body_level, Some(1));
    assert_eq!(
        composed.header.composed_of.as_ref().map(|v| v.len()),
        Some(3)
    );
    // Each input's content should appear in the body.
    for id in &["role-declaration", "refusal-policy-strict", "tone-direct"] {
        assert!(
            composed.body.contains(id),
            "expected {id} to appear in body"
        );
    }
    // Body should contain the level-1 delimiters.
    assert!(composed.body.contains("~~>>"));
    assert!(composed.body.contains("~~<<"));
}

#[test]
fn compose_two_compounds_into_one_deeper_compound() {
    let library = Library::load(elements_dir()).unwrap();

    // Build two distinct compounds-of-atoms.
    let sub_a = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "subprompt-a".into(),
        output_name: "Subprompt A".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let sub_b = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("output-format-markdown".into(), None),
            ("examples-block".into(), None),
        ],
        output_id: "subprompt-b".into(),
        output_name: "Subprompt B".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Stage them into a temp directory so we can re-compose from the library
    // side. (Compose resolves inputs via Library, so they must be on disk.)
    let tmp = tempdir_for_test("deep-compound-staging");
    write(&sub_a, &tmp.join("subprompt-a.md")).unwrap();
    write(&sub_b, &tmp.join("subprompt-b.md")).unwrap();

    // Copy the atoms into the same staging dir so they are resolvable too
    // (for the body's nested decomposition later).
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, tmp.join(p.file_name().unwrap())).unwrap();
    }

    let staged_lib = Library::load(&tmp).unwrap();
    let deep = compose(ComposeRequest {
        library: &staged_lib,
        inputs: vec![("subprompt-a".into(), None), ("subprompt-b".into(), None)],
        output_id: "two-stage-prompt".into(),
        output_name: "Two-Stage Prompt".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    assert_eq!(deep.header.kind, oovra::header::PromptElementKind::Compound);
    assert_eq!(deep.header.body_level, Some(2));
    assert!(deep.body.contains("~~~>>"));
    assert!(deep.body.contains("~~~<<"));
    // Inner level-1 delimiters must be preserved verbatim.
    assert!(deep.body.contains("~~>>"));
    assert!(deep.body.contains("~~<<"));
}

#[test]
fn decompose_one_level_recovers_immediate_inputs() {
    let library = Library::load(elements_dir()).unwrap();
    let composed = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "tmp-compose".into(),
        output_name: "tmp".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    let inputs = decompose(&composed).unwrap();
    assert_eq!(inputs.len(), 3);
    assert_eq!(inputs[0].header.id, "role-declaration");
    assert_eq!(inputs[1].header.id, "refusal-policy-strict");
    assert_eq!(inputs[2].header.id, "tone-direct");

    // Every recovered input must round-trip exactly.
    let original_a = library.get("role-declaration").unwrap();
    assert_eq!(inputs[0].body, original_a.body);
    assert_eq!(inputs[0].header.version, original_a.header.version);
}

#[test]
fn decompose_full_writes_folder_tree_for_deep_compound() {
    let tmp = tempdir_for_test("decompose-full");
    let library = Library::load(elements_dir()).unwrap();

    // Build two sub-compounds.
    let sub_a = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "subprompt-a".into(),
        output_name: "Subprompt A".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let sub_b = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("output-format-markdown".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "subprompt-b".into(),
        output_name: "Subprompt B".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Stage to a temp library that contains the atoms and the two sub-compounds.
    let staging = tmp.join("staging");
    std::fs::create_dir_all(&staging).unwrap();
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, staging.join(p.file_name().unwrap())).unwrap();
    }
    write(&sub_a, &staging.join("subprompt-a.md")).unwrap();
    write(&sub_b, &staging.join("subprompt-b.md")).unwrap();

    let staged_lib = Library::load(&staging).unwrap();
    let deep = compose(ComposeRequest {
        library: &staged_lib,
        inputs: vec![("subprompt-a".into(), None), ("subprompt-b".into(), None)],
        output_id: "full-test".into(),
        output_name: "Full Test".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    let out_dir = tmp.join("out");
    let element_root = decompose_full(&deep, &out_dir).unwrap();
    assert!(
        element_root.is_dir(),
        "{} should be a directory",
        element_root.display()
    );

    // Expected structure:
    //   out/full-test/
    //     full-test.md
    //     subprompt-a/
    //       subprompt-a.md
    //       role-declaration.md
    //       refusal-policy-strict.md
    //     subprompt-b/
    //       subprompt-b.md
    //       output-format-markdown.md
    //       tone-direct.md

    assert!(element_root.join("full-test.md").is_file());
    assert!(element_root.join("subprompt-a").is_dir());
    assert!(element_root.join("subprompt-a/subprompt-a.md").is_file());
    assert!(element_root
        .join("subprompt-a/role-declaration.md")
        .is_file());
    assert!(element_root
        .join("subprompt-a/refusal-policy-strict.md")
        .is_file());
    assert!(element_root.join("subprompt-b").is_dir());
    assert!(element_root.join("subprompt-b/subprompt-b.md").is_file());
    assert!(element_root
        .join("subprompt-b/output-format-markdown.md")
        .is_file());
    assert!(element_root.join("subprompt-b/tone-direct.md").is_file());

    // Each leaf must round-trip parse cleanly with original metadata.
    let leaf = parse_file(&element_root.join("subprompt-a/role-declaration.md")).unwrap();
    let original = library.get("role-declaration").unwrap();
    assert_eq!(leaf.header.id, original.header.id);
    assert_eq!(leaf.header.version, original.header.version);
    assert_eq!(leaf.header.name, original.header.name);
    assert_eq!(leaf.header.meta, original.header.meta);
    assert_eq!(leaf.body, original.body);
}

#[test]
fn compare_structural_diff_detects_version_change() {
    let library = Library::load(elements_dir()).unwrap();

    let v1 = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "stable".into(),
        output_name: "Stable".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Stage a modified library where refusal-policy-strict is bumped.
    let tmp = tempdir_for_test("structural-diff");
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, tmp.join(p.file_name().unwrap())).unwrap();
    }
    let bumped_path = tmp.join("refusal-policy-strict.md");
    let mut bumped = parse_file(&bumped_path).unwrap();
    bumped.header.version = "1.1.0".into();
    write(&bumped, &bumped_path).unwrap();

    let bumped_lib = Library::load(&tmp).unwrap();
    let v2 = compose(ComposeRequest {
        library: &bumped_lib,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "stable".into(),
        output_name: "Stable".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    match compare(&v1, &v2).unwrap() {
        DiffReport::Structural(s) => {
            assert!(s.added.is_empty());
            assert!(s.removed.is_empty());
            assert!(s.moved.is_empty(), "no reorder, no move");
            assert_eq!(s.version_changed.len(), 1);
            assert_eq!(s.version_changed[0].id, "refusal-policy-strict");
            assert_eq!(s.version_changed[0].before_version, "1.0.0");
            assert_eq!(s.version_changed[0].after_version, "1.1.0");
        }
        DiffReport::Content(_) => panic!("expected structural diff for compounds"),
    }
}

#[test]
fn compare_detects_reorder() {
    // SPEC §6 (sequence-aware compare): swapping input order produces a
    // diff with `moved` populated and `recipes_equal = false`.
    let library = Library::load(elements_dir()).unwrap();
    let v1 = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "abc".into(),
        output_name: "abc".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let v2 = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("tone-direct".into(), None),
            ("refusal-policy-strict".into(), None),
            ("role-declaration".into(), None),
        ],
        output_id: "cba".into(),
        output_name: "cba".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    match compare(&v1, &v2).unwrap() {
        DiffReport::Structural(s) => {
            assert!(s.added.is_empty());
            assert!(s.removed.is_empty());
            assert!(s.version_changed.is_empty());
            assert!(!s.recipes_equal, "reorder is a real change");
            // role-declaration (0→2) and tone-direct (2→0) both moved.
            // refusal-policy-strict stays at position 1.
            assert_eq!(s.moved.len(), 2);
            let by_id: std::collections::HashMap<_, _> = s
                .moved
                .iter()
                .map(|m| (m.id.as_str(), (m.before_pos, m.after_pos)))
                .collect();
            assert_eq!(by_id.get("role-declaration"), Some(&(0, 2)));
            assert_eq!(by_id.get("tone-direct"), Some(&(2, 0)));
        }
        DiffReport::Content(_) => panic!("expected structural diff for compounds"),
    }
}

#[test]
fn compare_distinguishes_move_from_add_remove() {
    // SPEC §6: when an input is moved in addition to other inputs being
    // swapped in/out, the move is reported as a move (not as add+remove).
    // Setup: a -> [role, refusal, tone], b -> [tone, refusal, examples].
    //   * role: removed (pos 0)
    //   * examples: added (pos 2)
    //   * refusal: stays at pos 1 (no change)
    //   * tone: moved 2 -> 0
    let library = Library::load(elements_dir()).unwrap();
    let v1 = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "v1".into(),
        output_name: "v1".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let v2 = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("tone-direct".into(), None),
            ("refusal-policy-strict".into(), None),
            ("examples-block".into(), None),
        ],
        output_id: "v2".into(),
        output_name: "v2".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    match compare(&v1, &v2).unwrap() {
        DiffReport::Structural(s) => {
            assert_eq!(s.removed.len(), 1);
            assert_eq!(s.removed[0].input.id, "role-declaration");
            assert_eq!(s.removed[0].position, 0);
            assert_eq!(s.added.len(), 1);
            assert_eq!(s.added[0].input.id, "examples-block");
            assert_eq!(s.added[0].position, 2);
            assert!(s.version_changed.is_empty());
            assert_eq!(s.moved.len(), 1);
            assert_eq!(s.moved[0].id, "tone-direct");
            assert_eq!(s.moved[0].before_pos, 2);
            assert_eq!(s.moved[0].after_pos, 0);
            assert!(!s.recipes_equal);
        }
        DiffReport::Content(_) => panic!("expected structural diff for compounds"),
    }
}

#[test]
fn compare_reports_pure_version_change_not_as_move() {
    // SPEC §6: a version bump at the same position is reported only as
    // version_changed, not as a move.
    let library = Library::load(elements_dir()).unwrap();
    let v1 = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "stable".into(),
        output_name: "Stable".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Bump tone-direct's version in a staged library, recompose.
    let tmp = tempdir_for_test("pure-version-bump");
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, tmp.join(p.file_name().unwrap())).unwrap();
    }
    let bumped_path = tmp.join("tone-direct.md");
    let mut bumped = parse_file(&bumped_path).unwrap();
    bumped.header.version = "1.1.0".into();
    write(&bumped, &bumped_path).unwrap();
    let bumped_lib = Library::load(&tmp).unwrap();
    let v2 = compose(ComposeRequest {
        library: &bumped_lib,
        inputs: vec![
            ("role-declaration".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "stable".into(),
        output_name: "Stable".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    match compare(&v1, &v2).unwrap() {
        DiffReport::Structural(s) => {
            assert!(s.added.is_empty());
            assert!(s.removed.is_empty());
            assert!(s.moved.is_empty(), "no position change");
            assert_eq!(s.version_changed.len(), 1);
            assert_eq!(s.version_changed[0].id, "tone-direct");
            assert!(!s.recipes_equal);
        }
        DiffReport::Content(_) => panic!("expected structural diff for compounds"),
    }
}

#[test]
fn mixed_kind_compose_does_not_collide_with_inner_delimiters() {
    // Regression test for the body-delimiter escalation bug: composing a
    // compound with atoms used to produce a body whose top-level delimiters
    // collided with the inner compound's body delimiters. After the
    // body_level fix, the top-level delimiter level is always strictly
    // greater than any input's body delimiter level.
    let library = Library::load(elements_dir()).unwrap();

    // Build an inner compound first.
    let inner = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "inner-compound".into(),
        output_name: "Inner".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Stage so it can be re-resolved as an input.
    let tmp = tempdir_for_test("mixed-kind");
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, tmp.join(p.file_name().unwrap())).unwrap();
    }
    write(&inner, &tmp.join("inner-compound.md")).unwrap();

    let staged_lib = Library::load(&tmp).unwrap();
    let mixed = compose(ComposeRequest {
        library: &staged_lib,
        inputs: vec![
            ("inner-compound".into(), None), // compound, body_level 1
            ("tone-direct".into(), None),    // atom
        ],
        output_id: "mixed-kind".into(),
        output_name: "Mixed".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // The outer compound must escalate body_level to 2 to avoid colliding
    // with the inner compound's level-1 delimiters.
    assert_eq!(
        mixed.header.kind,
        oovra::header::PromptElementKind::Compound
    );
    assert_eq!(mixed.header.body_level, Some(2));

    // Decompose must succeed because the outer delimiters are level 2
    // (3 tildes) while the inner element's body uses level-1 (2 tildes).
    let immediate = decompose(&mixed).unwrap();
    assert_eq!(immediate.len(), 2);
    assert_eq!(immediate[0].header.id, "inner-compound");
    assert_eq!(immediate[1].header.id, "tone-direct");

    // The recovered inner is still a valid compound with its own body_level=1.
    assert_eq!(
        immediate[0].header.kind,
        oovra::header::PromptElementKind::Compound
    );
    assert_eq!(immediate[0].header.body_level, Some(1));
}

#[test]
fn depth_equals_body_level_for_every_compose() {
    // SPEC §7.2: for any compose call, the output's depth field equals its
    // body_level. (The SPEC's claim that depth = body_level - 1 contradicts
    // its own §1.3 formula; we follow §1.3.)
    let library = Library::load(elements_dir()).unwrap();

    // Atoms-only compound: both should be 1.
    let c1 = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "depth-check-1".into(),
        output_name: "depth-check-1".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    assert_eq!(c1.header.body_level, Some(1));
    assert_eq!(c1.header.depth, Some(1));

    // Compound-of-compound: both should be 2.
    let tmp = tempdir_for_test("depth-staging");
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, tmp.join(p.file_name().unwrap())).unwrap();
    }
    write(&c1, &tmp.join("depth-check-1.md")).unwrap();
    let staged = Library::load(&tmp).unwrap();
    let c2 = compose(ComposeRequest {
        library: &staged,
        inputs: vec![
            ("depth-check-1".into(), None),
            ("examples-block".into(), None),
        ],
        output_id: "depth-check-2".into(),
        output_name: "depth-check-2".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    assert_eq!(c2.header.body_level, Some(2));
    assert_eq!(c2.header.depth, Some(2));
}

#[test]
fn migrate_rewrites_library_in_place() {
    // SPEC §7.2: build a temp library with v0.1-format files (using
    // `order = 0` and `order = 1` in frontmatter), run migrate_library,
    // verify each file now parses as v0.2 with the correct kind.
    let tmp = tempdir_for_test("migrate-in-place");
    std::fs::create_dir_all(&tmp).unwrap();

    // Write two v0.1 atoms.
    let atom_a = "+++\nname = \"Atom A\"\norder = 0\nid = \"atom-a\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nAtom A body.\n";
    let atom_b = "+++\nname = \"Atom B\"\norder = 0\nid = \"atom-b\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nAtom B body.\n";
    std::fs::write(tmp.join("atom-a.md"), atom_a).unwrap();
    std::fs::write(tmp.join("atom-b.md"), atom_b).unwrap();

    // Write a v0.1 compound whose body wraps two atoms in level-1 delimiters.
    let compound = "+++\nname = \"Compound\"\norder = 1\nid = \"compound\"\nversion = \"1.0.0\"\nmeta = \"\"\ngenerated_at = \"2026-05-09T14:23:15Z\"\nrender_mode = \"markdown-h2\"\nbody_level = 1\ncomposed_of = [{id = \"atom-a\", version = \"1.0.0\"}, {id = \"atom-b\", version = \"1.0.0\"}]\n+++\n\n~~>>\n+++\nname = \"Atom A\"\norder = 0\nid = \"atom-a\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nAtom A body.\n~~<<\n~~>>\n+++\nname = \"Atom B\"\norder = 0\nid = \"atom-b\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nAtom B body.\n~~<<\n";
    std::fs::write(tmp.join("compound.md"), compound).unwrap();

    let summary = migrate_library(&tmp).unwrap();
    assert_eq!(summary.migrated.len(), 3, "all three files should migrate");
    assert!(
        summary.failed.is_empty(),
        "no failures expected: {:?}",
        summary.failed
    );

    // After migration, files parse cleanly in v0.2-only mode (no --legacy).
    let migrated_a = parse_file(&tmp.join("atom-a.md")).unwrap();
    assert_eq!(
        migrated_a.header.kind,
        oovra::header::PromptElementKind::Atom
    );
    let migrated_compound = parse_file(&tmp.join("compound.md")).unwrap();
    assert_eq!(
        migrated_compound.header.kind,
        oovra::header::PromptElementKind::Compound
    );
    assert_eq!(migrated_compound.header.body_level, Some(1));
    assert_eq!(migrated_compound.header.depth, Some(1));
    // generated_at is preserved verbatim per SPEC §10.2.
    assert_eq!(
        migrated_compound.header.generated_at.as_deref(),
        Some("2026-05-09T14:23:15Z")
    );
}

#[test]
fn migrate_preserves_lossless_roundtrip() {
    // SPEC §7.3 headline test: decompose a v0.1 compound, migrate the
    // library, recompose, assert the resulting compound's --text output is
    // whitespace-equivalent to the pre-migration --text output. (The
    // recompose has a fresh generated_at, so frontmatter is NOT
    // byte-identical; we compare the rendered prose.)
    let tmp = tempdir_for_test("migrate-roundtrip");
    std::fs::create_dir_all(&tmp).unwrap();

    // v0.1 atoms.
    std::fs::write(
        tmp.join("role.md"),
        "+++\nname = \"Role\"\norder = 0\nid = \"role\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nYou are a senior engineer.\n",
    )
    .unwrap();
    std::fs::write(
        tmp.join("tone.md"),
        "+++\nname = \"Tone\"\norder = 0\nid = \"tone\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nBe direct.\n",
    )
    .unwrap();

    // Pre-migration render: load the library in legacy mode, compose, capture --text output.
    let pre_lib = Library::load_with(&tmp, oovra::element::ParseOptions { legacy: true }).unwrap();
    let pre_compound = compose(ComposeRequest {
        library: &pre_lib,
        inputs: vec![("role".into(), None), ("tone".into(), None)],
        output_id: "rt".into(),
        output_name: "rt".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let pre_text = render_text(&[&pre_compound]).unwrap();

    // Migrate the library.
    let summary = migrate_library(&tmp).unwrap();
    assert_eq!(summary.migrated.len(), 2);
    assert!(summary.failed.is_empty());

    // Post-migration: same compose, same render. Should match.
    let post_lib = Library::load(&tmp).unwrap();
    let post_compound = compose(ComposeRequest {
        library: &post_lib,
        inputs: vec![("role".into(), None), ("tone".into(), None)],
        output_id: "rt".into(),
        output_name: "rt".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let post_text = render_text(&[&post_compound]).unwrap();

    assert_eq!(
        pre_text.trim(),
        post_text.trim(),
        "rendered text drifted across migration"
    );
}

#[test]
fn migrate_recursively_rewrites_embedded_frontmatter() {
    // Regression test for the body-recursion fix: a v0.1 compound's body
    // contains wrapped sub-element files with their own (v0.1) frontmatters.
    // The migration must rewrite both the outer frontmatter AND every
    // embedded frontmatter, so subsequent v0.2-strict decompose succeeds.
    let tmp = tempdir_for_test("migrate-recursive");
    std::fs::create_dir_all(&tmp).unwrap();

    // Build a v0.1 compound with two v0.1 atoms embedded in its body.
    let compound = "+++\n\
        name = \"Compound\"\n\
        order = 1\n\
        id = \"compound\"\n\
        version = \"1.0.0\"\n\
        meta = \"\"\n\
        generated_at = \"2026-05-09T14:23:15Z\"\n\
        render_mode = \"markdown-h2\"\n\
        body_level = 1\n\
        composed_of = [{id = \"alpha\", version = \"1.0.0\"}, {id = \"beta\", version = \"1.0.0\"}]\n\
        +++\n\n\
        ~~>>\n\
        +++\n\
        name = \"Alpha\"\n\
        order = 0\n\
        id = \"alpha\"\n\
        version = \"1.0.0\"\n\
        meta = \"\"\n\
        +++\n\n\
        Alpha body.\n\
        ~~<<\n\
        ~~>>\n\
        +++\n\
        name = \"Beta\"\n\
        order = 0\n\
        id = \"beta\"\n\
        version = \"1.0.0\"\n\
        meta = \"\"\n\
        +++\n\n\
        Beta body.\n\
        ~~<<\n";
    std::fs::write(tmp.join("compound.md"), compound).unwrap();

    let summary = migrate_library(&tmp).unwrap();
    assert_eq!(summary.migrated.len(), 1);
    assert!(summary.failed.is_empty());

    // After migration, decompose in v0.2-strict mode must succeed because
    // every embedded frontmatter now carries `kind` instead of `order`.
    let migrated = parse_file(&tmp.join("compound.md")).unwrap();
    let leaves = decompose(&migrated).unwrap();
    assert_eq!(leaves.len(), 2);
    assert_eq!(leaves[0].header.id, "alpha");
    assert_eq!(
        leaves[0].header.kind,
        oovra::header::PromptElementKind::Atom
    );
    assert_eq!(leaves[1].header.id, "beta");
    assert_eq!(
        leaves[1].header.kind,
        oovra::header::PromptElementKind::Atom
    );
}

#[test]
fn migrate_is_idempotent_on_v0_2_files() {
    // Running migrate on an already-v0.2 library should re-serialize but
    // not change semantic content. The second migrate run should report
    // 0 failures and the file content should re-parse identically.
    let tmp = tempdir_for_test("migrate-idempotent");
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(
        tmp.join("a.md"),
        "+++\nname = \"A\"\nkind = \"atom\"\nid = \"a\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nbody\n",
    )
    .unwrap();
    let summary1 = migrate_library(&tmp).unwrap();
    assert_eq!(summary1.migrated.len(), 1);
    let summary2 = migrate_library(&tmp).unwrap();
    assert_eq!(summary2.migrated.len(), 1);
    assert!(summary2.failed.is_empty());
    let parsed = parse_file(&tmp.join("a.md")).unwrap();
    assert_eq!(parsed.header.kind, oovra::header::PromptElementKind::Atom);
}

#[test]
fn compare_refuses_atom_vs_compound() {
    let library = Library::load(elements_dir()).unwrap();
    let compound = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "x".into(),
        output_name: "x".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let atom = library.get("role-declaration").unwrap();
    let err = compare(atom, &compound).unwrap_err();
    assert!(matches!(err, oovra::OovraError::KindMismatch { .. }));
}

/// Lightweight tempdir helper. Creates a unique directory under
/// `target/tmp/<name>-<pid>/` that lives for the duration of the test process.
fn tempdir_for_test(name: &str) -> std::path::PathBuf {
    let dir = std::env::current_dir()
        .unwrap()
        .join("target/tmp")
        .join(format!("{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
