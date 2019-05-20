use egg_mode;
use serde_derive::Deserialize;

use std::collections::{HashMap, HashSet};
use tokio_core::reactor::Core;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    ck: String,
    cs: String,
    tokens: Vec<Token>,
}

#[derive(Debug, Clone, Deserialize)]
struct Token {
    tk: String,
    ts: String,
}

fn main() -> Result<(), Box<std::error::Error>> {
    let mut core = Core::new()?;
    let handle = core.handle();

    let config = toml::from_str::<Config>(&std::fs::read_to_string("config.toml")?)?;

    let consumer = egg_mode::KeyPair::new(config.ck, config.cs);
    let tokens = config
        .tokens
        .into_iter()
        .map(|Token { tk, ts }| egg_mode::KeyPair::new(tk, ts))
        .map(|access| egg_mode::Token::Access {
            access,
            consumer: consumer.clone(),
        })
        .collect::<Vec<_>>();

    let mutes_map = tokens
        .iter()
        .map(|token| {
            core.run(egg_mode::user::mutes_ids(token, &handle).call())
                .map(|res| {
                    (
                        get_token_key(token),
                        res.response.ids.into_iter().collect::<HashSet<_>>(),
                    )
                })
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    let mutes = mutes_map
        .iter()
        .flat_map(|(_, x)| x.iter().cloned())
        .collect::<HashSet<_>>();

    tokens
        .iter()
        .flat_map(|token| {
            mutes
                .difference(mutes_map.get(&get_token_key(token)).unwrap())
                .map(|&id| core.run(egg_mode::user::mute(id, token, &handle)))
                .collect::<Vec<_>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn get_token_key(token: &egg_mode::Token) -> String {
    match token {
        egg_mode::Token::Access { access, .. } => access.key.clone().into_owned(),
        _ => unimplemented!(),
    }
}
