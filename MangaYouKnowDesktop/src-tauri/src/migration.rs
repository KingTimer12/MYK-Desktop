use tauri_plugin_sql::{Builder, Migration, MigrationKind};

pub struct Migrations {
    migrations: Vec<Migration>,
}

impl Migrations {
    pub fn new() -> Self {
        let migrations = vec![
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL
                username TEXT NOT NULL,
                icon TEXT DEFAULT 'https://cdn.discordapp.com/embed/avatars/0.png',
                password TEXT DEFAULT '',
                is_authenticated BOOLEAN DEFAULT false
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS favorite (
              id                 INTEGER PRIMARY KEY AUTOINCREMENT,
              user_id            INTEGER NOT NULL,
              name               TEXT NOT NULL,
              folder_name        TEXT NOT NULL,
              link               TEXT NOT NULL DEFAULT '',
              cover              TEXT NOT NULL,
              source             TEXT NOT NULL,
              source_id          TEXT NOT NULL,
              type               TEXT DEFAULT 'manga',
              extra_name         TEXT DEFAULT '',
              title_color        TEXT DEFAULT '',
              card_color         TEXT DEFAULT '',
              grade              REAL DEFAULT 0.0,
              author             TEXT DEFAULT 'Unknow',
              is_ultra_favorite  BOOLEAN DEFAULT false,
              description        TEXT DEFAULT '',
              FOREIGN KEY (user_id) REFERENCES user(id),
              UNIQUE (source_id, source, type, user_id)
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS mark (
              id                 INTEGER PRIMARY KEY AUTOINCREMENT,
              name               TEXT NOT NULL,
              user_id            INTEGER NOT NULL,
              color              TEXT DEFAULT '',
              FOREIGN KEY (user_id) REFERENCES user(id)
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS readed (
              id                 INTEGER PRIMARY KEY AUTOINCREMENT,
              chapter_id         TEXT NOT NULL,
              source             TEXT NOT NULL,
              language           TEXT DEFAULT 'default',
              favorite_id        INTEGER NOT NULL,
              FOREIGN KEY (favorite_id) REFERENCES favorite(id),
              UNIQUE(chapter_id, source, language, favorite_id)
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS mark_favorites (
              mark_id            INTEGER NOT NULL,
              favorite_id        INTEGER NOT NULL,
              PRIMARY KEY (mark_id, favorite_id),
              FOREIGN KEY (mark_id) REFERENCES mark(id),
              FOREIGN KEY (favorite_id) REFERENCES favorite(id)
            );
            "#
        ];
        // convert str in migration struct
        let migrations: Vec<Migration> = migrations.iter().map(|query| {
            Migration {
                kind: MigrationKind::Up,
                version: 1,
                sql: query,
                description: "Create table",
            }
        }).collect();
        
        Self { migrations }
    }

    pub fn builder(self) -> Builder {
        Builder::new()
            .add_migrations("sqlite:mydatabase.db", self.migrations)
    }
}