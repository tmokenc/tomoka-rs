use magic::import_all;
use serde::{Serialize, Deserialize};
use crate::Result;
use futures::future;
use magic::traits::MagicIter;
use crate::constants::*;
use crate::traits::Paginator;
use db::DbInstance;
use std::fmt::Write as _;
use requester::Requester;
use serenity::model::channel::ReactionType;
use core::time::Duration;
use requester::smogon::{
    SmogonApi, 
    Generation,
    Pokemon as SmogonPokemon, 
    Move as SmogonMove,
    Ability as SmogonAbility,
    Item as SmogonItem,
    SmogonCommon,
    SmogonPokemon as SmogonPokemonDump,
};
use serenity::framework::standard::macros::group;
use crate::commands::prelude::*;
use crate::traits::Embedable;
use serenity::builder::CreateEmbed;

import_all! {
    strategy,
    ability,
    nature,
    r#move
}

const POKEMON_VERSIONS: [Generation; 8] = [
    Generation::RedBlue,
    Generation::GoldSilver,
    Generation::RubySapphire,
    Generation::DiamondPearl,
    Generation::BlackWhite,
    Generation::XY,
    Generation::SunMoon,
    Generation::SwordShield,
];

#[group]
#[prefixes("pokemon")]
#[default_command(pokemon)]
#[commands(nature, ability, moves, smogon_strategy)]
struct Pokemon;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokeKey {
    name: String,
    gen: Generation,
    kind: PokeKeyKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PokeKeyKind {
    Pokemon,
    Ability,
    Move,
    Item,
}

impl PokeKey {
    pub fn new(name: &str, gen: Generation, kind: PokeKeyKind) -> Self {
        Self {
            name: name.to_lowercase().split_whitespace().join('-'),
            gen,
            kind,
        }
    }
} 

#[command]
#[min_args(1)]
async fn pokemon(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (text, gen) = match parse_args(args.rest()) {
        Some(v) => v,
        None => return Ok(()),
    };
    
    let db = match get_data::<DatabaseKey>(ctx).await {
        Some(db) => db,
        None => return Err("Cannot get the database".into()),
    };
    
    let name = text.to_lowercase().replace(' ', "-");
    
    let (key, db) = tokio::task::spawn_blocking(move || {
        let database = db.open(SMOGON_POKEMON)?;
        
        let key = database
            .get_all_keys::<PokeKey>()
            .find(|v| &v.name == &name && gen == v.gen);
            
        key.map(|k| (k, database)).ok_or_else(|| {
            use serenity::framework::standard::CommandError;
            CommandError(format!("Cannot find the `{}` in my database", text))
        })
    }).await??;
    
    let _processed = process_data(ctx, key, msg, Some(db)).await?;
    
    Ok(())
}

async fn process_data(
    ctx: &Context,
    key: PokeKey,
    msg: &Message,
    db: Option<DbInstance>,
) -> Result<bool> {
    let db = match db {
        Some(d) => d,
        None => get_data::<DatabaseKey>(ctx)
            .await
            .expect("Database not found")
            .open(SMOGON_POKEMON)?
    };

    macro_rules! get_info {
        ($x:ident, $dump:ident) => ({
            let info: $x = match db.get(&key)? {
                Some(d) => d,
                None => return Ok(false),
            };
            
            let desc_db = db.open(SMOGON_DESCRIPTION)?;
            let mut desc: Option<SmogonCommon> = desc_db.get(&key)?;
            
            if desc.is_none() {
                let data = get_data::<ReqwestClient>(ctx)
                    .await
                    .expect("Http Requester")
                    .$dump(&info.name, key.gen)
                    .await?;
                    
                desc_db.insert(&key, &data)?;
                desc = Some(data);
            }
            
            let desc = desc.unwrap();
            
            msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
                info.append_to(embed).description(desc.description);
                
                if let Some(pokemon) = desc.pokemon.filter(|v| !v.is_empty()) {
                    let pokemons = if pokemon.len() > 50 {
                        format!(
                            "*A LOT!!!* with [{} pokemons]({}) in total!\n__Some examples__: {}",
                            pokemon.len(),
                            format_args!(
                                "https://www.smogon.com/dex/{}/{}s/{}/", 
                                key.gen.shorthand(), 
                                &stringify!($dump)[5..], 
                                &key.name
                            ),
                            pokemon.iter().take(15).join(", ")
                        )
                    } else {
                        pokemon.join(", ")
                    };
                
                    embed.field("Pokemons", pokemons, false);
                }
                
                embed
            })).await?;
        })
    }
    
    match key.kind {
        PokeKeyKind::Pokemon => process_pokemon_data(ctx, msg, key, db).await?,
        PokeKeyKind::Item => get_info!(SmogonItem, dump_item),
        PokeKeyKind::Move => get_info!(SmogonMove, dump_move),
        PokeKeyKind::Ability => get_info!(SmogonAbility, dump_ability),
    }
    
    Ok(true)
}

pub async fn process_pokemon_data(
    ctx: &Context,
    msg: &Message,
    key: PokeKey,
    db: DbInstance,
) -> Result<()> {
    let info: SmogonPokemon = match db.get(&key)? {
        Some(d) => d,
        None => return Ok(()),
    };
    
    let abilities = info
        .abilities
        .iter()
        .map(|abi| PokeKey::new(abi, key.gen, PokeKeyKind::Ability))
        .map(|ref key| match db.get::<PokeKey, SmogonAbility>(key) {
            Ok(Some(ability)) => {
                format!("- {} ({})", ability.name, ability.description)
            }
            _ => String::from(&key.name),
        })
        .join("\n");
        
    let base_stats = format!(
        "HP: {}\nAttack: {}\nDefense: {}\nSpecial Attack: {}\nSpecial Defense: {}\nSpeed: {}\n\n**Total**: {}",
        info.hp,
        info.atk,
        info.def,
        info.spa,
        info.spd,
        info.spe,
        info.base_stats_total(),
    );
    
    let sprite = format!("https://www.smogon.com/dex/media/sprites/xy/{}.gif", &key.name);
    let title = match info.dex_number() {
        Some(dex) => format!("#{} {}", dex, info.name),
        None => info.name.to_owned(),
    };
    
    let types = match crate::read_config().await.emoji.pokemon.as_ref() {
        None => info.types.join("\n"),
        Some(emoji) => info
            .types
            .iter()
            .map(|t| format!("{}  {}", emoji.get(&t).unwrap_or_default(), t))
            .join("\n"),
    };
    
    let message = msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title(title);
        embed.thumbnail(sprite);
        embed.field("Type",  types, true);
        embed.field("Base Stats", base_stats, false);
        embed.field("Abilities", abilities, false);
        embed.footer(|f| f.text(format!("Generation: {}", key.gen)));
        
        if let Some(oob) = info.oob.as_ref() {
            if !oob.evos.is_empty() {
                embed.field("Next Evolution", oob.evos.join("\n"), true);
            }
            
            if !oob.alts.is_empty() {
                embed.field("Altenative Pokemon", oob.alts.join("\n"), true);
            }
        }
        
        embed
    })).await?;
    
    let reaction = ReactionType::Unicode(String::from("⚔"));
    let duration = Duration::from_secs(30);
    let reacted = wait_for_reaction(ctx, &message, reaction, duration).await?;
    
    if let Some(user) = reacted {
        use crate::traits::paginator::PaginatorOption;
        
        let opt = PaginatorOption {
            channel_id: msg.channel_id,
            user
        };
        
        MovesPaginator::new(ctx, &info.name, key.gen, db)
            .await?
            .pagination(ctx, opt)
            .await?;
    }
    
    Ok(())
}

pub fn parse_args(args: &str) -> Option<(String, Generation)> {
    if args.is_empty() {
        return None
    }
    
    let (data, gen): (Vec<&str>, Vec<&str>) = args
        .split_whitespace()
        .partition(|v| !v.starts_with("--"));
        
    if data.is_empty() {
        return None;
    }
    
    let data = data.join(" ");
    let gen = gen
        .into_iter()
        .filter_map(|v| v.get(2..))
        .find_map(|v| v.parse::<Generation>().ok())
        .unwrap_or_default();
        
    Some((data, gen))
}

pub async fn update_pokemon<R: Requester>(db: &DbInstance, req: &R) -> Result<()> {
    let requests = POKEMON_VERSIONS
        .iter()
        .map(|v| req.dump_basics(*v));
        
    let data = future::try_join_all(requests).await?;
    let pokemon = db.open(SMOGON_POKEMON)?;
    
    tokio::task::spawn_blocking(move || {
        for (data, gen) in data.into_iter().zip(&POKEMON_VERSIONS) {
            let mut batch = db::Batch::new();
            
            macro_rules! insert {
                ($x:ident, $t:expr) => {
                    for v in data.$x {
                        let key = PokeKey::new(&v.name, *gen, $t);
                        batch.insert(&key, &v)?;
                    }
                    
                }
            };
            
            insert!(pokemon, PokeKeyKind::Pokemon);
            insert!(abilities, PokeKeyKind::Ability);
            insert!(moves, PokeKeyKind::Move);
            insert!(items, PokeKeyKind::Item);
            
            pokemon.batch(batch)?;
        }
        
        Ok(())
    }).await?
}

impl Embedable for SmogonMove {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        embed.title(&self.name);
        embed.field("Category", &self.category, true);
        embed.field("Power", self.power, true);
        embed.field("Accuracy", self.accuracy, true);
        embed.field("Priority", self.priority, true);
        embed.field("Type", &self.kind, true);
        embed.field("PP", self.pp, true);
        embed
    }
}

impl Embedable for SmogonAbility {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        embed.title(&self.name);
        embed
    }
}
impl Embedable for SmogonItem {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        embed.title(&self.name);
        embed
    }
}

pub struct MovesPaginator {
    pokemon: String,
    gen: Generation,
    list: std::sync::Mutex<Vec<(String, MoveFinding)>>,
    len: usize,
    db: DbInstance,
    icons: Option<crate::config::PokemonEmoji>,
}

pub enum MoveFinding {
    NotYet,
    Found(SmogonMove),
    NotFound,
}


impl MovesPaginator {
    pub async fn new(
        ctx: &Context,
        pokemon: &str, 
        gen: Generation, 
        db: DbInstance,
    ) -> Result<Self> {
        let key = PokeKey::new(pokemon, gen, PokeKeyKind::Pokemon);
    
        let desc_db = db.open(SMOGON_DESCRIPTION)?;
        
        let learnset = match desc_db.get::<PokeKey, SmogonPokemonDump>(&key)? {
            Some(pokemon) => pokemon.learnset,
            None => {
                let data = get_data::<ReqwestClient>(ctx)
                    .await
                    .unwrap()
                    .dump_pokemon(&key.name, gen)
                    .await?;
                
                desc_db.insert(&key, &data)?;
                data.learnset
            }
        };
        
        let icons = crate::read_config().await.emoji.pokemon.to_owned();
        let list: Vec<_> = learnset.into_iter().map(|v| (v, MoveFinding::NotYet)).collect();
        let len = list.len();
        
        Ok(Self {
            pokemon: pokemon.to_owned(),
            gen,
            list: std::sync::Mutex::new(list),
            len,
            db,
            icons
        })
    }
}

impl Paginator for MovesPaginator {
    fn append_page_data<'a>(
        &mut self,
        page: core::num::NonZeroUsize,
        embed: &'a mut CreateEmbed,
    ) -> &'a mut CreateEmbed {
        let page = page.get();
        let index = (page-1) * POKEMON_MOVE_PER_PAGE;
        
        let description = self
            .list
            .lock()
            .unwrap()[index..]
            .iter_mut()
            .take(POKEMON_MOVE_PER_PAGE)
            .map(|(name, ref mut desc)| {
                if let MoveFinding::NotYet = desc {
                    let key = PokeKey::new(&name, self.gen, PokeKeyKind::Move);
                    *desc = match self.db.get::<PokeKey, SmogonMove>(&key) {
                        Ok(Some(m)) => MoveFinding::Found(m),
                        Ok(None) => MoveFinding::NotFound,
                        Err(why) => {
                            error!("Cannot get a pokemon move\n{:#?}", why);
                            MoveFinding::NotYet
                        }
                    };
                }
                
                match desc {
                    MoveFinding::Found(m) => {
                        let mut info = match &self.icons {
                            Some(ref icons) => format!(
                                "{} {}  **{}** -",
                                icons.get(&m.kind).unwrap_or_default(),
                                icons.get(&m.category).unwrap_or_default(),
                                m.name,
                            ),
                            None => format!(
                                "**{}** ({} {})",
                                m.name,
                                m.category,
                                m.kind,
                            )
                        };
                        
                        if m.category != "Non-Damaging" {
                            write!(&mut info, " Power: {},", m.power).unwrap();
                        }
                        
                        if m.accuracy != 0 {
                            write!(&mut info, " Accuracy: {}%", m.accuracy).unwrap();
                        } else {
                            info.push_str(" Accuracy: ---%");
                        }
                        
                        if m.priority != 0 {
                            write!(&mut info, ", Priority: {}", m.priority).unwrap();
                        }
                        
                        write!(&mut info, "\n- {}", m.description).unwrap();
                        
                        info
                    }
                    _ => format!("- {} (Cannot find the information of this move)", name),
                }
            })
            .join("\n\n");
        
        embed.title(format!("Learn set for {}", &self.pokemon));
        embed.description(description);
        embed.footer(|f| f.text(format!("Page {} / {}", page, self.total_pages().unwrap())));
        
        embed
    }
    
    fn total_pages(&self) -> Option<usize> {
        Some(((self.len - 1) / POKEMON_MOVE_PER_PAGE) + 1)
    }
}


