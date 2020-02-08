use failure::format_err;
use rkv::{Manager, OwnedValue, Rkv, SingleStore, StoreOptions, Value};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};

type Result<T> = std::result::Result<T, failure::Error>;
pub type DbManager = Arc<RwLock<Rkv>>;

pub struct DbInstance {
    pub store: SingleStore,
    pub manager: DbManager,
}

impl DbInstance {
    pub fn new<'a, P, N>(path: P, name: N) -> Result<Self>
    where
        P: AsRef<Path>,
        N: Into<Option<&'a str>>,
    {
        let db_name = name.into().unwrap_or("tomoka");
        let manager = get_db_manager(path)?;
        let result = Self::from_manager(manager, db_name)?;
        Ok(result)
    }

    pub fn from_manager<N: AsRef<str>>(manager: DbManager, name: N) -> Result<Self> {
        let store = manager
            .read()
            .unwrap()
            .open_single(name.as_ref(), StoreOptions::create())?;

        Ok(Self { manager, store })
    }

    pub fn open(&self, name: impl AsRef<str>) -> Result<Self> {
        let res = Self::from_manager(self.manager.clone(), name)?;
        Ok(res)
    }

    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<OwnedValue> {
        let env = self.manager.read().unwrap();
        let reader = env.read()?;

        let data = self
            .store
            .get(&reader, key)?
            .as_ref()
            .map(OwnedValue::from)
            .ok_or_else(|| format_err!("Not found any value"))?;

        Ok(data)
    }

    pub fn get_json<K, D>(&self, key: K) -> Result<D>
    where
        K: AsRef<[u8]>,
        D: DeserializeOwned,
    {
        let data = self.get(key)?;
        let value = Value::from(&data).to_bytes()?;
        let result = serde_json::from_slice(value.as_slice())?;
        Ok(result)
    }

    pub fn get_all(&self) -> Result<Vec<(Vec<u8>, OwnedValue)>> {
        let env = self.manager.read().unwrap();
        let reader = env.read()?;

        let data = self
            .store
            .iter_start(&reader)?
            .filter_map(|v| v.ok())
            .filter_map(|(k, v)| v.map(|v| (k.to_owned(), OwnedValue::from(&v))))
            .collect();

        Ok(data)
    }

    pub fn get_all_json<D: DeserializeOwned>(&self) -> Result<Vec<(Vec<u8>, D)>> {
        let data = self
            .get_all()?
            .into_iter()
            .filter_map(|(k, v)| {
                if let OwnedValue::Json(s) = v {
                    serde_json::from_str(&s).ok().map(|v| (k, v))
                } else {
                    None
                }
            })
            .collect();

        Ok(data)
    }

    // ? don't know why it doesn't work
    // pub fn get_all_json<D: DeserializeOwned>(&self) -> Result<Vec<(Vec<u8>, D)>> {
    //     let env = self.manager.read().unwrap();
    //     let reader = env.read()?;

    //     let data = self
    //         .store
    //         .iter_start(&reader)?
    //         .filter_map(|v| v.ok())
    //         .filter_map(|(k, v)| {
    //             v
    //             .and_then(|val| val.to_bytes().ok())
    //             .and_then(|val| serde_json::from_slice(val.as_slice()).ok())
    //             .map(|val| (k.to_vec(), val))
    //         })
    //         .collect();

    //     Ok(data)
    // }

    pub fn put<K: AsRef<[u8]>>(&self, key: K, value: &Value) -> Result<()> {
        let env = self.manager.read().unwrap();
        let mut writer = env.write()?;
        self.store.put(&mut writer, key, value)?;
        writer.commit()?;
        Ok(())
    }

    pub fn put_json<K, V>(&self, key: K, value: &V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: Serialize,
    {
        let json = serde_json::to_string(value)?;
        let val = Value::Json(json.as_str());
        self.put(key, &val)?;
        Ok(())
    }

    pub fn delete<K: AsRef<[u8]>>(&self, key: K) -> Result<()> {
        let env = self.manager.read().unwrap();
        let mut writer = env.write()?;
        self.store.delete(&mut writer, key)?;
        writer.commit()?;
        Ok(())
    }
}

pub fn get_db_manager<P: AsRef<Path>>(path: P) -> Result<DbManager> {
    let db_path = path.as_ref();
    fs::create_dir_all(db_path)?;

    let manager = Manager::singleton()
        .write()
        .unwrap()
        .get_or_create(db_path, Rkv::new)?;

    Ok(manager)
}
