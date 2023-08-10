use diesel::{Insertable, Queryable, Selectable};
use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Insertable, Deserialize, Serialize, Validate)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewUser {
    #[garde(ascii, length(min = 3, max = 20))]
    pub name: String,

    #[garde(email)]
    pub email: String,

    #[garde(ascii, length(min = 5))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct LoginUser {
    #[garde(email)]
    pub email: String,

    #[garde(alphanumeric, length(min = 8))]
    pub password: String,
}

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ShowUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
