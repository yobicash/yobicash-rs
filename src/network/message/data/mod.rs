use libyobicash::crypto::hash::digest::YDigest64;
use libyobicash::data::YData;
use libyobicash::errors::*;
use bytes::{BytesMut, BufMut, BigEndian, ByteOrder};
use network::method::YMethod;

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct YListDataReq {
    pub method: YMethod,
    pub tx_id: YDigest64,
}

impl YListDataReq {
    pub fn new(tx_id: YDigest64) -> YListDataReq {
        YListDataReq {
            method: YMethod::ListData,
            tx_id: tx_id,
        }
    }

    pub fn check(&self) -> YResult<()> {
        if self.method != YMethod::ListData {
            return Err(YErrorKind::Other("Invalid method".to_string()).into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> YResult<Vec<u8>> {
        self.check()?;
        let mut buf = BytesMut::new();
        buf.put(self.method.to_bytes());
        buf.put(self.tx_id.to_bytes());
        Ok(buf.to_vec())
    }

    pub fn from_bytes(buf: &[u8]) -> YResult<YListDataReq> {
        if buf.len() != 108 {
            return Err(YErrorKind::InvalidLength.into());
        }
        let mut b = BytesMut::new();
        b.extend_from_slice(buf);
        let method = BigEndian::read_u32(b.get(28..32).unwrap()).into();
        let tx_id = YDigest64::from_bytes(b.get(44..).unwrap())?;
        let ls_data_req = YListDataReq {
            method: method,
            tx_id: tx_id,
        };
        ls_data_req.check()?;
        Ok(ls_data_req)
    }
}

#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct YListDataRes {
    pub method: YMethod,
    pub count: u32,
    pub data: Vec<YData>,
}

impl YListDataRes {
    pub fn new(data: &Vec<YData>) -> YListDataRes {
        YListDataRes {
            method: YMethod::ListData,
            count: data.len() as u32,
            data: data.clone(),
        }
    }

    pub fn check(&self) -> YResult<()> {
        if self.method != YMethod::ListData {
            return Err(YErrorKind::Other("Invalid method".to_string()).into());
        }
        if self.data.len() != self.count as usize {
            return Err(YErrorKind::InvalidLength.into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> YResult<Vec<u8>> {
        self.check()?;
        let mut buf = BytesMut::new();
        buf.put(self.method.to_bytes());
        buf.put_u32::<BigEndian>(self.count as u32);
        for i in 0..self.count as usize {
            let data_buf = self.data[i].to_bytes()?;
            let data_size = data_buf.len();
            buf.put_u32::<BigEndian>(data_size as u32);
            buf.put(data_buf);
        }
        Ok(buf.to_vec())
    }

    pub fn from_bytes(buf: &[u8]) -> YResult<YListDataRes> {
        if buf.len() < 8 {
            return Err(YErrorKind::InvalidLength.into());
        }
        let mut b = BytesMut::new();
        b.extend_from_slice(buf);
        let method = BigEndian::read_u32(b.get(0..4).unwrap()).into();
        let count = BigEndian::read_u32(b.get(4..8).unwrap());
        let mut data_buf = BytesMut::new();
        data_buf.extend_from_slice(b.get(8..).unwrap());
        let mut data = Vec::new();
        for i in 0..count as usize {
            let size = BigEndian::read_u32(data_buf.get(i..i+4).unwrap()) as usize;
            data.push(YData::from_bytes(b.get(i+4..i+4+size).unwrap())?);
        }
        let ls_data_res = YListDataRes {
            method: method,
            count: count,
            data: data,
        };
        Ok(ls_data_res)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct YGetDataReq {
    pub method: YMethod,
    pub checksum: YDigest64,
}

impl YGetDataReq {
    pub fn new(checksum: YDigest64) -> YGetDataReq {
        YGetDataReq {
            method: YMethod::GetData,
            checksum: checksum,
        }
    }

    pub fn check(&self) -> YResult<()> {
        if self.method != YMethod::GetData {
            return Err(YErrorKind::Other("Invalid method".to_string()).into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> YResult<Vec<u8>> {
        self.check()?;
        let mut buf = BytesMut::new();
        buf.put(self.method.to_bytes());
        buf.put(self.checksum.to_bytes());
        Ok(buf.to_vec())
    }

    pub fn from_bytes(buf: &[u8]) -> YResult<YGetDataReq> {
        if buf.len() != 68 {
            return Err(YErrorKind::InvalidLength.into());
        }
        let mut b = BytesMut::new();
        b.extend_from_slice(buf);
        let method = BigEndian::read_u32(b.get(0..4).unwrap()).into();
        let checksum = YDigest64::from_bytes(b.get(4..).unwrap())?;
        let get_data_req = YGetDataReq {
            method: method,
            checksum: checksum,
        };
        get_data_req.check()?;
        Ok(get_data_req)
    }
}

#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct YGetDataRes {
    pub method: YMethod,
    pub data: YData,
}

impl YGetDataRes {
    pub fn new(data: &YData) -> YGetDataRes {
        YGetDataRes {
            method: YMethod::GetData,
            data: data.clone(),
        }
    }

    pub fn check(&self) -> YResult<()> {
        if self.method != YMethod::GetData {
            return Err(YErrorKind::Other("Invalid method".to_string()).into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> YResult<Vec<u8>> {
        self.check()?;
        let mut buf = BytesMut::new();
        buf.put(self.method.to_bytes());
        buf.put(self.data.to_bytes()?);
        Ok(buf.to_vec())
    }

    pub fn from_bytes(buf: &[u8]) -> YResult<YGetDataRes> {
        if buf.len() < 104 {
            return Err(YErrorKind::InvalidLength.into());
        }
        let mut b = BytesMut::new();
        b.extend_from_slice(buf);
        let method = BigEndian::read_u32(b.get(0..4).unwrap()).into();
        let data = YData::from_bytes(b.get(4..).unwrap())?;
        let get_data_res = YGetDataRes {
            method: method,
            data: data,
        };
        get_data_res.check()?;
        Ok(get_data_res)
    }
}
