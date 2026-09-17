#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{env, net::SocketAddr};

use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post, put},
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, sqlite::SqlitePoolOptions};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

#[derive(Debug)]
struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "database error");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erro ao acessar o banco de dados".into(),
        )
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Deserialize)]
struct YearQuery {
    year: Option<i32>,
}

#[derive(Serialize, FromRow)]
struct Entry {
    id: i64,
    entry_date: String,
    description: String,
    document: String,
    category: String,
    entry_type: String,
    amount_cents: i64,
}

#[derive(Deserialize)]
struct EntryInput {
    entry_date: String,
    description: String,
    #[serde(default)]
    document: String,
    #[serde(default)]
    category: String,
    entry_type: String,
    amount_cents: i64,
}

#[derive(Serialize)]
struct LedgerResponse {
    year: i32,
    opening_balance_cents: i64,
    total_income_cents: i64,
    total_expense_cents: i64,
    final_balance_cents: i64,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct BalanceInput {
    opening_balance_cents: i64,
}

#[derive(Serialize)]
struct YearList {
    years: Vec<i32>,
}

fn validate_year(year: i32) -> ApiResult<i32> {
    if (1900..=2200).contains(&year) {
        Ok(year)
    } else {
        Err(ApiError(StatusCode::BAD_REQUEST, "Ano inválido".into()))
    }
}

fn validate_entry(input: &EntryInput) -> ApiResult<NaiveDate> {
    let date = NaiveDate::parse_from_str(&input.entry_date, "%Y-%m-%d")
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "Data inválida".into()))?;
    if input.description.trim().is_empty() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Informe a descrição".into(),
        ));
    }
    if !matches!(input.entry_type.as_str(), "entrada" | "saida") {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Tipo de lançamento inválido".into(),
        ));
    }
    if input.amount_cents <= 0 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "O valor deve ser maior que zero".into(),
        ));
    }
    Ok(date)
}

async fn health() -> &'static str {
    "ok"
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn styles() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/styles.css"),
    )
}

async fn javascript() -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../static/app.js"),
    )
}

async fn list_years(State(state): State<AppState>) -> ApiResult<Json<YearList>> {
    let current = Utc::now().year();
    let rows: Vec<(i32,)> = sqlx::query_as(
        "SELECT year FROM yearly_balances UNION SELECT CAST(substr(entry_date, 1, 4) AS INTEGER) FROM entries ORDER BY 1 DESC"
    ).fetch_all(&state.db).await?;
    let mut years: Vec<i32> = rows.into_iter().map(|r| r.0).collect();
    if !years.contains(&current) {
        years.insert(0, current);
    }
    Ok(Json(YearList { years }))
}

async fn get_ledger(
    State(state): State<AppState>,
    Query(q): Query<YearQuery>,
) -> ApiResult<Json<LedgerResponse>> {
    let year = validate_year(q.year.unwrap_or_else(|| Utc::now().year()))?;
    let start = format!("{year:04}-01-01");
    let end = format!("{:04}-01-01", year + 1);
    let manual_opening: Option<(i64,)> =
        sqlx::query_as("SELECT opening_balance_cents FROM yearly_balances WHERE year = ?")
            .bind(year)
            .fetch_optional(&state.db)
            .await?;
    let opening = if let Some((value,)) = manual_opening {
        value
    } else {
        // Sem ajuste manual, transporta o saldo acumulado até o fim do ano anterior.
        // O ajuste manual mais recente funciona como novo ponto de partida.
        let base: Option<(i32, i64)> = sqlx::query_as(
            "SELECT year, opening_balance_cents FROM yearly_balances WHERE year < ? ORDER BY year DESC LIMIT 1",
        )
        .bind(year)
        .fetch_optional(&state.db)
        .await?;
        let movement_start = base
            .map(|(base_year, _)| format!("{base_year:04}-01-01"))
            .unwrap_or_else(|| "1900-01-01".to_string());
        let base_value = base.map(|(_, value)| value).unwrap_or(0);
        let totals: (i64, i64) = sqlx::query_as(
            "SELECT COALESCE(SUM(CASE WHEN entry_type = 'entrada' THEN amount_cents ELSE 0 END), 0), COALESCE(SUM(CASE WHEN entry_type = 'saida' THEN amount_cents ELSE 0 END), 0) FROM entries WHERE entry_date >= ? AND entry_date < ?",
        )
        .bind(movement_start)
        .bind(&start)
        .fetch_one(&state.db)
        .await?;
        base_value + totals.0 - totals.1
    };
    let entries = sqlx::query_as::<_, Entry>(
        "SELECT id, entry_date, description, document, category, entry_type, amount_cents FROM entries WHERE entry_date >= ? AND entry_date < ? ORDER BY entry_date, id"
    ).bind(start).bind(end).fetch_all(&state.db).await?;
    let income = entries
        .iter()
        .filter(|e| e.entry_type == "entrada")
        .map(|e| e.amount_cents)
        .sum();
    let expense = entries
        .iter()
        .filter(|e| e.entry_type == "saida")
        .map(|e| e.amount_cents)
        .sum();
    Ok(Json(LedgerResponse {
        year,
        opening_balance_cents: opening,
        total_income_cents: income,
        total_expense_cents: expense,
        final_balance_cents: opening + income - expense,
        entries,
    }))
}

async fn set_balance(
    State(state): State<AppState>,
    Path(year): Path<i32>,
    Json(input): Json<BalanceInput>,
) -> ApiResult<StatusCode> {
    let year = validate_year(year)?;
    sqlx::query("INSERT INTO yearly_balances (year, opening_balance_cents, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP) ON CONFLICT(year) DO UPDATE SET opening_balance_cents = excluded.opening_balance_cents, updated_at = CURRENT_TIMESTAMP")
        .bind(year).bind(input.opening_balance_cents).execute(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_entry(
    State(state): State<AppState>,
    Json(input): Json<EntryInput>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    validate_entry(&input)?;
    let result = sqlx::query("INSERT INTO entries (entry_date, description, document, category, entry_type, amount_cents) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&input.entry_date).bind(input.description.trim()).bind(input.document.trim()).bind(input.category.trim()).bind(&input.entry_type).bind(input.amount_cents)
        .execute(&state.db).await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": result.last_insert_rowid() })),
    ))
}

async fn update_entry(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<EntryInput>,
) -> ApiResult<StatusCode> {
    validate_entry(&input)?;
    let result = sqlx::query("UPDATE entries SET entry_date = ?, description = ?, document = ?, category = ?, entry_type = ?, amount_cents = ? WHERE id = ?")
        .bind(&input.entry_date).bind(input.description.trim()).bind(input.document.trim()).bind(input.category.trim()).bind(&input.entry_type).bind(input.amount_cents).bind(id)
        .execute(&state.db).await?;
    if result.rows_affected() == 0 {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "Lançamento não encontrado".into(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_entry(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<StatusCode> {
    let result = sqlx::query("DELETE FROM entries WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "Lançamento não encontrado".into(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("livro_caixa_rv=info".parse()?),
        )
        .init();
    // Ao abrir por duplo clique, mantém o banco ao lado do executável portátil.
    if env::var("DATABASE_URL").is_err()
        && let Some(executable_dir) = env::current_exe()?.parent()
    {
        env::set_current_dir(executable_dir)?;
    }
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://livro_caixa.db?mode=rwc".into());
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("não foi possível abrir o banco")?;
    sqlx::migrate!()
        .run(&db)
        .await
        .context("falha ao executar migrações")?;
    let state = AppState { db };
    let app = Router::new()
        .route("/", get(index))
        .route("/styles.css", get(styles))
        .route("/app.js", get(javascript))
        .route("/api/health", get(health))
        .route("/api/years", get(list_years))
        .route("/api/ledger", get(get_ledger))
        .route("/api/years/{year}/balance", put(set_balance))
        .route("/api/entries", post(create_entry))
        .route("/api/entries/{id}", put(update_entry).delete(remove_entry))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!(%address, "Livro Caixa RV iniciado");
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
