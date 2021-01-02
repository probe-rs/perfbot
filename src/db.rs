use diesel::{Connection, SqliteConnection};
pub fn establish_connection(database_path: &str) -> SqliteConnection {
    SqliteConnection::establish(&database_path)
        .expect(&format!("Error connecting to {}", database_path))
}
