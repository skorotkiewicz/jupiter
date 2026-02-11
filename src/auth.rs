use actix_web::{web, HttpRequest, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::db::Database;
use crate::models::*;

pub fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "jupiter-secret-key-change-me".to_string())
}

pub fn extract_user_id(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing Authorization header"})))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid Authorization format"})))?;

    let secret = jwt_secret();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        log::warn!("JWT decode error: {}", e);
        HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid or expired token"}))
    })?;

    Ok(token_data.claims)
}

pub async fn register(
    db: web::Data<Database>,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    let username = body.username.trim().to_string();
    let email = body.email.trim().to_lowercase();
    let password = body.password.clone();

    if username.len() < 3 || password.len() < 6 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Username must be 3+ chars, password 6+ chars"
        }));
    }

    let password_hash = match hash(&password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to hash password"})),
    };

    let user_id = Uuid::new_v4().to_string();
    let display_name = body.display_name.clone().unwrap_or_else(|| username.clone());

    let conn = db.conn.lock().unwrap();

    // Check if user exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE username = ?1 OR email = ?2",
            rusqlite::params![&username, &email],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;

    if exists {
        return HttpResponse::Conflict().json(serde_json::json!({"error": "Username or email already exists"}));
    }

    if let Err(e) = conn.execute(
        "INSERT INTO users (id, username, email, password_hash, display_name) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![&user_id, &username, &email, &password_hash, &display_name],
    ) {
        log::error!("Register error: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to create user"}));
    }

    // Create agent profile
    let _ = conn.execute(
        "INSERT INTO agent_profiles (user_id) VALUES (?1)",
        rusqlite::params![&user_id],
    );

    // Generate JWT
    let secret = jwt_secret();
    let claims = Claims {
        sub: user_id.clone(),
        username: username.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .unwrap_or_default();

    HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserPublic {
            id: user_id,
            username,
            email,
            display_name,
            bio: String::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    })
}

pub async fn login(
    db: web::Data<Database>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let conn = db.conn.lock().unwrap();

    let result = conn.query_row(
        "SELECT id, username, email, password_hash, display_name, bio, created_at FROM users WHERE username = ?1",
        rusqlite::params![&body.username],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        },
    );

    match result {
        Ok((id, username, email, password_hash, display_name, bio, created_at)) => {
            if !verify(&body.password, &password_hash).unwrap_or(false) {
                return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid credentials"}));
            }

            let secret = jwt_secret();
            let claims = Claims {
                sub: id.clone(),
                username: username.clone(),
                exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
            };
            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
                .unwrap_or_default();

            HttpResponse::Ok().json(AuthResponse {
                token,
                user: UserPublic {
                    id,
                    username,
                    email,
                    display_name,
                    bio,
                    created_at,
                },
            })
        }
        Err(_) => HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid credentials"})),
    }
}

pub async fn get_profile(
    req: HttpRequest,
    db: web::Data<Database>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let conn = db.conn.lock().unwrap();
    let result = conn.query_row(
        "SELECT id, username, email, display_name, bio, created_at FROM users WHERE id = ?1",
        rusqlite::params![&claims.sub],
        |row| {
            Ok(UserPublic {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                display_name: row.get(3)?,
                bio: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    );

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
    }
}

pub async fn update_profile(
    req: HttpRequest,
    db: web::Data<Database>,
    body: web::Json<UpdateProfileRequest>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let conn = db.conn.lock().unwrap();

    if let Some(ref name) = body.display_name {
        let _ = conn.execute(
            "UPDATE users SET display_name = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![name, &claims.sub],
        );
    }
    if let Some(ref bio) = body.bio {
        let _ = conn.execute(
            "UPDATE users SET bio = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![bio, &claims.sub],
        );
    }

    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}
