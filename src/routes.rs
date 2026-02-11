use actix_web::{web, HttpRequest, HttpResponse};
use crate::agent::LlmAgent;
use crate::auth::extract_user_id;
use crate::db::Database;
use crate::models::*;

// ── Chat with personal agent ──

pub async fn get_chat_history(
    req: HttpRequest,
    db: web::Data<Database>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, role, content, created_at FROM conversations WHERE user_id = ?1 ORDER BY created_at ASC")
        .unwrap();

    let messages: Vec<ChatMessage> = stmt
        .query_map(rusqlite::params![&claims.sub], |row| {
            Ok(ChatMessage {
                id: Some(row.get(0)?),
                role: row.get(1)?,
                content: row.get(2)?,
                created_at: Some(row.get(3)?),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(messages)
}

pub async fn send_message(
    req: HttpRequest,
    db: web::Data<Database>,
    agent: web::Data<LlmAgent>,
    body: web::Json<SendMessageRequest>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let user_id = claims.sub.clone();
    let user_content = body.content.trim().to_string();

    if user_content.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Message cannot be empty"}));
    }

    // Get conversation history
    let history = {
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, role, content, created_at FROM conversations WHERE user_id = ?1 ORDER BY created_at ASC LIMIT 50")
            .unwrap();
        stmt.query_map(rusqlite::params![&user_id], |row| {
            Ok(ChatMessage {
                id: Some(row.get(0)?),
                role: row.get(1)?,
                content: row.get(2)?,
                created_at: Some(row.get(3)?),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>()
    };

    // Get agent profile
    let agent_profile = {
        let conn = db.conn.lock().unwrap();
        get_agent_profile_db(&conn, &user_id)
    };

    // Save user message
    {
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO conversations (user_id, role, content) VALUES (?1, 'user', ?2)",
            rusqlite::params![&user_id, &user_content],
        ).unwrap();
    }

    // Get LLM response
    let agent_response = match agent.chat_with_user(&history, &agent_profile, &user_content).await {
        Ok(r) => r,
        Err(e) => {
            log::error!("Agent chat error: {}", e);
            format!("I'm having a moment — could you try again? (Error: {})", e)
        }
    };

    // Save agent response
    {
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO conversations (user_id, role, content) VALUES (?1, 'assistant', ?2)",
            rusqlite::params![&user_id, &agent_response],
        ).unwrap();
    }

    // Trigger profile update in background (every 5 messages)
    let msg_count = history.len() + 2; // +2 for new messages
    if msg_count % 5 == 0 {
        let db_clone = db.clone();
        let agent_clone = agent.clone();
        let uid = user_id.clone();
        tokio::spawn(async move {
            update_user_profile_bg(db_clone, agent_clone, uid).await;
        });
    }

    let user_msg = ChatMessage {
        id: None,
        role: "user".to_string(),
        content: user_content,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    let agent_msg = ChatMessage {
        id: None,
        role: "assistant".to_string(),
        content: agent_response,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    HttpResponse::Ok().json(ChatResponse {
        user_message: user_msg,
        agent_message: agent_msg,
    })
}

async fn update_user_profile_bg(
    db: web::Data<Database>,
    agent: web::Data<LlmAgent>,
    user_id: String,
) {
    let history = {
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, role, content, created_at FROM conversations WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 30")
            .unwrap();
        stmt.query_map(rusqlite::params![&user_id], |row| {
            Ok(ChatMessage {
                id: Some(row.get(0)?),
                role: row.get(1)?,
                content: row.get(2)?,
                created_at: Some(row.get(3)?),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>()
    };

    let current_profile = {
        let conn = db.conn.lock().unwrap();
        get_agent_profile_db(&conn, &user_id)
    };

    match agent.update_user_profile(&history, &current_profile).await {
        Ok(updated) => {
            let conn = db.conn.lock().unwrap();
            let _ = conn.execute(
                "UPDATE agent_profiles SET personality_summary=?1, interests=?2, core_values=?3, communication_style=?4, looking_for=?5, deal_breakers=?6, raw_notes=?7, updated_at=datetime('now') WHERE user_id=?8",
                rusqlite::params![
                    &updated.personality_summary,
                    &updated.interests,
                    &updated.core_values,
                    &updated.communication_style,
                    &updated.looking_for,
                    &updated.deal_breakers,
                    &updated.raw_notes,
                    &user_id,
                ],
            );
            log::info!("Updated profile for user {}", user_id);
        }
        Err(e) => log::error!("Profile update failed for {}: {}", user_id, e),
    }
}

// ── Agent Profile ──

pub async fn get_agent_profile(
    req: HttpRequest,
    db: web::Data<Database>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let conn = db.conn.lock().unwrap();
    let profile = get_agent_profile_db(&conn, &claims.sub);
    HttpResponse::Ok().json(profile)
}

pub async fn trigger_profile_update(
    req: HttpRequest,
    db: web::Data<Database>,
    agent: web::Data<LlmAgent>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let db_clone = db.clone();
    let agent_clone = agent.clone();
    let uid = claims.sub.clone();
    tokio::spawn(async move {
        update_user_profile_bg(db_clone, agent_clone, uid).await;
    });

    HttpResponse::Ok().json(serde_json::json!({"status": "Profile update triggered"}))
}

fn get_agent_profile_db(conn: &rusqlite::Connection, user_id: &str) -> AgentProfile {
    conn.query_row(
        "SELECT user_id, personality_summary, interests, core_values, communication_style, looking_for, deal_breakers, raw_notes, updated_at FROM agent_profiles WHERE user_id = ?1",
        rusqlite::params![user_id],
        |row| {
            Ok(AgentProfile {
                user_id: row.get(0)?,
                personality_summary: row.get(1)?,
                interests: row.get(2)?,
                core_values: row.get(3)?,
                communication_style: row.get(4)?,
                looking_for: row.get(5)?,
                deal_breakers: row.get(6)?,
                raw_notes: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    )
    .unwrap_or_default()
}

// ── Matching Engine ──

pub async fn trigger_matching(
    req: HttpRequest,
    db: web::Data<Database>,
    agent: web::Data<LlmAgent>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let my_user_id = claims.sub.clone();

    // Get my profile
    let my_profile = {
        let conn = db.conn.lock().unwrap();
        get_agent_profile_db(&conn, &my_user_id)
    };

    if my_profile.personality_summary.is_empty() && my_profile.interests.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Your agent doesn't know enough about you yet. Chat more first!"
        }));
    }

    // Get all other users with profiles
    let other_users: Vec<(String, AgentProfile)> = {
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT user_id FROM agent_profiles WHERE user_id != ?1 AND (personality_summary != '' OR interests != '')")
            .unwrap();
        stmt.query_map(rusqlite::params![&my_user_id], |row| {
            let uid: String = row.get(0)?;
            Ok(uid)
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .map(|uid| {
            let profile = get_agent_profile_db(&conn, &uid);
            (uid, profile)
        })
        .collect()
    };

    let mut evaluated = 0;
    let mut new_recommendations = 0;
    let mut new_matches = 0;

    for (other_id, other_profile) in &other_users {
        // Get existing notes
        let existing_notes: Option<AgentPeerNote> = {
            let conn = db.conn.lock().unwrap();
            conn.query_row(
                "SELECT id, agent_user_id, about_user_id, compatibility_score, notes, recommends_match, conversation_count, updated_at FROM agent_peer_notes WHERE agent_user_id = ?1 AND about_user_id = ?2",
                rusqlite::params![&my_user_id, other_id],
                |row| {
                    Ok(AgentPeerNote {
                        id: row.get(0)?,
                        agent_user_id: row.get(1)?,
                        about_user_id: row.get(2)?,
                        compatibility_score: row.get(3)?,
                        notes: row.get(4)?,
                        recommends_match: row.get::<_, i32>(5)? != 0,
                        conversation_count: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .ok()
        };

        // Evaluate compatibility
        match agent
            .evaluate_compatibility(&my_profile, other_profile, existing_notes.as_ref())
            .await
        {
            Ok((score, notes, recommends)) => {
                evaluated += 1;
                let conv_count = existing_notes
                    .as_ref()
                    .map(|n| n.conversation_count + 1)
                    .unwrap_or(1);

                // Save peer notes
                {
                    let conn = db.conn.lock().unwrap();
                    conn.execute(
                        "INSERT INTO agent_peer_notes (agent_user_id, about_user_id, compatibility_score, notes, recommends_match, conversation_count, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
                         ON CONFLICT(agent_user_id, about_user_id) DO UPDATE SET
                         compatibility_score=?3, notes=?4, recommends_match=?5, conversation_count=?6, updated_at=datetime('now')",
                        rusqlite::params![&my_user_id, other_id, score, &notes, recommends as i32, conv_count],
                    ).unwrap();
                }

                if recommends {
                    new_recommendations += 1;

                    // Check if there's already a match record
                    let match_exists: bool = {
                        let conn = db.conn.lock().unwrap();
                        conn.query_row(
                            "SELECT COUNT(*) FROM matches WHERE (user_a_id=?1 AND user_b_id=?2) OR (user_a_id=?2 AND user_b_id=?1)",
                            rusqlite::params![&my_user_id, other_id],
                            |row| row.get::<_, i64>(0),
                        ).unwrap_or(0) > 0
                    };

                    if !match_exists {
                        // Create match proposal
                        let conn = db.conn.lock().unwrap();
                        conn.execute(
                            "INSERT INTO matches (user_a_id, user_b_id, agent_a_approves) VALUES (?1, ?2, 1)",
                            rusqlite::params![&my_user_id, other_id],
                        ).unwrap();

                        // Notify the other user
                        let my_name: String = conn.query_row(
                            "SELECT display_name FROM users WHERE id = ?1",
                            rusqlite::params![&my_user_id],
                            |row| row.get(0),
                        ).unwrap_or_else(|_| "Someone".to_string());

                        conn.execute(
                            "INSERT INTO notifications (user_id, notification_type, title, message, related_user_id) VALUES (?1, 'match_proposal', 'New Match Suggestion!', ?2, ?3)",
                            rusqlite::params![
                                other_id,
                                format!("Your agent has been contacted by {}'s agent. They think you might be a great match! (Compatibility: {:.0}%)", my_name, score * 100.0),
                                &my_user_id,
                            ],
                        ).unwrap();
                    } else {
                        // Check if other agent already approved — if so, it's a mutual match!
                        let conn = db.conn.lock().unwrap();
                        let updated = conn.execute(
                            "UPDATE matches SET agent_a_approves = 1, is_matched = CASE WHEN agent_b_approves = 1 THEN 1 ELSE is_matched END, updated_at = datetime('now') WHERE user_a_id = ?1 AND user_b_id = ?2",
                            rusqlite::params![&my_user_id, other_id],
                        ).unwrap_or(0);

                        if updated == 0 {
                            // We might be user_b
                            conn.execute(
                                "UPDATE matches SET agent_b_approves = 1, is_matched = CASE WHEN agent_a_approves = 1 THEN 1 ELSE is_matched END, updated_at = datetime('now') WHERE user_a_id = ?1 AND user_b_id = ?2",
                                rusqlite::params![other_id, &my_user_id],
                            ).unwrap();
                        }

                        // Check if now matched
                        let is_matched: bool = conn.query_row(
                            "SELECT is_matched FROM matches WHERE (user_a_id=?1 AND user_b_id=?2) OR (user_a_id=?2 AND user_b_id=?1)",
                            rusqlite::params![&my_user_id, other_id],
                            |row| row.get::<_, i32>(0),
                        ).unwrap_or(0) != 0;

                        if is_matched {
                            new_matches += 1;
                            // Notify both users
                            let my_name: String = conn.query_row(
                                "SELECT display_name FROM users WHERE id = ?1",
                                rusqlite::params![&my_user_id],
                                |row| row.get(0),
                            ).unwrap_or_else(|_| "Someone".to_string());

                            let other_name: String = conn.query_row(
                                "SELECT display_name FROM users WHERE id = ?1",
                                rusqlite::params![other_id],
                                |row| row.get(0),
                            ).unwrap_or_else(|_| "Someone".to_string());

                            conn.execute(
                                "INSERT INTO notifications (user_id, notification_type, title, message, related_user_id) VALUES (?1, 'match_confirmed', '🎉 It''s a Match!', ?2, ?3)",
                                rusqlite::params![
                                    &my_user_id,
                                    format!("Both agents agree — you and {} could be amazing together! You can now chat directly.", other_name),
                                    other_id,
                                ],
                            ).unwrap();

                            conn.execute(
                                "INSERT INTO notifications (user_id, notification_type, title, message, related_user_id) VALUES (?1, 'match_confirmed', '🎉 It''s a Match!', ?2, ?3)",
                                rusqlite::params![
                                    other_id,
                                    format!("Both agents agree — you and {} could be amazing together! You can now chat directly.", my_name),
                                    &my_user_id,
                                ],
                            ).unwrap();
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Compatibility eval failed for {} vs {}: {}", my_user_id, other_id, e);
            }
        }
    }

    HttpResponse::Ok().json(MatchingStatus {
        evaluated,
        new_recommendations,
        new_matches,
    })
}

// ── Matches ──

pub async fn get_matches(
    req: HttpRequest,
    db: web::Data<Database>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT m.id, m.user_a_id, m.user_b_id, m.agent_a_approves, m.agent_b_approves, m.is_matched, m.created_at, m.updated_at
             FROM matches m
             WHERE m.user_a_id = ?1 OR m.user_b_id = ?1
             ORDER BY m.is_matched DESC, m.updated_at DESC"
        )
        .unwrap();

    let matches: Vec<MatchRecord> = stmt
        .query_map(rusqlite::params![&claims.sub], |row| {
            let user_a_id: String = row.get(1)?;
            let user_b_id: String = row.get(2)?;
            let other_user_id = if user_a_id == claims.sub { &user_b_id } else { &user_a_id };

            let other_user = conn.query_row(
                "SELECT id, username, email, display_name, bio, created_at FROM users WHERE id = ?1",
                rusqlite::params![other_user_id],
                |r| {
                    Ok(UserPublic {
                        id: r.get(0)?,
                        username: r.get(1)?,
                        email: r.get(2)?,
                        display_name: r.get(3)?,
                        bio: r.get(4)?,
                        created_at: r.get(5)?,
                    })
                },
            ).ok();

            Ok(MatchRecord {
                id: row.get(0)?,
                user_a_id,
                user_b_id,
                agent_a_approves: row.get::<_, i32>(3)? != 0,
                agent_b_approves: row.get::<_, i32>(4)? != 0,
                is_matched: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                other_user,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(matches)
}

// ── Notifications ──

pub async fn get_notifications(
    req: HttpRequest,
    db: web::Data<Database>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let conn = db.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, user_id, notification_type, title, message, related_user_id, is_read, created_at FROM notifications WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 50")
        .unwrap();

    let notifications: Vec<Notification> = stmt
        .query_map(rusqlite::params![&claims.sub], |row| {
            Ok(Notification {
                id: row.get(0)?,
                user_id: row.get(1)?,
                notification_type: row.get(2)?,
                title: row.get(3)?,
                message: row.get(4)?,
                related_user_id: row.get(5)?,
                is_read: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(notifications)
}

pub async fn mark_notification_read(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<i64>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let notification_id = path.into_inner();
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "UPDATE notifications SET is_read = 1 WHERE id = ?1 AND user_id = ?2",
        rusqlite::params![notification_id, &claims.sub],
    ).unwrap();

    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

pub async fn get_unread_count(
    req: HttpRequest,
    db: web::Data<Database>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let conn = db.conn.lock().unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notifications WHERE user_id = ?1 AND is_read = 0",
            rusqlite::params![&claims.sub],
            |row| row.get(0),
        )
        .unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({"count": count}))
}

// ── Direct Messages ──

pub async fn get_direct_messages(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<i64>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let match_id = path.into_inner();

    // Verify user is part of this match and it's confirmed
    let conn = db.conn.lock().unwrap();
    let is_participant: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM matches WHERE id = ?1 AND is_matched = 1 AND (user_a_id = ?2 OR user_b_id = ?2)",
            rusqlite::params![match_id, &claims.sub],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;

    if !is_participant {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Not authorized for this conversation"}));
    }

    let mut stmt = conn
        .prepare("SELECT id, match_id, sender_id, content, created_at FROM direct_messages WHERE match_id = ?1 ORDER BY created_at ASC")
        .unwrap();

    let messages: Vec<DirectMessage> = stmt
        .query_map(rusqlite::params![match_id], |row| {
            Ok(DirectMessage {
                id: row.get(0)?,
                match_id: row.get(1)?,
                sender_id: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(messages)
}

pub async fn send_direct_message(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<i64>,
    body: web::Json<SendDirectMessageRequest>,
) -> HttpResponse {
    let claims = match extract_user_id(&req) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let match_id = path.into_inner();
    let content = body.content.trim().to_string();

    if content.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Message cannot be empty"}));
    }

    let conn = db.conn.lock().unwrap();

    // Verify user is part of this match
    let is_participant: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM matches WHERE id = ?1 AND is_matched = 1 AND (user_a_id = ?2 OR user_b_id = ?2)",
            rusqlite::params![match_id, &claims.sub],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;

    if !is_participant {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": "Not authorized"}));
    }

    conn.execute(
        "INSERT INTO direct_messages (match_id, sender_id, content) VALUES (?1, ?2, ?3)",
        rusqlite::params![match_id, &claims.sub, &content],
    ).unwrap();

    let msg = DirectMessage {
        id: conn.last_insert_rowid(),
        match_id,
        sender_id: claims.sub,
        content,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    HttpResponse::Ok().json(msg)
}
