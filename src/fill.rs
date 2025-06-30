/// Fill an iterator with a specified value.
pub trait Fill: Iterator + Sized {
    fn fill(self, filler: Self::Item) -> Filler<Self>;
}

impl<T> Fill for T
where
    T: Iterator,
{
    fn fill(self, filler: Self::Item) -> Filler<Self> {
        Filler::new(self, filler)
    }
}

pub struct Filler<T>
where
    T: Iterator,
{
    iter: T,
    filler: T::Item,
}

impl<T> Filler<T>
where
    T: Iterator,
{
    #[must_use]
    pub const fn new(iter: T, filler: T::Item) -> Self {
        Self { iter, filler }
    }
}

impl<T> Iterator for Filler<T>
where
    T: Iterator,
    <T as Iterator>::Item: Clone,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.iter.next() {
            Some(item)
        } else {
            Some(self.filler.clone())
        }
    }
}
