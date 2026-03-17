use reqwest::Client as HttpClient;
use serenity::{all::Context, prelude::TypeMapKey};

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

pub async fn get_http_client(ctx: &Context) -> HttpClient {
    let data = ctx.data.read().await;
    data.get::<HttpKey>()
        .cloned()
        .expect("Guaranteed to exist in the typemap.")
}
