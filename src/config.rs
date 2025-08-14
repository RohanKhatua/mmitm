use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub google_maps_api_key: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let google_maps_api_key = env::var("GOOGLE_MAPS_API_KEY")
            .map_err(|_| "GOOGLE_MAPS_API_KEY environment variable is required")?;

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| "Invalid PORT value")?;

        Ok(Config {
            google_maps_api_key,
            port,
        })
    }
}
