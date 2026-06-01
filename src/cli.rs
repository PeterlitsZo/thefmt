use crate::core::format_markdown;
use crate::error::Error;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn run<I>(args: I) -> Result<(), Error>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let path = match (args.next(), args.next()) {
        (Some(path), None) => PathBuf::from(path),
        _ => return Err(Error::Usage),
    };

    let input = fs::read_to_string(&path).map_err(|source| Error::Read {
        path: path.clone(),
        source,
    })?;
    let output = format_markdown(&input)?;

    fs::write(&path, output).map_err(|source| Error::Write { path, source })?;
    Ok(())
}

pub fn args_without_binary_name() -> impl Iterator<Item = String> {
    env::args().skip(1)
}
