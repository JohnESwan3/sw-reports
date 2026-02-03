use std::path::PathBuf;

use sqlx::Row;

use crate::importing::{ensure_schema, open_pool};

pub async fn load_metrics(db_path: PathBuf) -> Result<Vec<(String, f32)>, String> {
    let pool = open_pool(&db_path).await?;
    ensure_schema(&pool).await?;

    let row = sqlx::query(
        r#"
        SELECT
            AVG(it_lead_time_elapsed) AS avg_elapsed,
            AVG(it_lead_time_business) AS avg_business,
            SUM(CASE WHEN sla_breaches IS NOT NULL AND sla_breaches != '' THEN 1 ELSE 0 END) AS breaches,
            COUNT(*) AS total
        FROM new_hire_metrics
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|err| format!("Failed to load radar metrics: {err}"))?;

    let avg_elapsed: f32 = row
        .try_get::<f64, _>("avg_elapsed")
        .unwrap_or(0.0) as f32;
    let avg_business: f32 = row
        .try_get::<f64, _>("avg_business")
        .unwrap_or(0.0) as f32;
    let breaches: f32 = row
        .try_get::<i64, _>("breaches")
        .unwrap_or(0) as f32;
    let total: f32 = row.try_get::<i64, _>("total").unwrap_or(0) as f32;

    Ok(vec![
        ("Avg Elapsed".to_string(), avg_elapsed),
        ("Avg Business".to_string(), avg_business),
        ("Breach Count".to_string(), breaches),
        ("Total Records".to_string(), total),
    ])
}
