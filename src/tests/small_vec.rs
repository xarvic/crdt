use crate::cvrdt::{CrdtBox, SmallVector};
use crate::util::CrdtCollection;

use std::fmt::Debug;
use im::Vector;

#[test]
fn simple_insert() {
    test_small_vec(
        Vector::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9][..]),
        Vector::from(&[0, 1, 2, 3, 35, 4, 5, 55, 56, 6, 7, 8, 9][..]),
        |a, b| {
            a.insert(4, 35);

            b.insert(6, 55);
            b.insert(7, 56);
        },
    );
}

#[test]
fn simple_delete() {
    test_small_vec(
        Vector::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9][..]),
        Vector::from(&[0, 1, 2, 4, 7, 8, 9][..]),
        |a, b| {
            a.delete(3);

            b.delete(5);
            b.delete(5);
        },
    );
}

fn test_small_vec<V: CrdtCollection + PartialEq + Clone + Debug>(
    initial: V,
    expected: V,
    change: impl Fn(&mut CrdtBox<SmallVector<V>>, &mut CrdtBox<SmallVector<V>>),
) {
    let mut crdt_a = CrdtBox::new(SmallVector::with_data(initial.clone(), 1));
    let mut crdt_b = CrdtBox::new(SmallVector::with_data(initial.clone(), 2));

    change(&mut crdt_a, &mut crdt_b);

    let mut old_b = crdt_b.clone();

    for update in crdt_a.drain_update() {
        crdt_b.update(update);
    }

    for update in old_b.drain_update() {
        crdt_a.update(update);
    }

    assert_eq!(
        crdt_a.document(),
        crdt_b.document(),
        "the CRDTs don't converge from {:?}",
        initial
    );
    assert!(
        crdt_a.document() == &expected,
        "the converged CRDTs don't have the expected value:\nExpected: {:?}\n Actual: {:?}\n",
        expected,
        crdt_a.document()
    );
}
