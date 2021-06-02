mod small_vec;



#[allow(clippy::upper_case_acronyms)]
pub trait CmRDT {
    fn merge(&mut self, other: &Self);
}
