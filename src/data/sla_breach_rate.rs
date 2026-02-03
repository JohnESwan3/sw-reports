use std::path::PathBuf;

use sqlx::Row;

use crate::importing::{ensure_schema, open_pool};

pub async fn load_rate(db_path: PathBuf) -> Result<(f32, f32), String> {
    let pool = open_pool(&db_path).await?;
    ensure_schema(&pool).await?;

    let row = sqlx::query(
        r#"
        SELECT
            SUM(CASE WHEN sla_breaches IS NOT NULL AND sla_breaches != '' THEN 1 ELSE 0 END) AS breaches,
            COUNT(*) AS total
        FROM new_hire_metrics
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|err| format!("Failed to load breach rate: {err}"))?;

    let breaches: f32 = row
        .try_get::<i64, _>("breaches")
        .unwrap_or(0) as f32;
    let total: f32 = row.try_get::<i64, _>("total").unwrap_or(0) as f32;

    Ok((breaches, total.max(1.0)))
}
