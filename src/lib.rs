//! **Streamer** is a handy tool deal with stream, it provides
//! a set of structs and trait to stream
//! - `File`
//! - `String`
//! - `&'static str`
//! - `Vec<T>`
//! - `[T; N]`
//! - `Box<[T]>`
//!
//! # Quick Start
//!
//! Some example to walk you through:
//! ```bash
//! echo "1234567890abcdefghijklmnopqrstuvw" >> info
//! ```
//! ```rust no_run
//! use streamer::{Stream, StreamExt, Streaming};
//! async fn run() {
//!     let file = File::open("info").unwrap();
//!     let streaming = Streaming::from(file);
//!     streaming
//!         .chunks(5)
//!         .take(3)
//!         .for_each(|en| async move {
//!             println!("stream 5 bytes as a chunk: {:?}", std::str::from_utf8(&en).unwrap());
//!         })
//!         .await;
//! }
//! ```
//! it output:
//! ```bash
//! stream 5 bytes as a chunk: 12345
//! stream 5 bytes as a chunk: 67890
//! stream 5 bytes as a chunk: abcde
//! ```
//!
//! ```rust no_run
//! async fn run() {
//!     // let s = "1234567890abcdefghijklmnopqrstuvw"; // output the same as above
//!     // let s = "1234567890abcdefghijklmnopqrstuvw".to_string(); // output the same as above
//!     let s: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
//!     let s  = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
//!     let streaming = Streaming::from(s);
//!     streaming
//!         .chunks(4)
//!         .take(3)
//!         .for_each(|en| async move {
//!             println!("stream 5 bytes as a chunk: {:?}", &en);
//!         })
//!         .await;
//! }
//! ```
//! it output:
//! ```bash
//! [1, 2, 3, 4]
//! [5, 6, 7, 8]
//! [9, 0]
//! ```
//!
//! # Hyper Body Integration
//!
//! Since I am heavy user of [hyper](https://hyper.rs),
//! send body of request is not that convenient as others
//! especially for streaming `image`, `video` `document` and so on.
//! it enable you stream large file in chunks in HTTP Format
//!
//! To use, enable `hyper` feature
//! ```toml
//! [dependencies]
//! streamer = { version = "*", features = ["hyper"] }
//! ```
//!
//! ```rust no_run
//! use hyper::{Body, Request}:
//! let file = File::open("info").unwrap();
//! let mut streaming = Streamer::new(file);
//! streaming.meta.set_buf_len(10); // length sent as a chunk, the default is 64kB
//! streaming.meta.set_name("doc"); // field name
//! streaming.meta.set_filename("info"); // file name
//! let body: Body = streaming.streaming();
//! // build a request
//! let request: Request<Body> = Request::post("<uri-here>").body(body).expect("failed to build a request");
//! ```
//!
//! it will sennd the file `info` in 10-bytes chunks, each chunk are formated into HTTP `multipart/form-data`, with this you can split large file and stream it chunks by chunks.
//!
//! ## Features to Add
//!
//! May be some compression could be done to lower network traffic
//! - [ ] Streaming Chunks Compression
//!

pub mod streaming;
pub use streaming::Streaming;

#[cfg(feature = "hyper")]
pub mod hyper;

pub use futures_util::{Stream, StreamExt};
