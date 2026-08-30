#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A> Either<A, A> {
    pub fn unify(self) -> A {
        match self {
            Self::Left(a) => a,
            Self::Right(a) => a,
        }
    }
}
