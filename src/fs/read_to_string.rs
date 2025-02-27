use std::fs;
use std::path::Path;

use crate::io;
use crate::task::blocking;

/// Read the entire contents of a file into a string.
///
/// This function is an async version of [`std::fs::read_to_string`].
///
/// [`std::fs::read_to_string`]: https://doc.rust-lang.org/std/fs/fn.read_to_string.html
///
/// # Errors
///
/// An error will be returned in the following situations (not an exhaustive list):
///
/// * `path` is not a file.
/// * The current process lacks permissions to read `path`.
///
/// # Examples
///
/// ```no_run
/// # #![feature(async_await)]
/// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
/// #
/// use async_std::fs;
///
/// let contents = fs::read_to_string("a.txt").await?;
/// #
/// # Ok(()) }) }
/// ```
pub async fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let path = path.as_ref().to_owned();
    blocking::spawn(async move { fs::read_to_string(path) }).await
}
