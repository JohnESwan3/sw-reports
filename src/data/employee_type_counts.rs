use std::path::PathBuf;

use sqlx::Row;

use crate::importing::{ensure_schema, open_pool};

pub async fn load_series(db_path: PathBuf) -> Result<Vec<(String, f32)>, String> {
    let pool = open_pool(&db_path).await?;
    ensure_schema(&pool).await?;

    let rows = sqlx::query(
        r#"
        SELECT COALESCE(employee_type, 'Unknown') AS label, COUNT(*) AS count
        FROM new_hire_metrics
        GROUP BY label
        ORDER BY count DESC
        LIMIT 10
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Failed to load employee type counts: {err}"))?;

    let points = rows
        .into_iter()
        .map(|row| {
            let label: String = row.get("label");
            let count: i64 = row.get("count");
            (label, count as f32)
        })
        .collect();

    Ok(points)
}
