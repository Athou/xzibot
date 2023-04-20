use anyhow::Error;
use rand::Rng;
use sql_builder::SqlBuilder;
use sqlx::MySqlPool;
use sqlx::Row;

#[derive(sqlx::FromRow)]
pub struct Quote {
    pub id: i64,
    pub quote: String,
    pub number: i64,
}

impl Quote {
    pub async fn find_by_number(pool: &MySqlPool, number: i64) -> Result<Option<Quote>, Error> {
        let quote = sqlx::query_as::<_, Quote>("SELECT * FROM Quote where number = ?")
            .bind(number)
            .fetch_optional(pool)
            .await?;
        Ok(quote)
    }

    pub async fn count(pool: &MySqlPool) -> Result<i64, Error> {
        let count: i64 = sqlx::query("SELECT count(*) from Quote")
            .fetch_one(pool)
            .await?
            .get(0);
        Ok(count)
    }

    fn build_search_sql(tokens: &[&str]) -> Result<String, Error> {
        let mut sql = SqlBuilder::select_from("Quote");
        sql.field("*");
        for token in tokens {
            let like_pattern = format!("%{}%", token.to_lowercase());
            sql.and_where_like("LOWER(quote)", like_pattern);
        }
        sql.sql()
    }

    pub async fn search(pool: &MySqlPool, tokens: &[&str]) -> Result<Vec<Quote>, Error> {
        let quotes = sqlx::query_as::<_, Quote>(&Quote::build_search_sql(tokens)?)
            .fetch_all(pool)
            .await?;

        Ok(quotes)
    }

    pub async fn random(pool: &MySqlPool) -> Result<Option<Quote>, Error> {
        let count = Quote::count(pool).await?;
        if count <= 0 {
            Ok(None)
        } else {
            let offset = rand::thread_rng().gen_range(0..count);
            let quote = sqlx::query_as::<_, Quote>("SELECT * FROM Quote LIMIT 1 OFFSET ?")
                .bind(offset)
                .fetch_one(pool)
                .await?;
            Ok(Some(quote))
        }
    }

    pub async fn save(pool: &MySqlPool, quote: &str) -> Result<i64, Error> {
        let number = Quote::count(pool).await? + 1;

        sqlx::query(
            r#"
            INSERT INTO Quote (`quote`, `number`)
            VALUES(?, ?)"#,
        )
        .bind(quote)
        .bind(number)
        .execute(pool)
        .await?;

        Ok(number)
    }
}
