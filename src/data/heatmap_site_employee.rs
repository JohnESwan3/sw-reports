use std::collections::HashMap;
use std::path::PathBuf;

use sqlx::Row;

use crate::importing::{ensure_schema, open_pool};

pub async fn load_grid(
    db_path: PathBuf,
) -> Result<(Vec<String>, Vec<String>, Vec<Vec<f32>>), String> {
    let pool = open_pool(&db_path).await?;
    ensure_schema(&pool).await?;

    let rows = sqlx::query(
        r#"
        SELECT
            COALESCE(site, 'Unknown') AS site,
            COALESCE(employee_type, 'Unknown') AS employee_type,
            COUNT(*) AS count
        FROM new_hire_metrics
        GROUP BY site, employee_type
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| format!("Failed to load heatmap data: {err}"))?;

    let mut sites = Vec::new();
    let mut types = Vec::new();
    let mut map: HashMap<(String, String), f32> = HashMap::new();

    for row in rows {
        let site: String = row.get("site");
        let employee_type: String = row.get("employee_type");
        let count: i64 = row.get("count");
        if !sites.contains(&site) {
            sites.push(site.clone());
        }
        if !types.contains(&employee_type) {
            types.push(employee_type.clone());
        }
        map.insert((site, employee_type), count as f32);
    }

    sites.sort();
    types.sort();

    let mut values = vec![vec![0.0; sites.len()]; types.len()];
    for (y, employee_type) in types.iter().enumerate() {
        for (x, site) in sites.iter().enumerate() {
            if let Some(value) = map.get(&(site.clone(), employee_type.clone())) {
                values[y][x] = *value;
            }
        }
    }

    Ok((sites, types, values))
}
