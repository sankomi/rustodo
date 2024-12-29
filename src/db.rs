use std::path::Path;

use sqlite::{Connection, State};

pub struct Task {
    pub id: i64,
    pub done: bool,
    pub subject: String,
    pub body: String,
    pub created: String,
}

pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn new() -> Self {
        #[cfg(not(test))]
        let file = "sqlite.db";
        #[cfg(test)]
        let file = "test.db";

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

    pub fn insert_one(&self, subject: &str, body: &str) -> Option<Task> {
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

    pub fn get_one(&self, id: i64) -> Option<Task> {
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

    pub fn list(&self) -> Vec<Task> {
        let mut tasks = vec![];

        let sql = "SELECT * FROM tasks;";
        let mut stat = self.connection.prepare(sql).unwrap();
        while let Ok(State::Row) = stat.next() {
            let task = Task {
                id: stat.read::<i64, _>("id").unwrap(),
                done: stat.read::<i64, _>("done").unwrap() == 1,
                subject: stat.read::<String, _>("subject").unwrap(),
                body: stat.read::<String, _>("body").unwrap(),
                created: stat.read::<String, _>("created").unwrap(),
            };
            tasks.push(task);
        }

        tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_one() {
        let subject = "test_subject";
        let body = "test_body";

        let db = Db::new();
        let insert = db.insert_one(subject, body);
        if let Some(task) = insert {
            assert_eq!(task.subject, subject);
            assert_eq!(task.body, body);
        } else {
            assert!(false, "failed to insert task!");
        }
    }

    #[test]
    fn test_insert_then_get_one() {
        let id;
        let subject = "test_subject";
        let body = "test_body";

        let db = Db::new();
        let insert = db.insert_one(subject, body);
        if let Some(task) = insert {
            id = task.id;
        } else {
            assert!(false, "failed to insert task!");
            return;
        }

        let get = db.get_one(id);
        if let Some(task) = get {
            assert_eq!(task.subject, subject);
            assert_eq!(task.body, body);
        } else {
            assert!(false, "failed to get task!");
        }
    }
}
