use std::io::{Error, ErrorKind, Result};

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn read_length_prefixed<R: AsyncRead + Unpin>(
    reader: &mut R,
    max_buf_size: usize,
) -> Result<Vec<u8>> {
    let len = unsigned_varint::aio::read_u64(&mut *reader)
        .await
        .map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("failed to read length-prefixed message: {e}"),
            )
        })? as usize;

    if len > max_buf_size {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("message too large: {len} bytes, max: {max_buf_size} bytes"),
        ));
    }

    let mut buf = vec![0; len];
    reader.read_exact(&mut buf).await?;

    Ok(buf)
}

pub async fn write_length_prefixed<W: AsyncWrite + Unpin>(
    writer: &mut W,
    data: &[u8],
) -> Result<()> {
    let len = data.len() as u64;
    let mut len_buf = unsigned_varint::encode::u64_buffer();
    let len_bytes = unsigned_varint::encode::u64(len, &mut len_buf);

    writer.write_all(len_bytes).await?;
    writer.write_all(data).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use futures::io::Cursor;

    #[test]
    fn test_read_length_prefixed() {
        block_on(async {
            let mut buf = Cursor::new(vec![0x04, 0x01, 0x02, 0x03, 0x04]);
            let data = read_length_prefixed(&mut buf, 32).await.unwrap();
            assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04]);
        });
    }

    #[test]
    fn test_write_length_prefixed() {
        block_on(async {
            let mut buf = Vec::new();

            write_length_prefixed(&mut buf, &[0x01, 0x02, 0x03, 0x04])
                .await
                .unwrap();

            assert_eq!(buf, vec![0x04, 0x01, 0x02, 0x03, 0x04]);
        });
    }
}
