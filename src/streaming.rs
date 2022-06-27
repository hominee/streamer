use futures_util::stream::Stream;
use std::{
    borrow::Cow,
    fs::File,
    io::Read,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Streaming<T> {
    inner: T,
    offset: usize,
    len: usize,
}

impl<T> Streaming<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: data,
            offset: 0,
            len: 0,
        }
    }
}

impl From<File> for Streaming<File> {
    fn from(file: File) -> Self {
        Self {
            len: file.metadata().unwrap().len() as _,
            inner: file,
            offset: 0,
        }
    }
}

impl Stream for Streaming<File> {
    type Item = u8;
    fn poll_next(mut self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut byte = 0;
        let mut r = Poll::Ready(None);
        if let Ok(size) = self.as_mut().inner.read(std::slice::from_mut(&mut byte)) {
            if size > 0 {
                //println!("offset {:?}, buf: {:?}", offset, buf);
                r = Poll::Ready(Some(byte))
            }
        }
        r
        /*
         *stream::poll_fn(move |_| -> Poll<Option<&'static [u8]>> {
         *    match f.read(&mut buf) {
         *        Ok(size) => {
         *            if size > 0 {
         *                println!("{:?}", &size);
         *                Poll::Ready(Some(buf))
         *            } else {
         *                println!("{:?}", "EOF");
         *                Poll::Ready(None)
         *            }
         *        }
         *        Err(_) => Poll::Ready(None),
         *    }
         *})
         *.flat_map(|e| stream::iter(e))
         *.chunks(5)
         */
    }
}

#[test]
fn test_stream_file() {
    use futures_util::StreamExt;
    async fn run() {
        let file = File::open("markdown-tools.js").unwrap();
        let streaming = Streaming {
            len: file.metadata().unwrap().len() as _,
            offset: 0,
            inner: file,
        };
        streaming
            .chunks(5)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", &en);
            })
            .await;
    }

    use tokio::runtime::Builder;
    let _rt = Builder::new_current_thread().enable_all().build().unwrap();
    //_rt.block_on(run());
    //assert!(false);
}

impl<T: Clone, const N: usize> From<[T; N]> for Streaming<Cow<'static, [T]>> {
    fn from(s: [T; N]) -> Self {
        Self {
            len: s.len(),
            inner: Cow::Owned(Box::<[T]>::from(s).into_vec()),
            offset: 0,
        }
    }
}

impl<T: Clone> From<Vec<T>> for Streaming<Cow<'static, Vec<T>>> {
    fn from(s: Vec<T>) -> Self {
        Self {
            len: s.len(),
            inner: Cow::Owned(s),
            offset: 0,
        }
    }
}

impl<T: Clone + Copy + Unpin> Stream for Streaming<Cow<'static, Vec<T>>> {
    type Item = T;
    fn poll_next(mut self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut r = Poll::Ready(None);
        let offset = self.offset;
        if offset > self.len - 1 {
            return r;
        }
        if let Cow::Owned(ref own) = self.as_mut().inner {
            r = Poll::Ready(Some(own[offset]));
            self.get_mut().offset += 1;
        }
        r
    }
}

impl From<&'static str> for Streaming<Cow<'static, [u8]>> {
    fn from(s: &'static str) -> Self {
        Self {
            len: s.len(),
            inner: Cow::Borrowed(s.as_bytes()),
            offset: 0,
        }
    }
}

impl From<String> for Streaming<Cow<'static, [u8]>> {
    fn from(s: String) -> Self {
        Self {
            len: s.len(),
            inner: Cow::Owned(s.into_bytes()),
            offset: 0,
        }
    }
}

impl Stream for Streaming<Cow<'static, [u8]>> {
    type Item = u8;
    fn poll_next(mut self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut r = Poll::Ready(None);
        let offset = self.offset;
        if offset > self.len - 1 {
            return r;
        }
        match self.as_mut().inner {
            Cow::Owned(ref mut own) => {
                r = Poll::Ready(Some(own[offset]));
                self.as_mut().offset += 1;
            }
            Cow::Borrowed(bow) => {
                r = Poll::Ready(Some(bow[offset]));
                self.as_mut().offset += 1;
            }
        }

        r
    }
}

#[test]
fn test_stream_byte() {
    use futures_util::StreamExt;
    async fn run() {
        let file: [u64; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 00];
        let streaming = Streaming::from(Box::from(file));
        streaming
            .chunks(4)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", &en);
            })
            .await;
        let file: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let streaming = Streaming::from(file);
        streaming
            .chunks(4)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", &en);
            })
            .await;
        let file = "a very long string though";
        let streaming = Streaming::from(file);
        streaming
            .chunks(10)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", std::str::from_utf8(&en));
            })
            .await;
        let file = String::from("a very long string though");
        let streaming = Streaming::from(file);
        streaming
            .chunks(8)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", std::str::from_utf8(&en));
            })
            .await;
        let file = vec![11, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let streaming = Streaming::from(file);
        streaming
            .chunks(4)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", &en);
            })
            .await;
    }

    use tokio::runtime::Builder;
    let _rt = Builder::new_current_thread().enable_all().build().unwrap();
    //_rt.block_on(run());
    //assert!(false);
}

impl<T> From<Box<[T]>> for Streaming<Cow<'static, Vec<T>>>
where
    T: Clone,
{
    fn from(s: Box<[T]>) -> Self {
        Self {
            len: s.len(),
            inner: Cow::Owned(s.into_vec()),
            offset: 0,
        }
    }
}
