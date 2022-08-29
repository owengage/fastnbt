use de::Deserializer;
use error::Result;

pub mod de;
pub mod error;
mod input;

#[cfg(test)]
mod test;

pub fn from_str<'a, T>(input: &'a str) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    let mut des = Deserializer::from_str(input);
    let t = T::deserialize(&mut des)?;
    Ok(t)
}
