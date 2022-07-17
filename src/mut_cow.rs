use std::borrow::{Borrow, BorrowMut};
use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

/// A mutable version of [`std::borrow::Cow`].
///
/// Despite the name still contains "Cow" (Copy-on-write) part, this actually
/// never does any copying. This allows the structure making use of it to either
/// operate on a `&mut` reference to the data, or to own the data.
pub enum MutCow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    /// Mutably borrowed data.
    Borrowed(&'a mut B),
    /// Owned data.
    Owned(<B as ToOwned>::Owned),
}

impl<B: ?Sized> Debug for MutCow<'_, B>
where
    B: Debug + ToOwned,
    B::Owned: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MutCow::Borrowed(ref b) => Debug::fmt(b, f),
            MutCow::Owned(ref o) => Debug::fmt(o, f),
        }
    }
}

impl<'a> From<&'a mut [u8]> for MutCow<'a, [u8]> {
    fn from(reference: &'a mut [u8]) -> Self {
        Self::Borrowed(reference)
    }
}

impl<'a, const N: usize> From<&'a mut [u8; N]> for MutCow<'a, [u8]> {
    fn from(reference: &'a mut [u8; N]) -> Self {
        Self::Borrowed(reference)
    }
}

impl<'a, const N: usize> From<[u8; N]> for MutCow<'a, [u8]> {
    fn from(owned: [u8; N]) -> Self {
        Self::Owned(Vec::from(owned))
    }
}

impl<'a> From<Vec<u8>> for MutCow<'a, [u8]> {
    fn from(owned: Vec<u8>) -> Self {
        Self::Owned(owned)
    }
}

impl<'a> Deref for MutCow<'a, [u8]> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            MutCow::Borrowed(reference) => reference,
            MutCow::Owned(ref owned) => owned.borrow(),
        }
    }
}

impl<'a> DerefMut for MutCow<'a, [u8]> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MutCow::Borrowed(reference) => reference,
            MutCow::Owned(owned) => owned.borrow_mut(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mut_cow::MutCow;

    #[test]
    fn test_create_from_array() {
        let mut data: [u8; 4] = [1, 2, 3, 4];
        let mut mut_cow = MutCow::from(&mut data);
        mut_cow[0] = 5;

        assert_eq!(*mut_cow, [5, 2, 3, 4]);
        assert_eq!(data, [5, 2, 3, 4]);
    }

    #[test]
    fn test_create_from_slice() {
        let mut data = vec![1, 2, 3, 4];
        let mut mut_cow = MutCow::from(data.as_mut_slice());
        mut_cow[0] = 5;

        assert_eq!(*mut_cow, [5, 2, 3, 4]);
        assert_eq!(data, [5, 2, 3, 4]);
    }

    #[test]
    fn test_create_from_owned() {
        let data: [u8; 4] = [1, 2, 3, 4];
        let mut mut_cow = MutCow::from(data.to_vec());
        mut_cow[0] = 5;

        assert_eq!(*mut_cow, [5, 2, 3, 4]);
    }
}
