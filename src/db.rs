use std::path::Path;

use sqlite::{State, Connection};

pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn new() -> Self {
        let file = "sqlite.db";
        let exists = Path::new(file).exists();
        let connection = sqlite::open(file).unwrap();
        let db = Db { connection };

        if !exists {
            db.init_tables();
        }

        db
    }

    fn init_tables(&self) {
        let sql = "
            CREATE TABLE tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                done BOOLEAN NOT NULL DEFAULT 0,
                subject VARCHAR(50) NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                created DATETIME DEFAULT CURRENT_TIMESTAMP
            );
        ";
        self.connection.execute(sql).unwrap();
    }

    fn insert_one(&self, subject: &str, body: &str) -> Option<Task> {
        let sql = "
            INSERT INTO tasks (subject, body)
            VALUES (?, ?)
            RETURNING *;
        ";
        let mut stat = self.connection.prepare(sql).unwrap();
        stat.bind((1, subject)).unwrap();
        stat.bind((2, body)).unwrap();
        while let Ok(State::Row) = stat.next() {
            return Some(Task {
                id: stat.read::<i64, _>("id").unwrap(),
                done: stat.read::<i64, _>("done").unwrap() == 1,
                subject: stat.read::<String, _>("subject").unwrap(),
                body: stat.read::<String, _>("body").unwrap(),
                created: stat.read::<String, _>("created").unwrap(),
            });
        }

        None
    }

    fn get_one(&self, id: i64) -> Option<Task> {
        let sql = "SELECT * FROM tasks WHERE id = ?";
        let mut stat = self.connection.prepare(sql).unwrap();
        stat.bind((1, id)).unwrap();
        while let Ok(State::Row) = stat.next() {
            return Some(Task {
                id: stat.read::<i64, _>("id").unwrap(),
                done: stat.read::<i64, _>("done").unwrap() == 1,
                subject: stat.read::<String, _>("subject").unwrap(),
                body: stat.read::<String, _>("body").unwrap(),
                created: stat.read::<String, _>("created").unwrap(),
            });
        }

        None
    }
}

struct Task {
    id: i64,
    done: bool,
    subject: String,
    body: String,
    created: String,
}
