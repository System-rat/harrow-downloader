use std::{path::Path};

use anyhow::Context;
use entity::prelude::*;
use migration::{Migrator, MigratorTrait};
use reqwest::Url;
use sea_orm::{ConnectOptions, Database, EntityTrait, QueryFilter, ColumnTrait};
use tokio::{fs::File, io::copy};
use tracing::{Level, debug};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let sub = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    
    tracing::subscriber::set_global_default(sub)?;
    
    let harrow_dir = Path::new(&dirs::data_dir().context("Could not get data dir")?).join("harrow-downloader");

    let db = Database::connect(
        ConnectOptions::new(
            "sqlite://".to_string() + harrow_dir.join("db.sqlite?mode=rwc").to_str().context("Invalid data path")?
        )
        .to_owned(),
    )
    .await?;
    if let Err(err) = Migrator::up(&db, None).await {
        println!("Error occurred during database initialization: {}", err);
    }

    for post in Post::find().all(&db).await? {
        let post_media = Media::find().filter(entity::media::Column::PostId.eq(post.id.clone())).all(&db).await?;
        debug!("Downloading post: {}", post.id);
        if post_media.len() > 0 {
            if post_media.iter().any(|m| m.r#type.clone().unwrap_or("photo".into()) == "photo") {
                for m in post_media {
                    let url = Url::parse(&m.url)?;
                    let filename = Path::new(url.path().clone()).file_name().context("invalid filename")?;
                    
                    debug!("Downloading file: {}", url);

                    let res_bytes = reqwest::get(&m.url).await?.bytes().await?.to_vec();
                    let mut f = File::create(harrow_dir.join("data").join(filename)).await?;
                    copy(&mut res_bytes.as_slice(), &mut f).await?;
                }
            } else {
                let mut max = &post_media[0];
                let mut max_brt = 0;
                for m in post_media.iter() {
                    if let Some(brt) = m.bitrate {
                        if brt > max_brt {
                            max = &m;
                            max_brt = brt;
                        }
                    }
                }

                let url = Url::parse(&max.url)?;
                let filename = Path::new(url.path().clone()).file_name().context("Invalid filename")?;

                debug!("Downloading file: {}", url);

                let res_bytes = reqwest::get(&max.url).await?.bytes().await?.to_vec();
                let mut f = File::create(harrow_dir.join("data").join(filename)).await?;
                copy(&mut res_bytes.as_slice(), &mut f).await?;
            }
        }
    }

    Ok(())
}
