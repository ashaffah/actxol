use log::error;
use mongodb::{ Client, Database };
use std::borrow::ToOwned;
use std::env;
use postgres::{ Client as ClientPos, NoTls };

pub struct AppStates {
    // pub postgres_db: ClientPos,
    // pub client_mongo: Client,
    pub mongo_db: Database,
}

// MongoDB initialize function.
// Get DB connection url from environment file and connect.
pub async fn init() -> Client {
    let uri = match env::var("MONGODB_URI") {
        Ok(uri) => uri,
        Err(_) => {
            error!("Error loading env info for MongoDB connection");
            "Error loading env variables to connect to MongoDB".to_owned()
        }
    };

    // panic if not able to connect to DB.
    let client = Client::with_uri_str(uri).await.expect("Error connecting to backend database");
    client
}

pub async fn connect() -> Result<postgres::Client, postgres::Error> {
    let uri = match env::var("POSGRES_URI") {
        Ok(uri) => uri,
        Err(_) => {
            error!("Error loading env info for PostgresDB connection");
            "Error loading env variables to connect to PostgresDB".to_owned()
        }
    };
    let client = ClientPos::connect(&uri, NoTls);
    client
}
