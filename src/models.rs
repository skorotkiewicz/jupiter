use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

// ── Auth ──

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPublic {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub bio: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub exp: usize,
}

// ── Chat ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub role: String,
    pub content: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub user_message: ChatMessage,
    pub agent_message: ChatMessage,
}

// ── Agent Profile ──

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AgentProfile {
    pub user_id: String,
    pub personality_summary: String,
    pub interests: String,
    pub core_values: String,
    pub communication_style: String,
    pub looking_for: String,
    pub deal_breakers: String,
    pub raw_notes: String,
    pub updated_at: String,
}

// ── Agent Peer Notes ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentPeerNote {
    pub id: i64,
    pub agent_user_id: String,
    pub about_user_id: String,
    pub compatibility_score: f64,
    pub notes: String,
    pub recommends_match: bool,
    pub conversation_count: i32,
    pub updated_at: String,
}

// ── Matches ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchRecord {
    pub id: i64,
    pub user_a_id: String,
    pub user_b_id: String,
    pub agent_a_approves: bool,
    pub agent_b_approves: bool,
    pub is_matched: bool,
    pub created_at: String,
    pub updated_at: String,
    pub other_user: Option<UserPublic>,
}

// ── Notifications ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub id: i64,
    pub user_id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub related_user_id: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

// ── Direct Messages ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectMessage {
    pub id: i64,
    pub match_id: i64,
    pub sender_id: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendDirectMessageRequest {
    pub content: String,
}

// ── LLM Types ──

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmChoice {
    pub message: LlmMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    pub choices: Vec<LlmChoice>,
}

// ── Profile Update ──

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

// ── Matching trigger ──

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchingStatus {
    pub evaluated: usize,
    pub new_recommendations: usize,
    pub new_matches: usize,
}
