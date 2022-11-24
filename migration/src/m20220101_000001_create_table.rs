use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_query::Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Post::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Post::AccountUsername).string().not_null())
                    .col(ColumnDef::new(Post::Text).string())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(Media::Table)
                    .col(ColumnDef::new(Media::Id).string().not_null())
                    .col(ColumnDef::new(Media::Url).string().not_null())
                    .col(ColumnDef::new(Media::AltText).string())
                    .col(ColumnDef::new(Media::Type).string().not_null())
                    .col(ColumnDef::new(Media::Bitrate).integer().not_null())
                    .col(ColumnDef::new(Media::PostId).string().not_null())
                    .primary_key(Index::create().col(Media::Id).col(Media::Bitrate).unique())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-media-post")
                            .from_tbl(Media::Table)
                            .from_col(Media::PostId)
                            .to_tbl(Post::Table)
                            .to_col(Post::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(Likes::Table)
                    .col(
                        ColumnDef::new(Likes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Likes::PostId).string().not_null())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-likes-post")
                            .from_tbl(Likes::Table)
                            .from_col(Likes::PostId)
                            .to_tbl(Post::Table)
                            .to_col(Post::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(Bookmarks::Table)
                    .col(
                        ColumnDef::new(Bookmarks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Bookmarks::PostId).string().not_null())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-bookmarks-post")
                            .from_tbl(Bookmarks::Table)
                            .from_col(Bookmarks::PostId)
                            .to_tbl(Post::Table)
                            .to_col(Post::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                sea_query::Table::create()
                    .table(Lists::Table)
                    .col(ColumnDef::new(Lists::PostId).string().not_null())
                    .col(ColumnDef::new(Lists::ListName).string().not_null())
                    .primary_key(Index::create().col(Lists::ListName).col(Lists::PostId))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-lists-post")
                            .from_tbl(Lists::Table)
                            .from_col(Lists::PostId)
                            .to_tbl(Post::Table)
                            .to_col(Post::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(sea_query::Table::drop().table(Post::Table).to_owned())
            .await?;
        manager
            .drop_table(sea_query::Table::drop().table(Media::Table).to_owned())
            .await?;
        manager
            .drop_table(sea_query::Table::drop().table(Bookmarks::Table).to_owned())
            .await?;
        manager
            .drop_table(sea_query::Table::drop().table(Likes::Table).to_owned())
            .await?;
        manager
            .drop_table(sea_query::Table::drop().table(Lists::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Post {
    Table,
    Id,
    AccountUsername,
    Text,
}

#[derive(Iden)]
enum Media {
    Table,
    Id,
    PostId,
    Type,
    Url,
    AltText,
    Bitrate,
}

#[derive(Iden)]
enum Likes {
    Table,
    Id,
    PostId,
}

#[derive(Iden)]
enum Bookmarks {
    Table,
    Id,
    PostId,
}

#[derive(Iden)]
enum Lists {
    Table,
    PostId,
    ListName,
}
