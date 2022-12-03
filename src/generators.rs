use sea_orm::Database;

pub trait Generator {
    fn generate_entires(&self, db: &mut Database);
}

pub struct ArtistsGenerator;

pub struct LikesGenerator;

pub struct BookmarksGenerator;
