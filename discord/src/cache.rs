use std::collections::BTreeMap;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::{Path, PathBuf};

use serenity::model::channel::{Attachment, Message};
use serenity::model::id::{AttachmentId, MessageId, UserId};

use log::info;
use tempdir::TempDir;
use parking_lot::Mutex;

use crate::utils::save_file;
use crate::Result;

/// An atomic custom cache for the logging purpose
/// All of its method only use a `&self`
/// so it can be wrap into an Arc for multithreading
pub struct MyCache {
    /// Mutex because we always need the write access to it
    pub message: Mutex<BTreeMap<MessageId, MessageCache>>,
    max_message: AtomicUsize,
    tmp_dir: PathBuf,
}

impl Drop for MyCache {
    fn drop(&mut self) {
        info!("Dropping the cache");
        if let Err(why) = fs::remove_dir_all(&self.tmp_dir) {
            error!("Cannot clean up the custom cache\n{:#?}", why);
        }
    }
}

/// Another version of the Message struct, but with the fields we actually need for the logger
/// only the message from guild are allowed
/// we don't need to store the channel id nor guild id here, since we have it at the events handler
pub struct MessageCache {
    pub attachments: Vec<AttachmentCache>,
    pub content: String,
    ///we only need the ID, and then fetch them from serenity cache or REST API
    pub author_id: UserId,
}

impl From<Message> for MessageCache {
    fn from(msg: Message) -> Self {
        let attachments = msg
            .attachments
            .into_iter()
            .map(AttachmentCache::from)
            .collect();

        Self {
            attachments,
            content: msg.content,
            author_id: msg.author.id,
        }
    }
}

pub struct AttachmentCache {
    pub id: AttachmentId,
    pub url: String,
    pub size: u32,
    pub cached: Option<PathBuf>,
}

impl From<Attachment> for AttachmentCache {
    fn from(a: Attachment) -> Self {
        Self {
            id: a.id,
            url: a.url,
            size: a.size as u32,
            cached: None,
        }
    }
}

impl Drop for AttachmentCache {
    /// This is the **BEST** thing that I have seen in Rust
    /// Do a post-hook when a memory is goen
    /// This way we don't have to do manually remove the file
    /// **Which save me from writing a lot of code and bugs as well as errors**
    /// just as easy as remove this and drop it, then the file will automatically goen
    fn drop(&mut self) {
        if let Some(file) = &self.cached {
            if let Err(why) = fs::remove_file(&file) {
                error!("Cannot remove a file in cache: {:?}\n{:#?}", file, why);
            }
        }
    }
}

impl AttachmentCache {
    /// Get the file name of the attachment
    /// This is not the actual filename, just an id of it for caching
    pub fn filename(&self) -> String {
        let ext = self.url.split('.').last().unwrap_or("jpg");
        format!("{}.{}", self.id, ext)
    }
}

impl MyCache {
    /// Create a new custom cache, as well as a cache directory
    /// Default max_message is 2000
    pub fn new() -> Result<Self> {
        let tmp_dir = create_tmp_dir(crate::read_config().temp_dir.as_ref(), "tomoka-cache")?;
        info!("the temp dir path:\n{:?}", tmp_dir.path());

        Ok(Self {
            message: Mutex::new(BTreeMap::new()),
            max_message: AtomicUsize::new(2000),
            tmp_dir: tmp_dir.into_path(),
        })
    }

    /// Clear the cache
    /// Return the length of messages and cached size on disk
    pub fn clear(&self) -> Result<(usize, usize)> {
        let cache_size = fs::read_dir(self.tmp_dir.as_path())?
            .filter_map(|v| v.ok())
            .filter_map(|v| v.metadata().ok())
            .map(|v| v.len() as usize)
            .sum();

        let mut message = self.message.lock();
        let cache_length = message.len();
        message.clear();
        drop(message);

        Ok((cache_length, cache_size))
    }
    
    /// This will also delete the cache directory
    pub fn clean_up(&self) {
        if let Err(why) = fs::remove_dir_all(&self.tmp_dir) {
            error!("Cannot clean up the cache\n{:#?}", why);
        }
    }
 
    /// Set new maximum message allow in the cache
    /// return the old value
    pub fn set_max_message(&self, value: usize) -> usize {
        let old_value = self.max_message.swap(value, Ordering::SeqCst);
        let mut message = self.message.lock();

        if message.len() > value {
            let drop_size = message.len() - value;
            let keys: MessageId = message.keys().nth(drop_size).copied().unwrap();
            *message = message.split_off(&keys);
        }

        old_value
    }

    /// Insert a message to the cache
    pub fn insert_message(&self, msg: Message) {
        let mut message = self.message.lock();
        if message.len() >= self.max_message.load(Ordering::Acquire) {
            let key_to_remove = *message.keys().next().unwrap();
            message.remove(&key_to_remove);
        }

        let id = msg.id;
        let mut cache_message = MessageCache::from(msg);

        for i in cache_message.attachments.iter_mut() {
            let max_file_size = {
                let config = crate::read_config();
                config.etc.max_cache_file_size
            };
            
            if i.size <= max_file_size {
                let path = self.tmp_dir.join(i.filename());
                save_file(i.url.to_owned(), path.to_owned());
                i.cached = Some(path);
            }
        }

        message.insert(id, cache_message);
    }
    
    /// Update the message content, return the old cached content
    pub fn update_message(&self, id: MessageId, content: &str) -> Option<String> {
        let mut message = self.message.lock();
        
        message.get_mut(&id)
            .map(|ref mut v| {
                let old = v.content.to_owned();
                v.content = content.to_owned();
                old
            })
    }

    /// Remove the message from cache by a given MessageId
    /// Return the cached message if exist
    pub fn remove_message<I: Into<MessageId>>(&self, msg: I) -> Option<MessageCache> {
        let id = msg.into();
        self.message.lock().remove(&id)
    }
}

fn create_tmp_dir<P: AsRef<Path>>(dir: Option<P>, prefix: &str) -> std::io::Result<TempDir> {
    match dir {
        Some(d) => TempDir::new_in(d, prefix),
        None => TempDir::new(prefix),
    }
}
