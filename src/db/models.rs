#[derive(sqlx::FromRow)]
pub struct Connerie {
    pub id: i64,
    pub value: String,
    pub author: Option<String>,
}
