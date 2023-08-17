use crate::{resources::languages::NewLanguage, DbPool};
use diesel::RunQueryDsl;

pub fn seed(pool: &DbPool) {
    let languages = vec![
        NewLanguage {
            name: "Rust".to_string(),
            extension: "rs".to_string(),
        },
        NewLanguage {
            name: "JavaScript".to_string(),
            extension: "js".to_string(),
        },
        NewLanguage {
            name: "TypeScript".to_string(),
            extension: "ts".to_string(),
        },
        NewLanguage {
            name: "Python".to_string(),
            extension: "py".to_string(),
        },
        NewLanguage {
            name: "Java".to_string(),
            extension: "java".to_string(),
        },
        NewLanguage {
            name: "C#".to_string(),
            extension: "cs".to_string(),
        },
        NewLanguage {
            name: "C++".to_string(),
            extension: "cpp".to_string(),
        },
        NewLanguage {
            name: "C".to_string(),
            extension: "c".to_string(),
        },
        NewLanguage {
            name: "Go".to_string(),
            extension: "go".to_string(),
        },
        NewLanguage {
            name: "PHP".to_string(),
            extension: "php".to_string(),
        },
        NewLanguage {
            name: "Ruby".to_string(),
            extension: "rb".to_string(),
        },
        NewLanguage {
            name: "Swift".to_string(),
            extension: "swift".to_string(),
        },
        NewLanguage {
            name: "Kotlin".to_string(),
            extension: "kt".to_string(),
        },
        NewLanguage {
            name: "Dart".to_string(),
            extension: "dart".to_string(),
        },
        NewLanguage {
            name: "Vue".to_string(),
            extension: "vue".to_string(),
        },
        NewLanguage {
            name: "React".to_string(),
            extension: "jsx".to_string(),
        },
        NewLanguage {
            name: "Json".to_string(),
            extension: "json".to_string(),
        },
        NewLanguage {
            name: "Html".to_string(),
            extension: "html".to_string(),
        },
        NewLanguage {
            name: "Css".to_string(),
            extension: "css".to_string(),
        },
        NewLanguage {
            name: "Sass".to_string(),
            extension: "sass".to_string(),
        },
        NewLanguage {
            name: "Scss".to_string(),
            extension: "scss".to_string(),
        },
        NewLanguage {
            name: "Sh".to_string(),
            extension: "sh".to_string(),
        },
        NewLanguage {
            name: "Sql".to_string(),
            extension: "sql".to_string(),
        },
        NewLanguage {
            name: "Yaml".to_string(),
            extension: "yaml".to_string(),
        },
        NewLanguage {
            name: "Toml".to_string(),
            extension: "toml".to_string(),
        },
        NewLanguage {
            name: "Xml".to_string(),
            extension: "xml".to_string(),
        },
        NewLanguage {
            name: "Markdown".to_string(),
            extension: "md".to_string(),
        },
    ];

    diesel::insert_into(crate::schema::languages::table)
        .values(languages)
        .execute(&mut pool.get().unwrap())
        .expect("Error seeding languages");
}
