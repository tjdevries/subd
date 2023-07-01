
#[allow(dead_code)]
pub struct Skybox {
    pub pool: sqlx::PgPool,
    pub name: String,
}

#[allow(dead_code)]
pub struct SkyboxHandler {
    pub pool: sqlx::PgPool,
}

#[allow(dead_code)]
pub struct SkyboxRemixHandler {
    pub pool: sqlx::PgPool,
}

