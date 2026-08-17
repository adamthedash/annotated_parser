use std::cmp::Ordering;

use crate::ParserOutput;

pub trait Input: Sized {
    type OwnedConst<const N: usize>: ParserOutput;
    type OwnedVar: ParserOutput;

    fn length(&self) -> usize;

    fn take_const<const N: usize>(&self) -> Option<(Self::OwnedConst<N>, Self)>;

    fn take_var(&self, n: usize) -> Option<(Self::OwnedVar, Self)>;
}

impl Input for &[u8] {
    type OwnedConst<const N: usize> = [u8; N];
    type OwnedVar = Vec<u8>;

    fn length(&self) -> usize {
        self.len()
    }

    fn take_const<const N: usize>(&self) -> Option<(Self::OwnedConst<N>, Self)> {
        self.split_first_chunk().map(|(taken, rest)| (*taken, rest))
    }

    fn take_var(&self, n: usize) -> Option<(Self::OwnedVar, Self)> {
        self.split_at_checked(n)
            .map(|(taken, rest)| (taken.to_owned(), rest))
    }
}

impl Input for &str {
    type OwnedConst<const N: usize> = String;

    type OwnedVar = String;

    fn length(&self) -> usize {
        self.chars().count()
    }

    fn take_const<const N: usize>(&self) -> Option<(Self::OwnedConst<N>, Self)> {
        // TODO: Check if this properly inlines and the match optimised away
        self.take_var(N)
    }

    fn take_var(&self, n: usize) -> Option<(Self::OwnedVar, Self)> {
        let i = match self.chars().count().cmp(&n) {
            Ordering::Less => return None,
            Ordering::Equal => self.len(),
            Ordering::Greater => {
                let (i, _) = self.char_indices().nth(n).expect("length is at least N+1");
                i
            }
        };

        let taken = self[..i].to_owned();
        let rest = &self[i..];

        Some((taken, rest))
    }
}
