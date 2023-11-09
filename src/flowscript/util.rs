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
