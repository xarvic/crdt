use crate::small_vector::{CvrdtVector, VectorUpdate, CrdtCollection};
use im::Vector;
use crate::CrdtBox;
use std::fmt::Debug;

#[test]
fn simple_insert() {
    test_small_vec(
        Vector::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9][..]),
        Vector::from(&[0, 1, 2, 3, 35, 4, 5, 55, 56, 6, 7, 8, 9][..]),
        |a, b|{
            a.update(VectorUpdate::Insert {
                previous_author: 0,
                previous_id: 4,
                this_author: 1,
                this_id: 1,
                element: 35,
            });

            b.update(VectorUpdate::Insert {
                previous_author: 0,
                previous_id: 6,
                this_author: 2,
                this_id: 1,
                element: 55,
            });

            b.update(VectorUpdate::Insert {
                previous_author: 2,
                previous_id: 1,
                this_author: 2,
                this_id: 2,
                element: 56,
            });
        }
    );
}

#[test]
fn simple_delete() {
    test_small_vec(
        Vector::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9][..]),
        Vector::from(&[0, 1, 2, 4, 7, 8, 9][..]),
        |a, b|{
            a.update(VectorUpdate::Delete {
                author: 0,
                id: 4,
            });

            b.update(VectorUpdate::Delete {
                author: 0,
                id: 6,
            });

            b.update(VectorUpdate::Delete {
                author: 0,
                id: 7,
            });
        }
    );
}

fn test_small_vec<V: CrdtCollection + PartialEq + Clone + Debug>(
    initial: V,
    expected: V,
    change: impl Fn(&mut CrdtBox<CvrdtVector<V>>, &mut CrdtBox<CvrdtVector<V>>)
) {
    let mut crdt_a = CrdtBox::new(CvrdtVector::with_data(initial.clone(), 1));
    let mut crdt_b = CrdtBox::new(CvrdtVector::with_data(initial.clone(), 2));

    change(&mut crdt_a, &mut crdt_b);

    let mut old_b = crdt_b.clone();

    for update in crdt_a.drain_update() {
        crdt_b.update(update);
    }

    for update in old_b.drain_update() {
        crdt_a.update(update);
    }

    assert_eq!(crdt_a.document(), crdt_b.document(), "the CRDTs don't converge from {:?}", initial);
    assert!(
        crdt_a.document() == &expected,
        "the converged CRDTs don't have the expected value:\nExpected: {:?}\n Actual: {:?}\n",
        expected,
        crdt_a.document()
    );
}