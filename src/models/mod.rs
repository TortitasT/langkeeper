use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub verified: i32,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::languages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Language {
    pub id: i32,
    pub name: String,
    pub extension: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Serialize, Deserialize)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Language))]
#[diesel(table_name = crate::schema::users_languages)]
#[diesel(primary_key(user_id, language_id))]
pub struct UserLanguage {
    pub user_id: i32,
    pub language_id: i32,
    pub seconds: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Serialize, Deserialize)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Language))]
#[diesel(table_name = crate::schema::users_languages_weekly)]
#[diesel(primary_key(id))]
pub struct UserLanguageWeekly {
    pub id: i32,
    pub user_id: i32,
    pub language_id: i32,
    pub seconds: i64,
    pub created_at: chrono::NaiveDateTime,
}
