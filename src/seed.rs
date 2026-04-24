use pgvector::Vector;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;

#[derive(Deserialize)]
struct Ref {
    vector: [f32; 14],
    label: String,
}

const SCHEMA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/schema.sql"));
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://rinha2026:rinha2026@localhost:55432/rinha2026".into());
    let references_path =
        std::env::var("REFERENCES_PATH").unwrap_or_else(|_| "resources/references.json".into());

    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await?;

    sqlx::query("SELECT pg_advisory_lock(7283746192837465)")
        .execute(&pool)
        .await?;

    for stmt in SCHEMA.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        sqlx::query(stmt).execute(&pool).await?;
    }

    let existing: i64 = sqlx::query_scalar("SELECT count(*) FROM vetores_rinha")
        .fetch_one(&pool)
        .await?;

    if existing > 0 {
        ensure_reference_index(&pool).await?;
        return Ok(());
    }

    let bytes = std::fs::read(&references_path)?;
    let refs: Vec<Ref> = serde_json::from_slice(&bytes)?;

    let (vectors, labels): (Vec<Vector>, Vec<String>) = refs
        .into_iter()
        .map(|r| (Vector::from(r.vector.to_vec()), r.label))
        .unzip();

    sqlx::query(
        "INSERT INTO vetores_rinha (vector, label) \
         SELECT v, l FROM unnest($1::vector[], $2::text[]) AS t(v, l)",
    )
    .bind(&vectors)
    .bind(&labels)
    .execute(&pool)
    .await?;

    ensure_reference_index(&pool).await?;

    Ok(())
}

async fn ensure_reference_index(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DROP INDEX IF EXISTS vetores_rinha_vector_hnsw_l2_m32_ef128_idx")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS vetores_rinha_vector_hnsw_l2_idx \
         ON vetores_rinha USING hnsw (vector vector_l2_ops)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
