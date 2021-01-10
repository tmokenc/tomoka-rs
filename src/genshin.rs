use async_trait::async_trait;
use db::DbInstance;
use crate::traits::{Embedable, CreateEmbed, RawEventHandlerRef};
use std::sync::atomic::{Ordering, AtomicBool};
use std::fmt::{self, Write};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use chrono::{Duration, Utc};
use serenity::client::Context;
use serenity::http::client::Http;
use serenity::model::{
    id::{ChannelId},
    event::Event,
    // channel::{Reaction},
};

const AN_HOUR: i64 = 60 * 60;
const A_DAY: i64 = AN_HOUR * 24;
const A_WEEK: i64 = A_DAY * 7   ;

const THUMBNAIL: &str = "https://static.wikia.nocookie.net/gensin-impact/images/d/d4/Item_Primogem.png";
const IMAGE: &str = "https://static2.gamerantimages.com/wordpress/wp-content/uploads/2020/12/Genshin-Impact-Ganyu-Albedo.jpg";
// const IMAGE: &str = "https://upload-os-bbs.mihoyo.com/upload/2020/01/17/1024407/a84a875b20ff6605f658188077c479d9_6340425692462648258.png";
const FOOTER_IMAGE: &str = "https://static.wikia.nocookie.net/gensin-impact/images/3/35/Item_Fragile_Resin.png";

enum Region {
    Asia, EU, NA,
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Asia => write!(f, "Asia (GMT+8)"),
            Self::EU => write!(f, "Europe (GMT+1)"),
            Self::NA => write!(f, "North America (GMT-7)"),
        }
    }
}

impl Region {
    fn time_diff(&self) -> i64 {
        match self {
            Self::Asia => 20,
            Self::EU => 3 + 24,
            Self::NA => 9 + 24,
        }
    }
    
    fn time_to_next_daily(&self) -> Duration {
        let current = Utc::now().timestamp();
        
        let region_diff = AN_HOUR * self.time_diff();
        let next_daily = current - (current % A_DAY) + A_DAY + region_diff;
        
        Duration::seconds((next_daily - current) % A_DAY)
    }
    
    fn time_to_next_weekly(&self) -> Duration {
        let current = Utc::now().timestamp();
            
        let region_diff = AN_HOUR * self.time_diff();
        let next_weekly = current - (current % A_WEEK) + A_WEEK + (A_DAY * 3) + region_diff;
        
        Duration::seconds((next_weekly - current) % A_WEEK)
    }
}

pub struct RegionsEmbed;

impl Embedable for RegionsEmbed {
    fn append(&self, embed: &mut CreateEmbed) {
        embed.timestamp(&Utc::now());
        embed.title("Genshin Impact Timer");
        embed.thumbnail(THUMBNAIL);
        embed.image(IMAGE);
        embed.footer(|f| f.icon_url(FOOTER_IMAGE));
        
        for region in &[Region::Asia, Region::EU, Region::NA] {
            let desc = format!(
                "Daily reset in **{}**\nWeekly reset in **{}**", 
                format_time(region.time_to_next_daily()),
                format_time(region.time_to_next_weekly()),
            );
            
            embed.field(region.to_string(), desc, false);
        }
    }
}

pub struct GenshinEvent {
    spawned: AtomicBool,
    db: DbInstance,
}

impl GenshinEvent {
    pub fn new(db: &DbInstance) -> crate::Result<Self> {
        Ok(Self {
            spawned: AtomicBool::default(),
            db: db.open("genshin_watch")?,
        })
    }
}

#[async_trait]
impl RawEventHandlerRef for GenshinEvent {
    async fn raw_event_ref(&self, ctx: &Context, ev: &Event) {
        match ev {
            Event::Ready(_e) if !self.spawned.load(Ordering::Relaxed) => {
                self.spawned.store(true, Ordering::Relaxed);
                
                let http = Arc::clone(&ctx.http);
                let db = self.db.clone();
                
                tokio::spawn(async move {
                    tokio::time::sleep(time_to_next_minute()).await;
                    update_time(http, db).await;
                });
            }
            
            Event::MessageDelete(e) => {
                let msg: Option<u64> = self.db.get(e.channel_id.as_u64()).ok().flatten();
                if let Some(msg) = msg {
                    if msg == e.message_id.0 {
                        if let Err(why) = self.db.remove(e.channel_id.as_u64()) {
                            log::error!("Error while removing a genshin channel:\n{:#?}", why);
                        }
                    }
                }
                
            }
            
            Event::MessageDeleteBulk(e) => {
                let msg: Option<u64> = self.db.get(e.channel_id.as_u64()).ok().flatten();
                if let Some(msg) = msg {
                    if e.ids.iter().find(|v| v.0 == msg).is_some() {
                        if let Err(why) = self.db.remove(e.channel_id.as_u64()) {
                            log::error!("Error while removing a genshin channel:\n{:#?}", why);
                        }
                    }
                }
            }
            
            _ => {}
        }
    }
}

fn time_to_next_minute() -> StdDuration {
    let current = Utc::now().timestamp();
    let next = current - (current % 60) + 60;
    StdDuration::from_secs((next - current) as u64)
}

async fn update_time(http: Arc<Http>, db: DbInstance) {
    loop {
        for (channel, msg) in db.get_all::<u64, u64>() {
            let http = Arc::clone(&http);
            tokio::spawn(async move {
                let send = ChannelId(channel)
                    .edit_message(http, msg, |f| f.embed(|e| RegionsEmbed.append_to(e)))
                    .await;
                // let send = ChannelExt::edit_message(&channel, http, MessageId(msg))
                //     .with_embed(RegionsEmbed)
                //     .await;
                
                if let Err(why) = send {
                    log::error!("Error while editing genshin timer on channel {}\n{:#?}", channel, why);
                }
            });
        }
        
        tokio::time::sleep(time_to_next_minute()).await;
    }
}

fn format_time(t: Duration) -> String {
    let mut res = String::new();
    
    let days = t.num_days();
    let hours = t.num_hours() % 24;
    let minutes = t.num_minutes() % 60;
    
    match days {
        1 => write!(&mut res, "{} day ", 1),
        n if n > 1 => write!(&mut res, "{} days ", n),
        _ => fmt::Result::Ok(()),
    }.ok();
    
    match hours {
        1 => write!(&mut res, "{} hour ", 1),
        n if n > 1 => write!(&mut res, "{} hours ", n),
        _ => fmt::Result::Ok(()),
    }.ok();
    
    match minutes {
        1 => write!(&mut res, "{} minute ", 1),
        n if n > 1 => write!(&mut res, "{} minutes ", n),
        _ => fmt::Result::Ok(()),
    }.ok();
    
    if res.is_empty() {
        res = "now".to_string();
    }
    
    res
}