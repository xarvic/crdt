pub trait CvRDT {
    type Update: Clone;

    fn update(&mut self, update: Self::Update);
}

pub trait CmRDT {
    fn merge(&mut self, other: &Self);
}