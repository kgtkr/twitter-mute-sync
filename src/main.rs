use egg_mode;
use serde;
use serde_derive::Deserialize;

use std::collections::HashSet;
use std::fs;
use std::io::{BufReader, Read};
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

    let config = toml::from_str::<Config>(&fs::read_to_string("config.toml")?)?;

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
    let mutes = tokens
        .iter()
        .map(|token| {
            core.run(egg_mode::user::mutes_ids(token, &handle).call())
                .map(|res| res.response.ids)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|x| x)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    tokens
        .iter()
        .flat_map(|token| {
            mutes
                .iter()
                .map(|&id| core.run(egg_mode::user::mute(id, token, &handle)))
                .collect::<Vec<_>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}
