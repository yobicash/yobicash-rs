use bytes::{BytesMut, BufMut, BigEndian, ByteOrder};
use network::rpc_method::YRPCMethod;
use errors::*;

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct YHErrorRes {
    pub method: YRPCMethod,
    pub message: String,
}

impl YHErrorRes {
    pub fn new(method: YRPCMethod, message: String) -> YHErrorRes {
        YHErrorRes {
            method: method,
            message: message,
        }
    }

    pub fn check(&self) -> YHResult<()> {
        if self.message.len() > 20 {
            return Err(YHErrorKind::InvalidLength.into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> YHResult<Vec<u8>> {
        let mut buf = BytesMut::new();
        buf.put(self.method.to_bytes());
        buf.put(self.message.as_bytes());
        Ok(buf.to_vec())
    }

    pub fn from_bytes(buf: &[u8]) -> YHResult<YHErrorRes> {
        if buf.len() > 44 {
            return Err(YHErrorKind::InvalidLength.into());
        }
        let mut b = BytesMut::new();
        b.extend_from_slice(buf);
        let method = BigEndian::read_u32(b.get(0..4).unwrap()).into();
        let message = String::from_utf8_lossy(b.get(8..).unwrap()).into_owned();
        Ok(YHErrorRes {
            method: method,
            message: message,
        })
    }
}
