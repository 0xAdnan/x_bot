use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
};

pub struct Config {
    pub repo_root: PathBuf,
    pub env_path: PathBuf,
    pub x_access: String,
    pub x_refresh: String,
    pub x_client_id: String,
    pub x_client_secret: String,
    pub x_user_id: String,
    pub _x_operator_handle: String,
    pub pitch_api_key: String,
    pub db_path: PathBuf,
}

impl Config {
    pub fn load() -> Self {
        let repo_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let env_path = repo_root.join(".env");

        if env_path.exists() {
            let _ = dotenvy::from_path_override(&env_path);
        } else {
            let _ = dotenvy::dotenv();
        }

        let db_path = env::var("SQLITE_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| repo_root.join("data").join("pitch_bot.db"));

        Config {
            repo_root: repo_root.clone(),
            env_path,
            x_access: env::var("X_USER_ACCESS_TOKEN").unwrap_or_default(),
            x_refresh: env::var("X_USER_REFRESH_TOKEN").unwrap_or_default(),
            x_client_id: env::var("X_CLIENT_ID").unwrap_or_default(),
            x_client_secret: env::var("X_CLIENT_SECRET").unwrap_or_default(),
            x_user_id: env::var("X_USER_ID").unwrap_or_default(),
            _x_operator_handle: env::var("X_OPERATOR_HANDLE")
                .unwrap_or_else(|_| "@trypitchdotco".to_string()),
            pitch_api_key: env::var("PITCH_API_KEY")
                .unwrap_or_else(|_| "pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB".to_string()),
            db_path,
        }
    }

    pub fn update_env(&self, updates: HashMap<String, String>) -> Result<(), std::io::Error> {
        let content = if self.env_path.exists() {
            fs::read_to_string(&self.env_path)?
        } else {
            String::new()
        };

        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut seen = std::collections::HashSet::new();

        for line in lines.iter_mut() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || !line.contains('=') {
                continue;
            }
            if let Some(key) = line.split('=').next().map(|k| k.trim().to_string()) {
                if let Some(val) = updates.get(&key) {
                    *line = format!("{}={}", key, val);
                    seen.insert(key);
                }
            }
        }

        for (k, v) in &updates {
            if !seen.contains(k) {
                lines.push(format!("{}={}", k, v));
            }
        }

        fs::write(&self.env_path, lines.join("\n") + "\n")?;

        for (k, v) in updates {
            env::set_var(k, v);
        }

        Ok(())
    }
}
