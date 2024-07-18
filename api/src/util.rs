use axum::{body::Bytes, BoxError};
use futures::{Stream, TryStreamExt};
use tokio::{
    fs::{create_dir, File},
    io::{self, BufWriter},
};
use tokio_util::io::StreamReader;

use crate::error::CustomError;

pub fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();
    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }
    components.count() == 1
}

pub async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), CustomError>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err(anyhow::anyhow!("Invalid path").into());
    }

    Ok(async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let upload_path = std::path::Path::new("./uploads");
        if !upload_path.exists() {
            create_dir(upload_path).await?;
        }
        let path = upload_path.join(path);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await?)
}
