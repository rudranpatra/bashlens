//! Every rule under `rules/<id>/` ships a `fixtures/pass.sh` (must trigger
//! the rule) and `fixtures/fail.sh` (must not). This test generically walks
//! every rule directory and checks both, so a new rule is covered the moment
//! its fixtures exist - no per-rule test function to remember to add.

use bashlens_rules::RuleSet;
use std::path::PathBuf;

fn rules_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../rules"))
}

#[test]
fn every_rule_fires_on_its_pass_fixture_and_not_its_fail_fixture() {
    let dir = rules_dir();
    let rule_set = RuleSet::load_dir(&dir).expect("rules/ should load");

    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).expect("rules dir readable") {
        let entry = entry.expect("dir entry readable");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = path.file_name().unwrap().to_str().unwrap().to_string();
        let pass_path = path.join("fixtures/pass.sh");
        let fail_path = path.join("fixtures/fail.sh");
        if !pass_path.is_file() || !fail_path.is_file() {
            continue;
        }

        let rule = rule_set
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("rule {id:?} has fixtures but didn't load"));

        let pass_src = std::fs::read_to_string(&pass_path).unwrap();
        let fail_src = std::fs::read_to_string(&fail_path).unwrap();
        let pass_tree = bashlens_parser::parse(&pass_src).unwrap();
        let fail_tree = bashlens_parser::parse(&fail_src).unwrap();

        assert!(
            !rule.evaluate(&pass_tree, &pass_src).is_empty(),
            "rule {id:?} did not fire on its own fixtures/pass.sh:\n{pass_src}"
        );
        assert!(
            rule.evaluate(&fail_tree, &fail_src).is_empty(),
            "rule {id:?} incorrectly fired on its own fixtures/fail.sh:\n{fail_src}"
        );
        checked += 1;
    }

    assert!(
        checked >= 10,
        "expected at least 10 rules with fixtures to be checked, found {checked} - \
         did the rules/ directory move?"
    );
}
