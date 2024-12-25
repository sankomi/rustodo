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
                    title VARCHAR(255) DEFAULT '',
                    content TEXT DEFAULT ''
                );
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
            let title = stat.read::<String, _>("title").unwrap();
            let content = stat.read::<String, _>("content").unwrap();
            let stuff = Stuff { id, title, content };

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

    pub fn add_todo(&self, title: &str, content: &str) {
        let query = "INSERT INTO todos (title, content) VALUES (?, ?);";
        let mut stat = self.connection.prepare(query).unwrap();
        stat.bind((1, title)).unwrap();
        stat.bind((2, content)).unwrap();
        stat.next().unwrap();
    }

    pub fn edit_todo(&self, id: i64, title: &str, content: &str) {
        let query = "UPDATE todos SET title = ?, content = ? WHERE id = ?;";
        let mut stat = self.connection.prepare(query).unwrap();
        stat.bind((1, title)).unwrap();
        stat.bind((2, content)).unwrap();
        stat.bind((3, id)).unwrap();
        stat.next().unwrap();
    }
}
