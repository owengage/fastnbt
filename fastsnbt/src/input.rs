use crate::error::Result;

mod private {
    // Only this crate can implement this trait. Other traits can inherit from
    // Sealed in order to prevent other crates from creating implementations.
    pub trait Sealed {}
}

pub enum Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

pub trait Input<'de>: private::Sealed {
    #[doc(hidden)]
    fn next(&mut self) -> Result<Option<u8>>;
    #[doc(hidden)]
    fn peek(&mut self) -> Result<Option<u8>>;
}

pub(crate) struct StrInput<'de> {
    pub data: &'de [u8], // assume UTF-8
}

impl private::Sealed for StrInput<'_> {}

impl<'de> Input<'de> for StrInput<'de> {
    fn next(&mut self) -> Result<Option<u8>> {
        todo!()
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        Ok(self.data.first().cloned())
    }
}
