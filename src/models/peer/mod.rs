use libyobicash::errors::YErrorKind as LibErrorKind;
use libyobicash::utils::time::YTime;
use libyobicash::utils::random::YRandom;
use serde_json;
use bytes::{BytesMut, BufMut, BigEndian};
use std::net::Ipv4Addr;
use store::common::*;
use models::bucket::*;
use network::host::*;
use errors::*;

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct YPeer {
    pub host: YHost,
    pub last_time: YTime,
}

impl Default for YPeer {
    fn default() -> YPeer {
        YPeer::new(YHost::default())
    }
}

impl YPeer {
    pub fn new(host: YHost) -> YPeer {
        YPeer {
            host: host,
            last_time: YTime::now(),
        }
    }

    pub fn check(&self) -> YHResult<()> {
        if self.last_time > YTime::now() {
            return Err(YHErrorKind::Lib(LibErrorKind::InvalidTime).into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> YHResult<Vec<u8>> {
        let mut buf = BytesMut::new();
        buf.put(&self.host.to_bytes()?);
        buf.put(&self.last_time.to_bytes()[..]);
        Ok(buf.to_vec())
    }

    pub fn from_bytes(buf: &[u8]) -> YHResult<YPeer> {
        if buf.len() != 14 {
            return Err(YHErrorKind::InvalidLength.into());
        }
        let mut b = BytesMut::new();
        b.extend_from_slice(buf);
        let host = YHost::from_bytes(b.get(0..6).unwrap())?;
        let last_time = YTime::from_bytes(b.get(6..).unwrap())?;
        let peer = YPeer {
            host: host,
            last_time: last_time,
        };
        peer.check()?;
        Ok(peer)
    }

    pub fn to_json(&self) -> YHResult<String> {
        let json = serde_json::to_string(self)?;
        Ok(json)
    }

    pub fn from_json(s: &str) -> YHResult<YPeer> {
        let peer = serde_json::from_str(s)?;
        Ok(peer)
    }

    pub fn by_ip_key(&self) -> YHResult<YStoreKey> {
        self.check()?;
        let mut key = Vec::new();
        key.put(&self.host.ip()?.octets()[..]);
        Ok(key)
    }

    pub fn by_last_time_key(&self) -> YHResult<YStoreKey> {
        self.check()?;
        let mut key = BytesMut::new();
        key.put(&self.last_time.to_bytes()[..]);
        key.put_u32::<BigEndian>(YRandom::u32());
        Ok(key.to_vec())
    }

    pub fn value(&self) -> YHResult<YStoreValue> {
        self.to_bytes()
    }

    pub fn from_value(value: &YStoreValue) -> YHResult<YPeer> {
        YPeer::from_bytes(value)
    }

    pub fn lookup_by_ip<S: YStorage>(store: &S, ip: Ipv4Addr) -> YHResult<bool> {
        let store_buck = YBucket::PeersByIp.to_store_buck();
        let mut key = Vec::new();
        key.put(&ip.octets()[..]);
        store.lookup(&store_buck, &key)
    }

    pub fn lookup_by_last_time<S: YStorage>(store: &S, last_time: YTime) -> YHResult<bool> {
        let store_buck = YBucket::PeersByLastTime.to_store_buck();
        let mut key = Vec::new();
        key.put(&last_time.to_bytes()[..]);
        store.lookup(&store_buck, &key)
    }

    pub fn count_by_ip<S: YStorage>(store: &S) -> YHResult<u32> {
        let store_buck = YBucket::PeersByIp.to_store_buck();
        store.count(&store_buck)
    }

    pub fn count_by_last_time<S: YStorage>(store: &S) -> YHResult<u32> {
        let store_buck = YBucket::PeersByLastTime.to_store_buck();
        store.count(&store_buck)
    }

    pub fn list_by_ip<S: YStorage>(store: &S, skip: u32, count: u32) -> YHResult<Vec<YPeer>> {
        let store_buck = YBucket::PeersByIp.to_store_buck();
        let keys = store.list(&store_buck, skip, count)?;
        let mut peers = Vec::new();        
        for key in keys {
            let item = store.get(&store_buck, &key)?;
            let peer = YPeer::from_value(&item.value)?;
            peers.push(peer);
        }
        Ok(peers)
    }

    pub fn list_by_last_time<S: YStorage>(store: &S, skip: u32, count: u32) -> YHResult<Vec<Ipv4Addr>> {
        let store_buck = YBucket::PeersByLastTime.to_store_buck();
        let _keys = store.list_reverse(&store_buck, skip, count)?;
        let mut keys: Vec<Ipv4Addr> = Vec::new();        
        for _key in _keys {
            let octets_buf = store.get(&store_buck, &_key)?.key;
            let octets = [octets_buf[0], octets_buf[1], octets_buf[2], octets_buf[3]];
            let key = Ipv4Addr::from(octets);
            keys.push(key);
        }
        Ok(keys)
    }

    pub fn get<S: YStorage>(store: &S, ip: Ipv4Addr) -> YHResult<YPeer> {
        let store_buck = YBucket::PeersByLastTime.to_store_buck();
        let mut key = Vec::new();
        key.put(&ip.octets()[..]);
        let item = store.get(&store_buck, &key)?;
        YPeer::from_value(&item.value)
    }

    pub fn create<S: YStorage>(&self, store: &mut S) -> YHResult<()> {
        let store_buck_ip = YBucket::PeersByIp.to_store_buck();
        let key_ip = self.by_ip_key()?;
        if store.lookup(&store_buck_ip, &key_ip)? {
            return Err(YHErrorKind::AlreadyFound.into());
        }
        let value = self.value()?;
        store.put(&store_buck_ip, &key_ip, &value)?;
        let store_buck_lt = YBucket::PeersByLastTime.to_store_buck();
        let key_lt = self.by_last_time_key()?;
        if store.lookup(&store_buck_lt, &key_lt)? {
            return Err(YHErrorKind::AlreadyFound.into());
        }
        store.put(&store_buck_lt, &key_lt, &key_ip)?;
        Ok(())
    }

    pub fn update<S: YStorage>(&self, store: &mut S) -> YHResult<()> {
        let store_buck_ip = YBucket::PeersByIp.to_store_buck();
        let key_ip = self.by_ip_key()?;
        if !store.lookup(&store_buck_ip, &key_ip)? {
            return Err(YHErrorKind::NotFound.into());
        }
        self.delete(store)?;
        self.create(store)
    }

    pub fn delete<S: YStorage>(&self, store: &mut S) -> YHResult<()> {
        let store_buck_ip = YBucket::PeersByIp.to_store_buck();
        let key_ip = self.by_ip_key()?;
        if !store.lookup(&store_buck_ip, &key_ip)? {
            return Err(YHErrorKind::NotFound.into());
        }
        store.delete(&store_buck_ip, &key_ip)?;
        let store_buck_lt = YBucket::PeersByLastTime.to_store_buck();
        let key_lt = self.by_last_time_key()?;
        if !store.lookup(&store_buck_lt, &key_lt)? {
            return Err(YHErrorKind::NotFound.into());
        }
        store.delete(&store_buck_lt, &key_lt)?;
        Ok(())
    }
}
