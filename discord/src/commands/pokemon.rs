use magic::import_all;
use serde::{Serialize, Deserialize};
use crate::Result;
use futures::future;
use magic::traits::MagicIter;
use crate::constants::*;
use db::DbInstance;
use requester::Requester;
use requester::smogon::{
    SmogonApi, 
    Generation,
    Pokemon as SmogonPokemon, 
    Move as SmogonMove,
    Ability as SmogonAbility,
    Item as SmogonItem,
    SmogonCommon,
};
use serenity::framework::standard::macros::group;
use crate::commands::prelude::*;
use crate::traits::Embedable;
use serenity::builder::CreateEmbed;
use serenity::model::id::ChannelId;

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
    
    process_data(ctx, key, msg.channel_id, Some(db)).await?;
    
    Ok(())
}

async fn process_data(
    ctx: &Context,
    key: PokeKey,
    channel_id: ChannelId,
    db: Option<DbInstance>,
) -> Result<bool> {
    let db = match db {
        Some(d) => d,
        None => {
            let database = get_data::<DatabaseKey>(ctx).await.expect("Database not found");
            tokio::task::spawn_blocking(move || database.open(SMOGON_POKEMON)).await??
        }
    };

    macro_rules! get_info {
        () => ({
            match db.get(&key)? {
                Some(d) => d,
                None => return Ok(false),
            }
        });
        
        ($x:ident, $dump:ident) => ({
            let info: $x = get_info!();
            
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
            
            channel_id.send_message(ctx, |m| m.embed(|embed| {
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
        });
    }
    
    match key.kind {
        PokeKeyKind::Item => get_info!(SmogonItem, dump_item),
        PokeKeyKind::Move => get_info!(SmogonMove, dump_move),
        PokeKeyKind::Ability => get_info!(SmogonAbility, dump_ability),
        PokeKeyKind::Pokemon => {
            let info: SmogonPokemon = get_info!();
            
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
            
            channel_id.send_message(ctx, |m| m.embed(move |embed| {
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
        },
    }
    
    Ok(true)
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

pub async fn update_pokemon<R: Requester>(
    db: &DbInstance,
    req: &R,
) -> Result<()> {
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
