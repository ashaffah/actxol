use postgres::{ Client, NoTls };

use crate::config::ConnectPosgresDB;

pub struct PostgreSQL {
    connection_string: String,
}

impl PostgreSQL {
    pub fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
        }
    }
}

impl ConnectPosgresDB for PostgreSQL {
    fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _client = Client::connect(&self.connection_string, NoTls)?;
        Ok(())
    }

    fn get_connection_string(&self) -> &str {
        &self.connection_string
    }
}
