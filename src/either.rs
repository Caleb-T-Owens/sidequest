#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Either<A, B> {
    Left(A),
    Right(B),
}
