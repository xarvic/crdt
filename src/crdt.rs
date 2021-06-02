#[allow(clippy::upper_case_acronyms)]
pub trait CvRDT {
    type Update: Clone;

    fn update(&mut self, update: Self::Update);
}

#[allow(clippy::upper_case_acronyms)]
pub trait CmRDT {
    fn merge(&mut self, other: &Self);
}
