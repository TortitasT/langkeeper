use diesel::Insertable;
use serde::{Deserialize, Serialize};

#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::languages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewLanguage {
    pub name: String,
    pub extension: String,
}

#[derive(Deserialize, Serialize)]
pub struct PingRequest {
    pub extension: String,
}

#[derive(Serialize, Deserialize)]
pub struct PingResponse {
    pub user_id: i32,
    pub language_id: i32,
    pub language_name: String,
    pub language_extension: String,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
}

#[derive(Serialize, Deserialize)]
pub struct LanguageStats {
    pub language_id: i32,
    pub language_name: String,
    pub language_extension: String,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
}
