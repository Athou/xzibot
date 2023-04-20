use anyhow::Error;
use rand::Rng;
use sql_builder::SqlBuilder;
use sqlx::{mysql::MySqlQueryResult, MySqlPool, Row};

#[derive(sqlx::FromRow)]
pub struct Connerie {
    pub id: i64,
    pub value: String,
    pub author: Option<String>,
}

impl Connerie {
    pub async fn count(pool: &MySqlPool) -> Result<i64, Error> {
        let count: i64 = sqlx::query("SELECT count(*) from Connerie")
            .fetch_one(pool)
            .await?
            .get(0);
        Ok(count)
    }

    fn build_search_sql(tokens: &[&str], with_spaces: bool) -> Result<String, Error> {
        let mut sql = SqlBuilder::select_from("Connerie");
        sql.field("*");
        for token in tokens {
            let like_pattern = if with_spaces {
                format!("% {} %", token.to_lowercase())
            } else {
                format!("%{}%", token.to_lowercase())
            };
            sql.and_where_like("LOWER(value)", like_pattern);
        }
        sql.sql()
    }

    pub async fn search(pool: &MySqlPool, tokens: &[&str]) -> Result<Option<String>, Error> {
        let mut conneries =
            sqlx::query_as::<_, Connerie>(&Connerie::build_search_sql(tokens, true)?)
                .fetch_all(pool)
                .await?;

        if conneries.is_empty() {
            conneries = sqlx::query_as::<_, Connerie>(&Connerie::build_search_sql(tokens, false)?)
                .fetch_all(pool)
                .await?;
        }

        if conneries.is_empty() {
            return Ok(None);
        }

        let i = rand::thread_rng().gen_range(0..conneries.len());
        let connerie = conneries.into_iter().nth(i).map(|c| c.value);
        Ok(connerie)
    }

    pub async fn random(pool: &MySqlPool) -> Result<Option<String>, Error> {
        let count = Connerie::count(pool).await?;
        if count <= 0 {
            Ok(None)
        } else {
            let offset = rand::thread_rng().gen_range(0..count);
            let connerie = sqlx::query_as::<_, Connerie>("SELECT * FROM Connerie LIMIT 1 OFFSET ?")
                .bind(offset)
                .fetch_one(pool)
                .await?;
            Ok(Some(connerie.value))
        }
    }

    pub async fn insert(
        pool: &MySqlPool,
        author: &str,
        content: &str,
    ) -> Result<MySqlQueryResult, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO Connerie (`author`, `value`)
            VALUES(?, ?)"#,
        )
        .bind(author)
        .bind(content)
        .execute(pool)
        .await?;
        Ok(result)
    }
}
