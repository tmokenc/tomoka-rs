use crate::commands::prelude::*;
use crate::constants::*;
use crate::traits::Embedable;
use crate::traits::Paginator;
use crate::Result;
use crate::types::Ref;
use core::time::Duration;
use db::DbInstance;
use futures::future;
use magic::import_all;
use magic::traits::MagicIter;
use pokemon_core::types::Type;
use requester::smogon::{
    Ability as SmogonAbility, Generation, Item as SmogonItem, Move as SmogonMove,
    Pokemon as SmogonPokemon, SmogonApi, SmogonCommon, SmogonPokemon as SmogonPokemonDump,
};
use requester::Requester;
use serde::{Deserialize, Serialize};
use serenity::builder::CreateEmbed;
use serenity::framework::standard::macros::group;
use serenity::model::channel::ReactionType;
use std::fmt::Write as _;

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
#[prefixes("pokemon", "pkm")]
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
    })
    .await??;

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
            .open(SMOGON_POKEMON)?,
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
                Ref(info).append_to(embed).description(desc.description);

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

    let not_in_swsh =
        matches!(key.gen, Generation::SwordShield) && &info.is_nonestandard == "NatDex";

    let abilities = info
        .abilities
        .iter()
        .map(|abi| PokeKey::new(abi, key.gen, PokeKeyKind::Ability))
        .map(|ref key| match db.get::<PokeKey, SmogonAbility>(key) {
            Ok(Some(ability)) => format!("- **{}** ({})", ability.name, ability.description),
            _ => String::from(&key.name),
        })
        .join("\n");

    let base_title = format!("Base Stats  {}", info.base_stats_total());
    let base_stats = format!(
        "HP: {}\nAttack: {}\nDefense: {}\nSpecial Attack: {}\nSpecial Defense: {}\nSpeed: {}",
        info.hp, info.atk, info.def, info.spa, info.spd, info.spe,
    );

    let sprite = format!(
        "https://www.smogon.com/dex/media/sprites/xy/{}.gif",
        &key.name
    );
    let mut title = match info.dex_number() {
        Some(dex) => format!("#{} {}", dex, info.name),
        None => info.name.to_owned(),
    };

    let types: Vec<_> = info
        .types
        .iter()
        .filter_map(|v| v.parse::<Type>().ok())
        .collect();

    title.push_str("    ");
    let types = match crate::read_config().await.emoji.pokemon.as_ref() {
        None => {
            title.push_str(&info.types.join("   "));

            let mut data = String::new();

            for (k, v) in type_effective(types.as_slice()) {
                writeln!(&mut data, "**{}**: {}", k, v.iter().join(", "))?;
            }

            data
        }

        Some(emoji) => {
            let format_type =
                |t: &Type| format!("{}  {}", emoji.get(&t.to_string()).unwrap_or_default(), t);

            title.push_str(&types.iter().map(format_type).join("   "));
            let mut data = String::new();

            for (k, v) in type_effective(types.as_slice()) {
                let s = v.iter().map(format_type).join(", ");
                writeln!(&mut data, "**{}**: {}", k, s)?;
            }

            data
        }
    };

    let gen = if not_in_swsh {
        Generation::SunMoon
    } else {
        key.gen
    };

    let message = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|embed| {
                embed.title(title);
                embed.thumbnail(sprite);
                embed.field("Type Advantages", types, true);
                embed.field(base_title, base_stats, false);
                embed.field("Abilities", abilities, false);
                embed.footer(|f| f.text(format!("Generation: {}", gen)));

                if not_in_swsh {
                    embed.description("This pokemon isn't available in sword/shield yet...");
                }

                if let Some(oob) = info.oob.as_ref() {
                    if !oob.evos.is_empty() {
                        embed.field("Next Evolution", oob.evos.join("\n"), true);
                    }

                    if !oob.alts.is_empty() {
                        embed.field("Altenative Pokemon", oob.alts.join("\n"), true);
                    }
                }

                embed
            })
        })
        .await?;

    let reaction = ReactionType::Unicode(String::from("⚔"));
    let duration = Duration::from_secs(30);
    let reacted = wait_for_reaction(ctx, &message, reaction, duration).await?;

    if let Some(user) = reacted {
        use crate::traits::paginator::PaginatorOption;

        let opt = PaginatorOption {
            channel_id: msg.channel_id,
            user,
        };

        MovesPaginator::new(ctx, &info.name, gen, db)
            .await?
            .pagination(ctx, opt)
            .await?;
    }

    Ok(())
}

fn type_effective(types: &[Type]) -> Vec<(&'static str, Vec<Type>)> {
    let mut effective = Vec::new();
    let mut ex_effective = Vec::new();
    let mut not_effective = Vec::new();
    let mut not_very_effective = Vec::new();
    let mut immune = Vec::new();

    for t in Type::iter() {
        let modifier: f32 = types
            .iter()
            .map(|v| t.effective(*v))
            .map(f32::from)
            .product();

        macro_rules! check_eff {
            ($x:expr, $y:ident) => {
                if modifier == $x {
                    $y.push(t)
                }
            };
        }

        check_eff!(0.25, not_very_effective);
        check_eff!(0.50, not_effective);
        check_eff!(2.00, effective);
        check_eff!(4.00, ex_effective);
        check_eff!(0.00, immune);
    }

    let mut result = Vec::new();

    macro_rules! push_eff {
        ($x:expr, $y:ident) => {
            if !$y.is_empty() {
                result.push(($x, $y));
            }
        };
    }

    push_eff!("Strongly resists", not_very_effective);
    push_eff!("Resists", not_effective);
    push_eff!("Weak to", effective);
    push_eff!("Very weak to", ex_effective);
    push_eff!("Immune to", immune);

    result
}

pub fn parse_args(args: &str) -> Option<(String, Generation)> {
    if args.is_empty() {
        return None;
    }

    let (data, gen): (Vec<&str>, Vec<&str>) =
        args.split_whitespace().partition(|v| !v.starts_with("--"));

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
    let requests = POKEMON_VERSIONS.iter().map(|v| req.dump_basics(*v));

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
                };
            };

            insert!(pokemon, PokeKeyKind::Pokemon);
            insert!(abilities, PokeKeyKind::Ability);
            insert!(moves, PokeKeyKind::Move);
            insert!(items, PokeKeyKind::Item);

            pokemon.batch(batch)?;
        }

        Ok(())
    })
    .await?
}

impl Embedable for Ref<SmogonMove> {
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

impl Embedable for Ref<SmogonAbility> {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        embed.title(&self.name);
        embed
    }
}
impl Embedable for Ref<SmogonItem> {
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
        let list: Vec<_> = learnset
            .into_iter()
            .map(|v| (v, MoveFinding::NotYet))
            .collect();
        let len = list.len();

        Ok(Self {
            pokemon: pokemon.to_owned(),
            gen,
            list: std::sync::Mutex::new(list),
            len,
            db,
            icons,
        })
    }
}

impl Paginator for MovesPaginator {
    fn append_page(&mut self, page: core::num::NonZeroUsize, embed: &mut CreateEmbed) {
        let page = page.get();
        let index = (page - 1) * POKEMON_MOVE_PER_PAGE;

        let description = self.list.lock().unwrap()[index..]
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
                            None => format!("**{}** ({} {})", m.name, m.category, m.kind,),
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
        embed.footer(|f| {
            f.text(format!(
                "Page {} / {} | Generation {}",
                page,
                self.total_pages().unwrap(),
                self.gen
            ))
        });
    }

    fn total_pages(&self) -> Option<usize> {
        if self.len == 0 {
            Some(0)
        } else {
            Some(((self.len - 1) / POKEMON_MOVE_PER_PAGE) + 1)
        }
    }
}
