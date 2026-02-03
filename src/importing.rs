use std::path::{Path, PathBuf};

use csv_async::AsyncReaderBuilder;
use futures::StreamExt;
use serde::Deserialize;
use sqlx::{sqlite::SqliteConnectOptions, sqlite::SqlitePoolOptions, Row, SqlitePool};
use tokio::fs;
use tokio_util::compat::TokioAsyncReadCompatExt;

#[derive(Debug, Clone)]
pub struct NewHireRecord {
    pub number: i64,
    pub state: Option<String>,
    pub title: Option<String>,
    pub assignee_name: Option<String>,
    pub requester: Option<String>,
    pub created_at: Option<String>,
    pub site: Option<String>,
    pub division: Option<String>,
    pub employee_type: Option<String>,
    pub start_date: Option<String>,
    pub success_factors_date_entered: Option<String>,
    pub day_1_or_day_3: Option<String>,
    pub to_first_response_business: Option<String>,
    pub to_resolution_business: Option<String>,
    pub to_resolution_elapsed: Option<String>,
    pub sla_breaches: Option<String>,
    pub resolved_at: Option<String>,
    pub it_lead_time_elapsed: Option<i64>,
    pub it_lead_time_business: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct NewHireCsvRow {
    #[serde(rename = "Number")]
    number: Option<i64>,
    #[serde(rename = "State")]
    state: Option<String>,
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "Assignee Name")]
    assignee_name: Option<String>,
    #[serde(rename = "Requester")]
    requester: Option<String>,
    #[serde(rename = "Created At (Timestamp)")]
    created_at: Option<String>,
    #[serde(rename = "Site")]
    site: Option<String>,
    #[serde(rename = "Division")]
    division: Option<String>,
    #[serde(rename = "Employee Type")]
    employee_type: Option<String>,
    #[serde(rename = "Start Date")]
    start_date: Option<String>,
    #[serde(rename = "Success Factors Date entered")]
    success_factors_date_entered: Option<String>,
    #[serde(rename = "Day 1 or Day 3")]
    day_1_or_day_3: Option<String>,
    #[serde(rename = "To First Response (Business)")]
    to_first_response_business: Option<String>,
    #[serde(rename = "To Resolution (Business)")]
    to_resolution_business: Option<String>,
    #[serde(rename = "To Resolution (Elapsed)")]
    to_resolution_elapsed: Option<String>,
    #[serde(rename = "SLA Breaches")]
    sla_breaches: Option<String>,
    #[serde(rename = "Resolved At")]
    resolved_at: Option<String>,
    #[serde(rename = "IT Lead Time (Elapsed)")]
    it_lead_time_elapsed: Option<i64>,
    #[serde(rename = "IT Lead Time (Business)")]
    it_lead_time_business: Option<i64>,
}

impl From<NewHireCsvRow> for NewHireRecord {
    fn from(row: NewHireCsvRow) -> Self {
        Self {
            number: row.number.unwrap_or_default(),
            state: row.state,
            title: row.title,
            assignee_name: row.assignee_name,
            requester: row.requester,
            created_at: row.created_at,
            site: row.site,
            division: row.division,
            employee_type: row.employee_type,
            start_date: row.start_date,
            success_factors_date_entered: row.success_factors_date_entered,
            day_1_or_day_3: row.day_1_or_day_3,
            to_first_response_business: row.to_first_response_business,
            to_resolution_business: row.to_resolution_business,
            to_resolution_elapsed: row.to_resolution_elapsed,
            sla_breaches: row.sla_breaches,
            resolved_at: row.resolved_at,
            it_lead_time_elapsed: row.it_lead_time_elapsed,
            it_lead_time_business: row.it_lead_time_business,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingDuplicate {
    pub number: i64,
    pub title: Option<String>,
    pub created_at: Option<String>,
    pub changes: Vec<String>,
}

impl PendingDuplicate {
    pub fn from_record(record: &NewHireRecord, changes: Vec<String>) -> Self {
        Self {
            number: record.number,
            title: record.title.clone(),
            created_at: record.created_at.clone(),
            changes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportStatus {
    Idle,
    Loading,
    Importing,
    AwaitingDecision,
    Done,
    Error,
}

#[derive(Debug, Clone)]
pub struct ImportState {
    pub status: ImportStatus,
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub total: usize,
    pub pending_duplicates: Vec<PendingDuplicate>,
    pub message: Option<String>,
}

impl ImportState {
    pub fn new() -> Self {
        Self {
            status: ImportStatus::Idle,
            inserted: 0,
            updated: 0,
            skipped: 0,
            total: 0,
            pending_duplicates: Vec::new(),
            message: None,
        }
    }

    pub fn start(&mut self, total: usize) {
        self.status = ImportStatus::Importing;
        self.inserted = 0;
        self.updated = 0;
        self.skipped = 0;
        self.total = total;
        self.pending_duplicates.clear();
        self.message = None;
    }

    pub fn set_error(&mut self, message: String) {
        self.status = ImportStatus::Error;
        self.message = Some(message);
    }

    pub fn set_message(&mut self, message: String) {
        self.message = Some(message);
    }
}

#[derive(Debug, Clone)]
pub enum ImportStep {
    Inserted,
    Duplicate(DuplicateEntry),
    Updated,
    SkippedUnchanged,
    SkippedDecision,
}

#[derive(Debug, Clone)]
pub struct DuplicateEntry {
    pub record: NewHireRecord,
    pub summary: PendingDuplicate,
}

pub async fn read_new_hire_csv(path: PathBuf) -> Result<Vec<NewHireRecord>, String> {
    let file = fs::File::open(&path)
        .await
        .map_err(|err| format!("Failed to open CSV: {err}"))?;

    let mut reader = AsyncReaderBuilder::new()
        .trim(csv_async::Trim::All)
        .create_deserializer(file.compat());

    let mut rows = reader.deserialize::<NewHireCsvRow>();
    let mut records = Vec::new();

    while let Some(result) = rows.next().await {
        let row = result.map_err(|err| format!("CSV parse error: {err}"))?;

        if let Some(number) = row.number {
            let mut record: NewHireRecord = row.into();
            record.number = number;
            records.push(record);
        }
    }

    Ok(records)
}

pub async fn process_record(db_path: PathBuf, record: NewHireRecord) -> Result<ImportStep, String> {
    let pool = open_pool(&db_path).await?;
    ensure_schema(&pool).await?;

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM new_hire_metrics WHERE number = ? LIMIT 1",
    )
    .bind(record.number)
    .fetch_optional(&pool)
    .await
    .map_err(|err| format!("Failed to check duplicates: {err}"))?;

    if exists.is_some() {
        let existing = fetch_existing_record(&pool, record.number).await?;
        let changes = diff_records(&existing, &record);

        if changes.is_empty() {
            return Ok(ImportStep::SkippedUnchanged);
        }

        return Ok(ImportStep::Duplicate(DuplicateEntry {
            summary: PendingDuplicate::from_record(&record, changes),
            record,
        }));
    }

    insert_record(&pool, &record).await?;
    Ok(ImportStep::Inserted)
}

pub async fn apply_duplicate_decision(
    db_path: PathBuf,
    record: NewHireRecord,
    overwrite: bool,
) -> Result<ImportStep, String> {
    let pool = open_pool(&db_path).await?;
    ensure_schema(&pool).await?;

    if overwrite {
        update_record(&pool, &record).await?;
        Ok(ImportStep::Updated)
    } else {
        Ok(ImportStep::SkippedDecision)
    }
}

pub async fn open_pool(db_path: &Path) -> Result<SqlitePool, String> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| format!("Failed to create data directory: {err}"))?;
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|err| format!("Failed to connect to database: {err}"))
}

pub async fn ensure_schema(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS new_hire_metrics (
            number INTEGER PRIMARY KEY,
            state TEXT,
            title TEXT,
            assignee_name TEXT,
            requester TEXT,
            created_at TEXT,
            site TEXT,
            division TEXT,
            employee_type TEXT,
            start_date TEXT,
            success_factors_date_entered TEXT,
            day_1_or_day_3 TEXT,
            to_first_response_business TEXT,
            to_resolution_business TEXT,
            to_resolution_elapsed TEXT,
            sla_breaches TEXT,
            resolved_at TEXT,
            it_lead_time_elapsed INTEGER,
            it_lead_time_business INTEGER
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|err| format!("Failed to create schema: {err}"))?;

    Ok(())
}

async fn insert_record(pool: &SqlitePool, record: &NewHireRecord) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO new_hire_metrics (
            number,
            state,
            title,
            assignee_name,
            requester,
            created_at,
            site,
            division,
            employee_type,
            start_date,
            success_factors_date_entered,
            day_1_or_day_3,
            to_first_response_business,
            to_resolution_business,
            to_resolution_elapsed,
            sla_breaches,
            resolved_at,
            it_lead_time_elapsed,
            it_lead_time_business
        )
        VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        )
        "#,
    )
    .bind(record.number)
    .bind(&record.state)
    .bind(&record.title)
    .bind(&record.assignee_name)
    .bind(&record.requester)
    .bind(&record.created_at)
    .bind(&record.site)
    .bind(&record.division)
    .bind(&record.employee_type)
    .bind(&record.start_date)
    .bind(&record.success_factors_date_entered)
    .bind(&record.day_1_or_day_3)
    .bind(&record.to_first_response_business)
    .bind(&record.to_resolution_business)
    .bind(&record.to_resolution_elapsed)
    .bind(&record.sla_breaches)
    .bind(&record.resolved_at)
    .bind(record.it_lead_time_elapsed)
    .bind(record.it_lead_time_business)
    .execute(pool)
    .await
    .map_err(|err| format!("Failed to insert record: {err}"))?;

    Ok(())
}

async fn update_record(pool: &SqlitePool, record: &NewHireRecord) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE new_hire_metrics SET
            state = ?,
            title = ?,
            assignee_name = ?,
            requester = ?,
            created_at = ?,
            site = ?,
            division = ?,
            employee_type = ?,
            start_date = ?,
            success_factors_date_entered = ?,
            day_1_or_day_3 = ?,
            to_first_response_business = ?,
            to_resolution_business = ?,
            to_resolution_elapsed = ?,
            sla_breaches = ?,
            resolved_at = ?,
            it_lead_time_elapsed = ?,
            it_lead_time_business = ?
        WHERE number = ?
        "#,
    )
    .bind(&record.state)
    .bind(&record.title)
    .bind(&record.assignee_name)
    .bind(&record.requester)
    .bind(&record.created_at)
    .bind(&record.site)
    .bind(&record.division)
    .bind(&record.employee_type)
    .bind(&record.start_date)
    .bind(&record.success_factors_date_entered)
    .bind(&record.day_1_or_day_3)
    .bind(&record.to_first_response_business)
    .bind(&record.to_resolution_business)
    .bind(&record.to_resolution_elapsed)
    .bind(&record.sla_breaches)
    .bind(&record.resolved_at)
    .bind(record.it_lead_time_elapsed)
    .bind(record.it_lead_time_business)
    .bind(record.number)
    .execute(pool)
    .await
    .map_err(|err| format!("Failed to update record: {err}"))?;

    Ok(())
}

async fn fetch_existing_record(pool: &SqlitePool, number: i64) -> Result<NewHireRecord, String> {
    let row = sqlx::query(
        r#"
        SELECT
            number,
            state,
            title,
            assignee_name,
            requester,
            created_at,
            site,
            division,
            employee_type,
            start_date,
            success_factors_date_entered,
            day_1_or_day_3,
            to_first_response_business,
            to_resolution_business,
            to_resolution_elapsed,
            sla_breaches,
            resolved_at,
            it_lead_time_elapsed,
            it_lead_time_business
        FROM new_hire_metrics
        WHERE number = ?
        LIMIT 1
        "#,
    )
    .bind(number)
    .fetch_one(pool)
    .await
    .map_err(|err| format!("Failed to fetch existing record: {err}"))?;

    Ok(NewHireRecord {
        number: row.try_get("number").unwrap_or(number),
        state: row.try_get("state").unwrap_or(None),
        title: row.try_get("title").unwrap_or(None),
        assignee_name: row.try_get("assignee_name").unwrap_or(None),
        requester: row.try_get("requester").unwrap_or(None),
        created_at: row.try_get("created_at").unwrap_or(None),
        site: row.try_get("site").unwrap_or(None),
        division: row.try_get("division").unwrap_or(None),
        employee_type: row.try_get("employee_type").unwrap_or(None),
        start_date: row.try_get("start_date").unwrap_or(None),
        success_factors_date_entered: row
            .try_get("success_factors_date_entered")
            .unwrap_or(None),
        day_1_or_day_3: row.try_get("day_1_or_day_3").unwrap_or(None),
        to_first_response_business: row.try_get("to_first_response_business").unwrap_or(None),
        to_resolution_business: row.try_get("to_resolution_business").unwrap_or(None),
        to_resolution_elapsed: row.try_get("to_resolution_elapsed").unwrap_or(None),
        sla_breaches: row.try_get("sla_breaches").unwrap_or(None),
        resolved_at: row.try_get("resolved_at").unwrap_or(None),
        it_lead_time_elapsed: row.try_get("it_lead_time_elapsed").unwrap_or(None),
        it_lead_time_business: row.try_get("it_lead_time_business").unwrap_or(None),
    })
}

fn diff_records(existing: &NewHireRecord, incoming: &NewHireRecord) -> Vec<String> {
    let mut diffs = Vec::new();

    diff_opt_string(&mut diffs, "state", &existing.state, &incoming.state);
    diff_opt_string(&mut diffs, "title", &existing.title, &incoming.title);
    diff_opt_string(&mut diffs, "assignee", &existing.assignee_name, &incoming.assignee_name);
    diff_opt_string(&mut diffs, "requester", &existing.requester, &incoming.requester);
    diff_opt_string(&mut diffs, "created_at", &existing.created_at, &incoming.created_at);
    diff_opt_string(&mut diffs, "site", &existing.site, &incoming.site);
    diff_opt_string(&mut diffs, "division", &existing.division, &incoming.division);
    diff_opt_string(&mut diffs, "employee_type", &existing.employee_type, &incoming.employee_type);
    diff_opt_string(&mut diffs, "start_date", &existing.start_date, &incoming.start_date);
    diff_opt_string(
        &mut diffs,
        "success_factors",
        &existing.success_factors_date_entered,
        &incoming.success_factors_date_entered,
    );
    diff_opt_string(&mut diffs, "day_1_or_day_3", &existing.day_1_or_day_3, &incoming.day_1_or_day_3);
    diff_opt_string(
        &mut diffs,
        "first_response_business",
        &existing.to_first_response_business,
        &incoming.to_first_response_business,
    );
    diff_opt_string(
        &mut diffs,
        "resolution_business",
        &existing.to_resolution_business,
        &incoming.to_resolution_business,
    );
    diff_opt_string(
        &mut diffs,
        "resolution_elapsed",
        &existing.to_resolution_elapsed,
        &incoming.to_resolution_elapsed,
    );
    diff_opt_string(&mut diffs, "sla_breaches", &existing.sla_breaches, &incoming.sla_breaches);
    diff_opt_string(&mut diffs, "resolved_at", &existing.resolved_at, &incoming.resolved_at);
    diff_opt_i64(
        &mut diffs,
        "it_lead_time_elapsed",
        existing.it_lead_time_elapsed,
        incoming.it_lead_time_elapsed,
    );
    diff_opt_i64(
        &mut diffs,
        "it_lead_time_business",
        existing.it_lead_time_business,
        incoming.it_lead_time_business,
    );

    diffs
}

fn diff_opt_string(diffs: &mut Vec<String>, label: &str, old: &Option<String>, new: &Option<String>) {
    if old != new {
        diffs.push(format!(
            "{label}: {} -> {}",
            format_opt(old.as_ref()),
            format_opt(new.as_ref())
        ));
    }
}

fn diff_opt_i64(diffs: &mut Vec<String>, label: &str, old: Option<i64>, new: Option<i64>) {
    if old != new {
        diffs.push(format!(
            "{label}: {} -> {}",
            format_opt(old.as_ref()),
            format_opt(new.as_ref())
        ));
    }
}

fn format_opt<T: ToString>(value: Option<&T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "none".to_owned())
}
