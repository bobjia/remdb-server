use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::clone::Clone;
use std::io;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;

/// 零拷贝传输层
pub struct ZeroCopyTransport {
    inner: TcpStream,
    /// 预分配的缓冲区池
    buffer_pool: BufferPool,
    /// 零拷贝优化标志
    zero_copy_enabled: bool,
}

/// 缓冲区池
struct BufferPool {
    buffers: Vec<BytesMut>,
    current: usize,
}

impl BufferPool {
    /// 创建新的缓冲区池
    fn new(pool_size: usize, buffer_size: usize) -> Self {
        let mut buffers = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            let mut buf = BytesMut::with_capacity(buffer_size);
            unsafe {
                buf.set_len(buffer_size);
            }
            buffers.push(buf);
        }

        Self {
            buffers,
            current: 0,
        }
    }

    /// 获取缓冲区
    fn get_buffer(&mut self) -> BytesMut {
        let idx = self.current;
        self.current = (self.current + 1) % self.buffers.len();
        self.buffers[idx].clone()
    }
}

impl ZeroCopyTransport {
    /// 创建新的零拷贝传输实例
    pub fn new(socket: TcpStream) -> Self {
        Self {
            inner: socket,
            buffer_pool: BufferPool::new(16, 8192), // 16个8KB缓冲区
            // 在Linux和Windows平台上都启用零拷贝优化
            zero_copy_enabled: cfg!(target_os = "linux") || cfg!(target_os = "windows"),
        }
    }

    /// 零拷贝读取
    pub async fn read_zero_copy(&mut self) -> io::Result<Bytes> {
        if self.zero_copy_enabled {
            // 使用preallocated buffer避免拷贝
            let mut buf = self.buffer_pool.get_buffer();

            let n = self.inner.read_buf(&mut buf).await?;
            unsafe {
                buf.set_len(n);
            }

            Ok(buf.freeze())
        } else {
            // 回退到普通读取
            let mut buf = vec![0u8; 8192];
            let n = self.inner.read(&mut buf).await?;
            buf.truncate(n);
            Ok(Bytes::from(buf))
        }
    }

    /// 批量零拷贝发送
    pub async fn send_batch_zero_copy<I>(&mut self, batches: I) -> io::Result<()>
    where
        I: IntoIterator<Item = Bytes>,
    {
        use tokio::io::AsyncWriteExt;

        for batch in batches {
            // 使用write_all_vectored进行聚集写入
            self.inner.write_all(&batch).await?;
        }

        self.inner.flush().await?;
        Ok(())
    }

    /// 发送单个零拷贝数据
    pub async fn send_zero_copy(&mut self, data: Bytes) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;

        self.inner.write_all(&data).await?;
        self.inner.flush().await?;
        Ok(())
    }

    /// 启用TCP_NODELAY和TCP_QUICKACK
    pub fn set_tcp_options(&self) -> io::Result<()> {
        // 禁用Nagle算法，适用于所有平台
        self.inner.set_nodelay(true)?;

        // TCP_QUICKACK仅在Linux平台上支持
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::io::AsRawFd;

            let fd = self.inner.as_raw_fd();

            unsafe {
                // 启用TCP快速确认（仅Linux）
                let quickack: libc::c_int = 1;
                libc::setsockopt(
                    fd,
                    libc::IPPROTO_TCP,
                    libc::TCP_QUICKACK,
                    &quickack as *const _ as *const libc::c_void,
                    std::mem::size_of_val(&quickack) as libc::socklen_t,
                );
            }
        }

        Ok(())
    }

    /// 获取底层TcpStream的引用
    pub fn get_inner(&self) -> &TcpStream {
        &self.inner
    }

    /// 获取底层TcpStream的可变引用
    pub fn get_inner_mut(&mut self) -> &mut TcpStream {
        &mut self.inner
    }
}

// 实现AsyncRead trait
impl AsyncRead for ZeroCopyTransport {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

// 实现AsyncWrite trait
impl AsyncWrite for ZeroCopyTransport {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
