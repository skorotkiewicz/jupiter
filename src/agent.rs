use reqwest::Client;
use serde_json::json;

use crate::models::*;

pub struct LlmAgent {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
}

impl LlmAgent {
    pub fn new() -> Self {
        LlmAgent {
            client: Client::new(),
            base_url: std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434/v1".to_string()),
            model: std::env::var("LLM_MODEL")
                .unwrap_or_else(|_| "llama3".to_string()),
            api_key: std::env::var("LLM_API_KEY")
                .unwrap_or_else(|_| "not-needed".to_string()),
        }
    }

    async fn call_llm(&self, messages: Vec<LlmMessage>, temperature: f64, max_tokens: u32) -> Result<String, String> {
        let request = LlmRequest {
            model: self.model.clone(),
            messages,
            temperature: Some(temperature),
            max_tokens: Some(max_tokens),
        };

        let url = format!("{}/chat/completions", self.base_url);
        log::info!("LLM request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("LLM request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("LLM error {}: {}", status, body));
        }

        let llm_response: LlmResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

        llm_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "No response from LLM".to_string())
    }

    /// Chat with user — the personal agent conversation
    pub async fn chat_with_user(
        &self,
        history: &[ChatMessage],
        agent_profile: &AgentProfile,
        user_message: &str,
    ) -> Result<String, String> {
        let system_prompt = format!(
            r#"You are Jupiter, a warm, empathetic AI companion. Your job is to get to know your user deeply — their personality, interests, values, dreams, what they're looking for in a partner, and their daily life.

You should be conversational, curious, and genuinely interested. Ask thoughtful follow-up questions. Remember everything they tell you.

Current knowledge about this user:
- Personality: {}
- Interests: {}
- Core Values: {}
- Communication style: {}
- Looking for in a partner: {}
- Deal breakers: {}
- Additional notes: {}

Guidelines:
1. If this is a new user (empty profile), start by warmly welcoming them and asking about themselves
2. Be natural — don't interrogate. Have a real conversation
3. Periodically ask about what they're looking for in a partner
4. Remember and reference things they've told you before
5. Be supportive, positive, but honest
6. Keep responses concise but warm (2-4 paragraphs max)"#,
            if agent_profile.personality_summary.is_empty() { "Not yet known" } else { &agent_profile.personality_summary },
            if agent_profile.interests.is_empty() { "Not yet known" } else { &agent_profile.interests },
            if agent_profile.core_values.is_empty() { "Not yet known" } else { &agent_profile.core_values },
            if agent_profile.communication_style.is_empty() { "Not yet known" } else { &agent_profile.communication_style },
            if agent_profile.looking_for.is_empty() { "Not yet known" } else { &agent_profile.looking_for },
            if agent_profile.deal_breakers.is_empty() { "Not yet known" } else { &agent_profile.deal_breakers },
            if agent_profile.raw_notes.is_empty() { "None yet" } else { &agent_profile.raw_notes },
        );

        let mut messages = vec![LlmMessage {
            role: "system".to_string(),
            content: system_prompt,
        }];

        // Add recent conversation history (last 20 messages for context)
        for msg in history.iter().rev().take(20).rev() {
            messages.push(LlmMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        messages.push(LlmMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
        });

        self.call_llm(messages, 0.8, 1024).await
    }

    /// After a conversation, update the agent's understanding of the user
    pub async fn update_user_profile(
        &self,
        history: &[ChatMessage],
        current_profile: &AgentProfile,
    ) -> Result<AgentProfile, String> {
        let recent_conversation: String = history
            .iter()
            .rev()
            .take(30)
            .rev()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            r#"Based on the following conversation with a user, update the user profile. Extract and summarize key information.

Current profile:
- Personality: {}
- Interests: {}
- Core Values: {}
- Communication style: {}
- Looking for in a partner: {}
- Deal breakers: {}
- Additional notes: {}

Recent conversation:
{}

Respond in EXACTLY this JSON format (update fields with new info, keep existing info that's still valid):
{{
    "personality_summary": "...",
    "interests": "...",
    "core_values": "...",
    "communication_style": "...",
    "looking_for": "...",
    "deal_breakers": "...",
    "raw_notes": "..."
}}

Only output the JSON, nothing else."#,
            current_profile.personality_summary,
            current_profile.interests,
            current_profile.core_values,
            current_profile.communication_style,
            current_profile.looking_for,
            current_profile.deal_breakers,
            current_profile.raw_notes,
            recent_conversation,
        );

        let messages = vec![
            LlmMessage {
                role: "system".to_string(),
                content: "You are a profile analysis AI. You extract personality traits, interests, values, and preferences from conversations. Always respond with valid JSON only.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let response = self.call_llm(messages, 0.3, 2048).await?;

        // Try to parse the JSON response
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let parsed: serde_json::Value = serde_json::from_str(cleaned)
            .map_err(|e| format!("Failed to parse profile update: {} — raw: {}", e, cleaned))?;

        Ok(AgentProfile {
            user_id: current_profile.user_id.clone(),
            personality_summary: parsed["personality_summary"]
                .as_str()
                .unwrap_or(&current_profile.personality_summary)
                .to_string(),
            interests: parsed["interests"]
                .as_str()
                .unwrap_or(&current_profile.interests)
                .to_string(),
            core_values: parsed["core_values"]
                .as_str()
                .unwrap_or(&current_profile.core_values)
                .to_string(),
            communication_style: parsed["communication_style"]
                .as_str()
                .unwrap_or(&current_profile.communication_style)
                .to_string(),
            looking_for: parsed["looking_for"]
                .as_str()
                .unwrap_or(&current_profile.looking_for)
                .to_string(),
            deal_breakers: parsed["deal_breakers"]
                .as_str()
                .unwrap_or(&current_profile.deal_breakers)
                .to_string(),
            raw_notes: parsed["raw_notes"]
                .as_str()
                .unwrap_or(&current_profile.raw_notes)
                .to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Agent-to-agent evaluation: one agent evaluates another user for compatibility
    pub async fn evaluate_compatibility(
        &self,
        my_user_profile: &AgentProfile,
        other_user_profile: &AgentProfile,
        existing_notes: Option<&AgentPeerNote>,
    ) -> Result<(f64, String, bool), String> {
        let previous_context = match existing_notes {
            Some(notes) => format!(
                "\nPrevious evaluation notes: {}\nPrevious compatibility score: {:.0}%\nTimes evaluated: {}",
                notes.notes, notes.compatibility_score * 100.0, notes.conversation_count
            ),
            None => String::from("\nThis is the first evaluation."),
        };

        let prompt = format!(
            r#"You are an AI matchmaking agent. Your client has the following profile:

YOUR CLIENT:
- Personality: {}
- Interests: {}
- Core Values: {}
- Communication style: {}
- Looking for: {}
- Deal breakers: {}

POTENTIAL MATCH:
- Personality: {}
- Interests: {}
- Core Values: {}
- Communication style: {}
- Looking for: {}
- Deal breakers: {}
{}

Evaluate the compatibility between your client and this potential match. Consider:
1. Shared interests and values
2. Compatible communication styles
3. Whether each person matches what the other is looking for
4. Any deal breakers
5. Potential for genuine connection

Respond in EXACTLY this JSON format:
{{
    "compatibility_score": 0.75,
    "notes": "Detailed analysis of compatibility...",
    "recommends_match": true
}}

Score from 0.0 to 1.0. recommends_match should be true if score >= 0.65.
Only output JSON, nothing else."#,
            my_user_profile.personality_summary,
            my_user_profile.interests,
            my_user_profile.core_values,
            my_user_profile.communication_style,
            my_user_profile.looking_for,
            my_user_profile.deal_breakers,
            other_user_profile.personality_summary,
            other_user_profile.interests,
            other_user_profile.core_values,
            other_user_profile.communication_style,
            other_user_profile.looking_for,
            other_user_profile.deal_breakers,
            previous_context,
        );

        let messages = vec![
            LlmMessage {
                role: "system".to_string(),
                content: "You are a compatibility evaluation AI for a dating app. Be thorough but fair. Respond with JSON only.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let response = self.call_llm(messages, 0.4, 1024).await?;

        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let parsed: serde_json::Value = serde_json::from_str(cleaned)
            .map_err(|e| format!("Failed to parse compatibility: {} — raw: {}", e, cleaned))?;

        let score = parsed["compatibility_score"]
            .as_f64()
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let notes = parsed["notes"]
            .as_str()
            .unwrap_or("No notes")
            .to_string();
        let recommends = parsed["recommends_match"]
            .as_bool()
            .unwrap_or(score >= 0.65);

        Ok((score, notes, recommends))
    }
}
