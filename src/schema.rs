// @generated automatically by Diesel CLI.

diesel::table! {
    languages (id) {
        id -> Integer,
        name -> Text,
        extension -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        password -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        verified -> Integer,
    }
}

diesel::table! {
    users_languages (user_id, language_id) {
        user_id -> Integer,
        language_id -> Integer,
        seconds -> BigInt,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(users_languages -> languages (language_id));
diesel::joinable!(users_languages -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(languages, users, users_languages,);
