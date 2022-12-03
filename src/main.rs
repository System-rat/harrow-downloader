use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

use anyhow::Context;
use clap::Parser;
use entity::prelude::*;
use migration::{Migrator, MigratorTrait};
use reqwest::Url;
use sea_orm::{ColumnTrait, ConnectOptions, Database, EntityTrait, QueryFilter};
use tokio::{fs::File, io::copy, process::Command};
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;

mod cli;
mod generators;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = cli::CliArgs::parse();

    let sub = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(sub)?;

    debug!("Parser: {:?}", args);

    let harrow_dir = if let Some(dir) = args.data_directory {
        dir
    } else {
        Path::new(&dirs::data_dir().context("Could not get data dir")?).join("harrow-downloader")
    };

    // Reset the DB
    if !args.skip_db_regen && harrow_dir.join("db.sqlite").exists() {
        tokio::fs::remove_file(harrow_dir.join("db.sqlite")).await?;
    }

    if harrow_dir.join("data").exists() {
        if args.clean_data_directory {
            tokio::fs::remove_dir_all(harrow_dir.join("data")).await?;
            tokio::fs::create_dir_all(harrow_dir.join("data")).await?;
        }
    } else {
        tokio::fs::create_dir_all(harrow_dir.join("data")).await?;
    }

    let db = Database::connect(
        ConnectOptions::new(
            "sqlite://".to_string()
                + harrow_dir
                    .join("db.sqlite?mode=rwc")
                    .to_str()
                    .context("Invalid data path")?,
        )
        .to_owned(),
    )
    .await?;
    if let Err(err) = Migrator::up(&db, None).await {
        println!("Error occurred during database initialization: {}", err);
    }

    if !args.skip_db_regen {
        let db_gen_cmd = Command::new("bash")
            .args(["-c", "cd db-gen; node lib/index.js"])
            .spawn();
        if !db_gen_cmd
            .context("SPAWNING")?
            .wait()
            .await
            .context("WAITING")?
            .success()
        {
            tracing::error!("Could not regenerate DB");
            return Err(anyhow::Error::msg("DB Gen error"));
        }
    }

    for post in Post::find().all(&db).await? {
        let post_media = Media::find()
            .filter(entity::media::Column::PostId.eq(post.id.clone()))
            .all(&db)
            .await?;
        debug!("Downloading post: {}", post.id);
        if !post_media.is_empty() {
            if post_media
                .iter()
                .any(|m| m.r#type.clone().unwrap_or_else(|| "photo".into()) == "photo")
            {
                let mut filenames = vec![];
                for m in post_media {
                    let url = Url::parse(&m.url)?;
                    let filename = Path::new(url.path())
                        .file_name()
                        .context("invalid filename")?;

                    download_file(&harrow_dir.join("data"), filename, &url).await?;
                    filenames.push(filename.to_owned());
                }
                write_metadata(
                    &harrow_dir.join("data"),
                    &post.account_username,
                    if let Some(ref txt) = post.text {
                        txt.as_str()
                    } else {
                        ""
                    },
                    &post.id,
                    filenames.as_slice(),
                )
                .await?;
            } else {
                let mut max = &post_media[0];
                let mut max_brt = 0;
                for m in post_media.iter() {
                    if let Some(brt) = m.bitrate {
                        if brt > max_brt {
                            max = m;
                            max_brt = brt;
                        }
                    }
                }

                let url = Url::parse(&max.url)?;
                let filename = Path::new(url.path())
                    .file_name()
                    .context("Invalid filename")?;

                download_file(&harrow_dir.join("data"), filename, &url).await?;
                write_metadata(
                    &harrow_dir.join("data"),
                    &post.account_username,
                    if let Some(ref txt) = post.text {
                        txt.as_str()
                    } else {
                        ""
                    },
                    &post.id,
                    &[filename.to_owned()],
                )
                .await?;
            }
        }
    }

    Ok(())
}

async fn download_file(
    download_dir: &Path,
    filename: &std::ffi::OsStr,
    url: &Url,
) -> anyhow::Result<()> {
    if download_dir.join(filename).exists() {
        debug!("File {:?} already exists, skipping", filename);
        return Ok(());
    }

    let res_bytes = reqwest::get(url.clone()).await?.bytes().await?.to_vec();
    let mut f = File::create(download_dir.join(filename)).await?;
    copy(&mut res_bytes.as_slice(), &mut f).await?;

    Ok(())
}

async fn write_metadata(
    download_dir: &Path,
    artist: &str,
    text: &str,
    post_id: &str,
    filenames: &[OsString],
) -> anyhow::Result<()> {
    let mut filename = filenames.join(OsStr::new("__"));
    filename.push(".txt");

    debug!(
        "Writing metadata for files: {:?}",
        filenames.join(OsStr::new(", "))
    );
    tokio::fs::write(
        download_dir.join(filename),
        format!(
            "post id: {}\nartist: {}\ntext:\n{}\n\n\nfiles: {}",
            post_id,
            artist,
            text,
            filenames.join(OsStr::new(", ")).to_string_lossy().as_ref()
        ),
    )
    .await?;

    Ok(())
}
