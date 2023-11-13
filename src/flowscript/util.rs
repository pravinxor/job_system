use std::iter::Peekable;

pub(crate) fn extract_until<T, I, P>(iter: &mut Peekable<I>, predicate: P) -> Vec<T>
where
    I: Iterator<Item = T>,
    P: Fn(&T) -> bool,
{
    let mut v = Vec::new();
    while iter.peek().filter(|e| !predicate(e)).is_some() {
        v.push(iter.next().unwrap())
    }
    v
}

pub struct Spliterator<T, I, P>
where
    I: Iterator<Item = T>,
    P: Fn(&T) -> bool,
{
    inner: I,
    predicate: P,
}

impl<T, I, P> Spliterator<T, I, P>
where
    I: Iterator<Item = T>,
    P: Fn(&T) -> bool,
{
    fn new(inner: I, predicate: P) -> Self {
        Self { inner, predicate }
    }
}

impl<T, I, P> Iterator for Spliterator<T, I, P>
where
    I: Iterator<Item = T>,
    P: Fn(&T) -> bool,
{
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let chunk = extract_until(&mut self.inner.by_ref().peekable(), &self.predicate);
        if !chunk.is_empty() {
            Some(chunk)
        } else {
            None
        }
    }
}

pub trait SpliteratorAdapter: Iterator {
    fn split_by<P>(self, predicate: P) -> Spliterator<Self::Item, Self, P>
    where
        Self: Sized,
        P: Fn(&Self::Item) -> bool,
    {
        Spliterator::new(self, predicate)
    }
}
impl<T> SpliteratorAdapter for T where T: Iterator {}
