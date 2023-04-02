pub trait AgainstGuidelines<T>
where
    Self: Sized,
{
    type Error;

    /// Check if the given data follows the guidelines
    fn against_guidelines(self, guidelines: &T) -> Result<Self, Self::Error>;
}
