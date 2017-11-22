
// Copied from
// https://mmstick.tk/post/jmP
// and andded the gzip(), deflate(), brotli(), and the check_*() methods

use rocket::request::{FromRequest, Request};
use rocket::http::Status;
use rocket::Outcome;

const GZIP:    u8 = 1;
const DEFLATE: u8 = 2;
const BROTLI:  u8 = 4;

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum CompressionEncoding { Brotli, Gzip, Deflate, Uncompressed }

#[derive(Copy, Clone, Debug)]
pub struct AcceptCompression {
    supported: u8
}

impl AcceptCompression {
    pub fn contains_gzip(self)    -> bool { self.supported & GZIP != 0 }
    pub fn contains_deflate(self) -> bool { self.supported & DEFLATE != 0 }
    pub fn contains_brotli(self)  -> bool { self.supported & BROTLI != 0 }
    pub fn is_uncompressed(self)  -> bool { self.supported == 0 }
    pub fn preferred(self) -> CompressionEncoding {
        if self.supported & BROTLI != 0 {
            CompressionEncoding::Brotli
        } else if self.supported & GZIP != 0 {
            CompressionEncoding::Gzip
        } else if self.supported & DEFLATE != 0 {
            CompressionEncoding::Deflate
        } else {
            CompressionEncoding::Uncompressed
        }
    }
    /// Returns a new AcceptCompression that specifies the gzip method
    #[inline(always)]
    pub fn gzip() -> Self { AcceptCompression { supported: GZIP } }
    /// Returns a new AcceptCompression that specifies the deflate method
    #[inline(always)]
    pub fn deflate() -> Self { AcceptCompression { supported: DEFLATE } }
    /// Returns a new AcceptCompression that specifies the brotli method
    #[inline(always)]
    pub fn brotli() -> Self { AcceptCompression { supported: BROTLI } }
    
    /// Returns a new AcceptCompression that uses no compression
    #[inline(always)]
    pub fn no_compression() -> Self { AcceptCompression { supported: 0 } }
    
    /// Returns a new AcceptCompression that specifies the gzip method
    pub fn checked_gzip(&self) -> Self { 
        if self.contains_gzip() { 
            AcceptCompression { 
                supported: GZIP 
            } 
        } else { 
            AcceptCompression::no_compression() 
        } 
    }
    /// Returns a new AcceptCompression that specifies the deflate method
    pub fn checked_deflate(&self) -> Self {
        if self.contains_deflate() { 
            AcceptCompression { 
                supported: DEFLATE 
            } 
        } else { 
            AcceptCompression::no_compression() 
        } 
    }
    /// Returns a new AcceptCompression that specifies the brotli method
    pub fn checked_brotli(&self) -> Self { 
        if self.contains_brotli() { 
            AcceptCompression { 
                supported: BROTLI 
            } 
        } else { 
            AcceptCompression::no_compression() 
        } 
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AcceptCompression {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> Outcome<AcceptCompression, (Status, ()), ()> {
        let mut supported = 0u8;
        if let Some(encoding) = request.headers().get("Accept-Encoding").next() {
            if encoding.contains("gzip") { supported |= GZIP; }
            if encoding.contains("deflate") { supported |= DEFLATE; }
            if encoding.contains("br") { supported |= BROTLI; }
        }
        Outcome::Success(AcceptCompression { supported })
    }
}


