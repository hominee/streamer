use crate::streaming::*;
use futures_util::stream::{Stream, StreamExt};
use hyper::{body::Bytes, Body};
use std::io;

pub struct Streamer<T> {
    pub src: Streaming<T>,
    pub meta: Meta,
}

pub struct Meta {
    pub name: Option<String>,
    pub filename: Option<String>,
    pub boundary: Boundary,
    pub buf_len: usize,
}
impl Meta {
    pub fn set_filename<T: Into<String>>(&mut self, filename: T) {
        self.filename = Some(filename.into());
    }

    pub fn set_name<T: Into<String>>(&mut self, name: T) {
        self.name = Some(name.into());
    }

    pub fn set_buf_len(&mut self, buf_len: usize) {
        self.buf_len = buf_len;
    }

    fn ser_name(&self) -> String {
        if self.name.is_none() {
            "".into()
        } else {
            format!(" filename=\"{}\";", self.name.as_ref().unwrap())
        }
    }

    fn ser_filename(&self, ind: usize) -> String {
        if self.filename.is_none() {
            "".into()
        } else {
            format!(" name=\"{}.{}\";", self.filename.as_ref().unwrap(), ind)
        }
    }

    pub fn write_head(&self, ind: usize) -> Bytes {
        let s = format!(
            "--{}\r\nContent-Disposition: form-data;{}{}\r\n\r\n",
            self.boundary.to_str(),
            self.ser_name(),
            self.ser_filename(ind)
        );
        Bytes::from(s)
    }

    pub fn write_tail(&self, body_len: usize) -> Bytes {
        let mut s = "\r\n".into();
        if body_len < self.buf_len {
            s = format!("\r\n--{}--\r\n", self.boundary.to_str());
        }
        Bytes::from(s)
    }
}

impl<T> Streamer<T>
where
    //T: Stream + Send + 'static,
    Streaming<T>: Stream + Send + 'static,
    <Streaming<T> as Stream>::Item: Send + Into<u8>,
    Bytes: From<Vec<<Streaming<T> as Stream>::Item>>,
{
    pub fn new<P>(src: P) -> Self
    where
        Streaming<T>: From<P>,
    {
        let boundary = gen_boundary("1234567890abcdefghijklmnopqrstuvw");
        let src = Streaming::<T>::from(src);
        Self {
            src: src,
            meta: Meta {
                name: None,
                filename: None,
                boundary,
                buf_len: 64 * 1024,
            },
        }
    }

    pub fn streaming(self) -> Body {
        let (src, meta) = (self.src, self.meta);
        let mut ind = 0;
        let stream = src
            .chunks(meta.buf_len)
            .map(move |ck| {
                let head = meta.write_head(ind);
                ind += 1;
                let tail = meta.write_tail(ck.len());
                let stream_head = futures_util::stream::once(async { Ok::<_, io::Error>(head) });
                let stream_body =
                    futures_util::stream::once(async { Ok::<_, io::Error>(Bytes::from(ck)) });
                let stream_tail = futures_util::stream::once(async { Ok::<_, io::Error>(tail) });
                stream_head.chain(stream_body).chain(stream_tail)
            })
            .flatten();

        Body::wrap_stream(stream)
    }
}

#[test]
fn test_streamer() {
    use futures_util::StreamExt;

    async fn run() {
        //let file: [u8; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 00];
        let file = std::fs::File::open("info").unwrap();
        let mut streaming = Streamer::new(file);
        streaming.meta.set_buf_len(10);
        streaming.meta.set_name("doc");
        streaming.meta.set_filename("info");
        streaming
            .streaming()
            .take(100)
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

pub struct Boundary(pub [u8; 32]);
impl Boundary {
    pub fn to_str(&self) -> String {
        String::from_utf8_lossy(&self.0).into()
    }
}
impl From<&Meta> for Boundary {
    fn from(meta: &Meta) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut inner = [0; 32];
        let mut hasher = DefaultHasher::new();
        inner.iter_mut().for_each(|e| {
            hasher.write_usize(meta.buf_len);
            hasher.write(meta.filename.as_ref().unwrap_or(&"".into()).as_bytes());
            hasher.write(meta.name.as_ref().unwrap_or(&"".into()).as_bytes());
            let raw_ind = hasher.finish() % 36;
            if raw_ind < 10 {
                *e = raw_ind as u8 + 48;
            } else {
                *e = raw_ind as u8 - 10 + 97;
            }
        });
        Boundary(inner)
    }
}
pub fn gen_boundary<T: AsRef<[u8]>>(del: T) -> Boundary {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    let mut inner = [0; 32];
    let mut hasher = DefaultHasher::new();
    inner.iter_mut().for_each(|e| {
        hasher.write(del.as_ref());
        let raw_ind = hasher.finish() % 36;
        if raw_ind < 10 {
            *e = raw_ind as u8 + 48;
        } else {
            *e = raw_ind as u8 - 10 + 97;
        }
    });
    Boundary(inner)
}

/*
 *    fn ser_data(&mut self, sender: &mut Sender) {
 *        use std::io::Read;
 *        let del = self.len - self.offset;
 *        if del < self.buf_len && del > 0 {
 *            let mut data = Vec::new();
 *            self.file.read_to_end(&mut data).unwrap();
 *            self.offset += data.len();
 *            sender.send_data(Bytes::from(data));
 *            let tail = format!("\r\n--{}", self.boundary.to_str());
 *            sender.send_data(Bytes::from(tail));
 *            return;
 *        }
 *
 *        let mut data = Vec::with_capacity(self.buf_len);
 *        unsafe {
 *            data.set_len(self.buf_len);
 *        };
 *        self.file.read(&mut data).unwrap();
 *        sender.send_data(Bytes::from(data));
 *        let tail = format!("\r\n--{}", self.boundary.to_str());
 *        sender.send_data(Bytes::from(tail));
 *        self.offset += self.buf_len;
 *    }
 *
 */
/*
 *    async fn write_body(&mut self) -> Body {
 *        let (mut sender, body) = Body::channel();
 *        let head = format!("--{}\r\n", self.boundary.to_str());
 *        sender.send_data(Bytes::from(head));
 *
 *        let dcp = format!(
 *            "Content-Disposition: form-data;{}{}\r\n",
 *            self.ser_name(),
 *            self.ser_filename()
 *        );
 *        sender.send_data(Bytes::from(dcp));
 *        sender.send_data(Bytes::from("\r\n"));
 *
 *        // write the body
 *        self.ser_data(&mut sender);
 *
 *        body
 *
 *        //use std::io::Read;
 *        //sender.send_data(Bytes::from(data));
 *        //write!(data, "\r\n")?; // The key thing you are missing
 *        //sender.send_data("\r\n--{}--\r\n", self.boundary)?;
 *    }
 */
