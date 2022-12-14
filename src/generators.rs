use std::path::Path;

use anyhow::{Context, Result};
use async_trait::async_trait;
use entity::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio::fs::{create_dir_all, symlink};
use tracing::{debug, error};

use crate::utils::filename_from_url;

#[async_trait]
pub trait Generator {
    async fn generate_entires(
        &self,
        db: &DatabaseConnection,
        target_path: &Path,
        data_dir: &Path,
    ) -> Result<()>;
}

pub struct ArtistsGenerator;

pub struct LikesGenerator;

pub struct BookmarksGenerator;

#[async_trait]
impl Generator for ArtistsGenerator {
    async fn generate_entires(
        &self,
        db: &DatabaseConnection,
        target_path: &Path,
        data_dir: &Path,
    ) -> Result<()> {
        let posts = Post::find().find_with_related(Media).all(db).await?;

        for (post, medias) in posts {
            let path = &target_path.join(&post.account_username);
            if create_dir_all(path).await.is_err() {
                error!(
                    "Coult not create dir for account: {}. Possible encoding issue?",
                    &post.account_username
                );
                continue;
            }

            if medias.is_empty() {
                debug!("Skipping non-media post: {}", post.id);
                continue;
            }

            if let Err(e) = symlink_files_and_metafiles(data_dir, path, &medias).await {
                error!("Could not symlink post: {} due to error: {}", post.id, e);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Generator for LikesGenerator {
    async fn generate_entires(
        &self,
        db: &DatabaseConnection,
        target_path: &Path,
        data_dir: &Path,
    ) -> Result<()> {
        let likes = Likes::find().all(db).await?;
        let posts = Post::find().find_with_related(Media).all(db).await?;

        for like in likes {
            let (post, medias) = posts
                .iter()
                .find(|p| p.0.id == like.post_id)
                .context("Post does not exist from likes")?;

            if medias.is_empty() {
                debug!("Skipping non-media post: {}", post.id);
                continue;
            }

            if let Err(e) = symlink_files_and_metafiles(data_dir, target_path, medias).await {
                error!("Could not symlink post: {} due to error: {}", post.id, e);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Generator for BookmarksGenerator {
    async fn generate_entires(
        &self,
        db: &DatabaseConnection,
        target_path: &Path,
        data_dir: &Path,
    ) -> Result<()> {
        let bookmarks = Bookmarks::find().all(db).await?;
        let posts = Post::find().find_with_related(Media).all(db).await?;

        for bookmark in bookmarks {
            let (post, medias) = posts
                .iter()
                .find(|p| p.0.id == bookmark.post_id)
                .context("Post does not exist from bookmarks")?;

            if medias.is_empty() {
                debug!("Skipping non-media post: {}", post.id);
                continue;
            }

            if let Err(e) = symlink_files_and_metafiles(data_dir, target_path, medias).await {
                error!("Could not symlink post: {} due to error: {}", post.id, e);
            }
        }

        Ok(())
    }
}

async fn symlink_files_and_metafiles(
    src_dir: &Path,
    dest_dir: &Path,
    medias: &Vec<entity::media::Model>,
) -> Result<()> {
    let mut filenames = Vec::new();
    for media in medias {
        let filename = filename_from_url(&media.url).context("Could not get filename")?;
        symlink(src_dir.join(&filename), dest_dir.join(&filename))
            .await
            .context("Could not symlink media")?;

        filenames.push(filename);
    }

    let meta_filename = filenames.join("__") + ".txt";
    symlink(src_dir.join(&meta_filename), dest_dir.join(&meta_filename))
        .await
        .context(format!("Could not symlink metafile: {}", meta_filename))
}
