use adversary::model::{Operation, State, allowed_recovery};
fn state(a: &str, b: &str) -> State {
    [("a".into(), a.into()), ("b".into(), b.into())].into()
}
#[test]
fn unknown_atomic_operation_allows_only_two_complete_states() {
    let before = state("old", "old");
    let pending = Operation {
        id: "request-17".into(),
        batch: state("new", "new"),
    };
    assert!(allowed_recovery(&before, Some(&pending), &before));
    assert!(allowed_recovery(
        &before,
        Some(&pending),
        &state("new", "new")
    ));
    assert!(!allowed_recovery(
        &before,
        Some(&pending),
        &state("new", "old")
    ));
}
#[test]
fn acknowledged_operation_cannot_be_rolled_back() {
    assert!(!allowed_recovery(
        &state("committed", "committed"),
        None,
        &state("prior", "prior")
    ));
}
#[test]
fn pending_partial_key_batch_preserves_unrelated_acknowledged_keys() {
    let before = state("old", "durable");
    let pending = Operation {
        id: "partial-domain".into(),
        batch: [("a".into(), "new".into())].into(),
    };
    assert!(allowed_recovery(
        &before,
        Some(&pending),
        &state("new", "durable")
    ));
    assert!(!allowed_recovery(
        &before,
        Some(&pending),
        &state("new", "lost")
    ));
}

#[test]
fn explicit_absence_is_a_counterexample_not_an_observation_gap() {
    let observed = [("a".into(), Some("durable".into())), ("b".into(), None)].into();
    assert!(!adversary::model::allowed_observation(
        &state("durable", "durable"),
        None,
        &observed
    ));
}
