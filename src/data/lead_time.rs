use std::path::PathBuf;

use sqlx::Row;

use crate::importing::{ensure_schema, open_pool};

pub async fn load_series(db_path: PathBuf) -> Result<Vec<(f32, f32)>, String> {
    let pool = open_pool(&db_path).await?;
    ensure_schema(&pool).await?;

    let rows = sqlx::query(
        r#"
        SELECT number, it_lead_time_elapsed
        FROM new_hire_metrics
        WHERE it_lead_time_elapsed IS NOT NULL
        ORDER BY number
        LIMIT 200
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Failed to load report data: {err}"))?;

    let points = rows
        .into_iter()
        .filter_map(|row| {
            let number: i64 = row.get("number");
            let lead_time: i64 = row.get("it_lead_time_elapsed");
            Some((number as f32, lead_time as f32))
        })
        .collect();

    Ok(points)
}
