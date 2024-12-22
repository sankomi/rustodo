use std::path::Path;

use sqlite::{State, Connection};

use super::Stuff;

pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn new() -> Self {
        let ready = Path::new("sqlite.db").exists();

        let connection = sqlite::open("sqlite.db").unwrap();

        if !ready {
            let query = "
                CREATE TABLE todos (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    done BOOLEAN DEFAULT 0,
                    text VARCHAR(255) DEFAULT ''
                );
                INSERT INTO todos (text, done)
                VALUES
                    ('do this', 0),
                    ('be there', 0),
                    ('stop that', 0),
                    ('see here', 0),
                    ('sudo rm -rf /', 1);
            ";
            connection.execute(query).unwrap();
        }

        Db { connection }
    }

    pub fn get_todos(&self) -> (Vec<Stuff>, Vec<Stuff>) {
        let mut todos: Vec<Stuff> = vec![];
        let mut dones: Vec<Stuff> = vec![];

        let query = "SELECT * FROM todos;";
        let mut stat = self.connection.prepare(query).unwrap();
        while let Ok(State::Row) = stat.next() {
            let id = stat.read::<i64, _>("id").unwrap();
            let done = stat.read::<i64, _>("done").unwrap() == 1;
            let text = stat.read::<String, _>("text").unwrap();
            let stuff = Stuff { id, done, text };

            if done {
                dones.push(stuff);
            } else {
                todos.push(stuff);
            }
        }

        (todos, dones)
    }

    pub fn flip(&self, id: i64) {
        let query = "SELECT * FROM todos WHERE id = ?;";
        let stat = self.connection.prepare(query).unwrap();
        let rows = stat.into_iter().bind((1, id)).unwrap().map(|row| row.unwrap());

        for row in rows {
            let done = 1 - row.read::<i64, _>("done");
            let query = "UPDATE todos SET done = ? WHERE id = ?;";
            let stat = self.connection.prepare(query).unwrap();
            stat.into_iter().bind((1, done)).unwrap().bind((2, id)).unwrap().next();
            break;
        }
    }

    pub fn add_todo(&self, text: &str) {
        let query = "INSERT INTO todos (text) VALUES (?);";
        let mut stat = self.connection.prepare(query).unwrap();
        stat.bind((1, text)).unwrap();
        stat.next().unwrap();
    }

    pub fn edit_todo(&self, id: i64, text: &str) {
        let query = "UPDATE todos SET text = ? WHERE id = ?;";
        let mut stat = self.connection.prepare(query).unwrap();
        stat.bind((1, text)).unwrap();
        stat.bind((2, id)).unwrap();
        stat.next().unwrap();
    }
}
