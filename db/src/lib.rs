use lazy_static::lazy_static;
use log::error;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use sled::{Db, Tree};
use std::error::Error;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

type Manager = Arc<Db>;
type Result<T> = std::result::Result<T, Box<dyn Error + Sync + Send>>;

lazy_static! {
    static ref ENCODER: bincode::Config = {
        let mut c = bincode::config();
        c.big_endian();
        c
    };
}

pub struct DbInstance {
    tree: Option<Tree>,
    manager: Manager,
}

impl Drop for DbInstance {
    fn drop(&mut self) {
        self.manager.flush().ok();
    }
}

pub struct Iter<K: DeserializeOwned, V: DeserializeOwned> {
    iter: sled::Iter,
    _marker: PhantomData<(K, V)>,
}

impl<K: DeserializeOwned, V: DeserializeOwned> Iter<K, V> {
    pub(crate) fn new(iter: sled::Iter) -> Self {
        Self {
            iter,
            _marker: PhantomData,
        }
    }
}

impl<K: DeserializeOwned, V: DeserializeOwned> Iterator for Iter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .by_ref()
            .filter_map(|v| v.ok())
            .find_map(|(ref key, ref val)| {
                let k = match ENCODER.deserialize(key) {
                    Ok(e) => e,
                    Err(why) => {
                        match *why {
                            bincode::ErrorKind::Custom(_) => {}
                            e => error!("Cannot deserialize data | {}", e),
                        }

                        return None;
                    }
                };

                let v = match ENCODER.deserialize(val) {
                    Ok(e) => e,
                    Err(why) => {
                        match *why {
                            bincode::ErrorKind::Custom(_) => {}
                            e => error!("Cannot deserialize data | {}", e),
                        }

                        return None;
                    }
                };

                Some((k, v))
            })
    }
}

impl DbInstance {
    pub fn new<'a, P, N>(path: P, name: N) -> Result<Self>
    where
        P: AsRef<Path>,
        N: Into<Option<&'a [u8]>>,
    {
        let manager = get_db_manager(path)?;
        let db = Self::from_manager(manager, name.into())?;
        Ok(db)
    }

    pub fn from_manager<N: AsRef<[u8]>>(manager: Manager, name: Option<N>) -> Result<Self> {
        let tree = match name {
            Some(n) => Some(manager.open_tree(n)?),
            None => None,
        };

        Ok(Self { tree, manager })
    }

    pub fn open<N: AsRef<[u8]>>(&self, tree: N) -> Result<Self> {
        let manager = Arc::clone(&self.manager);
        Self::from_manager(manager, Some(tree))
    }

    pub fn get<K, V>(&self, key: &K) -> Result<Option<V>>
    where
        K: Serialize,
        V: DeserializeOwned,
    {
        let k = ENCODER.serialize(key)?;
        let res = self
            .tree()
            .get(&k)?
            .and_then(|ref v| ENCODER.deserialize(v).ok());

        Ok(res)
    }

    pub fn get_all<K, V>(&self) -> Iter<K, V>
    where
        K: DeserializeOwned,
        V: DeserializeOwned,
    {
        let iter = self.tree().iter();
        Iter::<K, V>::new(iter)
    }

    pub fn insert<K, V>(&self, key: &K, value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        let k = ENCODER.serialize(key)?;
        let v = ENCODER.serialize(value)?;
        self.tree().insert(&k, v)?;
        Ok(())
    }

    pub fn remove<K: Serialize>(&self, key: &K) -> Result<()> {
        let k = ENCODER.serialize(key)?;
        self.tree().remove(&k)?;
        Ok(())
    }

    pub fn tree(&self) -> &Tree {
        match &self.tree {
            Some(t) => t,
            None => &**self.manager,
        }
    }
}

#[inline]
pub fn get_db_manager(path: impl AsRef<Path>) -> Result<Manager> {
    sled::Config::new()
        .path(path)
        .use_compression(true)
        .open()
        .map(Arc::new)
        .map_err(|v| Box::new(v) as Box<_>)
}
