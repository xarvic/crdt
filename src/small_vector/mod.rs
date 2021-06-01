//! The module contains a list CRDT for small elements like strings.
//!
//!
//!

mod structure;
mod cvrdt;
mod cmrdt;

pub use cvrdt::{Vector as CvrdtVector, VectorUpdate};
pub use structure::CrdtCollection;

mod tests {
    use crate::CrdtBox;
    use crate::small_vector::{VectorUpdate, CvrdtVector};

    #[test]
    fn test_insert() {
        let mut crdt_a: CrdtBox<CvrdtVector<u64, im::Vector<u64>>> = CrdtBox::new(CvrdtVector::new(1));

        crdt_a.update(VectorUpdate::Insert {
            previous_author: 0,
            previous_id: 0,
            this_author: 1,
            this_id: 1,
            element: 4,
        });

        crdt_a.update(VectorUpdate::Insert {
            previous_author: 1,
            previous_id: 1,
            this_author: 1,
            this_id: 2,
            element: 7,
        });

        crdt_a.update(VectorUpdate::Insert {
            previous_author: 1,
            previous_id: 2,
            this_author: 1,
            this_id: 3,
            element: 10,
        });

        crdt_a.update(VectorUpdate::Insert {
            previous_author: 1,
            previous_id: 1,
            this_author: 1,
            this_id: 4,
            element: 13,
        });

        crdt_a.drain_update();

        let update_a = VectorUpdate::Insert {
            previous_author: 1,
            previous_id: 4,
            this_author: 2,
            this_id: 5,
            element: 14
        };

        let update_b = VectorUpdate::Insert {
            previous_author: 2,
            previous_id: 5,
            this_author: 2,
            this_id: 6,
            element: 15,
        };

        let update_c = VectorUpdate::Insert {
            previous_author: 1,
            previous_id: 2,
            this_author: 1,
            this_id: 5,
            element: 20
        };

        let update_d = VectorUpdate::Insert {
            previous_author: 1,
            previous_id: 5,
            this_author: 1,
            this_id: 6,
            element: 30,
        };

        let mut crdt_b = crdt_a.clone();

        // First ab than cd

        crdt_a.update(update_a.clone());
        crdt_a.update(update_b.clone());

        crdt_a.update(update_c.clone());
        crdt_a.update(update_d.clone());

        // First cd than ab

        crdt_b.update(update_c);
        crdt_b.update(update_d);

        crdt_b.update(update_a);
        crdt_b.update(update_b);

        assert_eq!(crdt_a.document(), crdt_b.document());
    }

    #[test]
    fn test_remove() {
        let mut crdt_a: CrdtBox<CvrdtVector<u64, im::Vector<u64>>> = CrdtBox::new(CvrdtVector::with_data(
            im::Vector::from(&[0u64, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..]),
            1,
        ));

        let mut crdt_b = crdt_a.clone();

        let update_a = VectorUpdate::Delete {
            author: 0,
            id: 4,
        };

        let update_b = VectorUpdate::Delete {
            author: 0,
            id: 5,
        };

        let update_c = VectorUpdate::Delete {
            author: 0,
            id: 7,
        };

        let update_d = VectorUpdate::Delete {
            author: 0,
            id: 8,
        };

        let update_e = VectorUpdate::Delete {
            author: 0,
            id: 6,
        };

        crdt_a.dbg_spans();

        crdt_a.update(update_a.clone());

        crdt_a.dbg_spans();

        crdt_a.update(update_b.clone());

        crdt_a.dbg_spans();

        crdt_a.update(update_c.clone());

        crdt_a.dbg_spans();

        crdt_a.update(update_d.clone());

        crdt_a.dbg_spans();

        crdt_a.update(update_e.clone());

        crdt_a.dbg_spans();

        println!("--------");

        crdt_b.update(update_c);
        crdt_b.update(update_d);

        crdt_b.update(update_a);
        crdt_b.update(update_b);

        crdt_b.update(update_e);

        println!("{:?}", crdt_a.document());
        assert_eq!(crdt_a.document(), crdt_b.document());
        println!()
    }
}