# 📟 Jupiter API Reference

All requests should be sent to the base URL: `http://localhost:8080/v1`

---

## 🔐 Authentication
Protected routes require a `Authorization: Bearer <token>` header.

### `POST /auth/register`
Create a new account.
- **Body**: `{ username, email, password, display_name? }`

### `POST /auth/login`
Authenticate and receive a JWT.
- **Body**: `{ username, password }`

### `GET /auth/profile`
Retrieve current user public info.

---

## 🤖 AI Agent
Interact with your personal matchmaking agent.

### `GET /chat`
Get conversation history with your agent.

### `POST /chat`
Send a message to your agent.
- **Body**: `{ content }`
- **Response**: `{ user_message, agent_message }`

### `GET /agent/profile`
View the profile data your agent has synthesized about you.

### `POST /agent/profile/update`
Manually trigger the agent to re-analyze your recent history and update your profile.

---

## 💖 Matchmaking

### `GET /matches`
List all matches (pending and confirmed).

### `POST /matching/trigger`
Trigger the background process where your agent evaluates new potential matches.

---

## 🔔 Notifications

### `GET /notifications`
Fetch recent notifications (match proposals, confirmations).

### `GET /notifications/unread`
Get the count of unread notifications.

### `POST /notifications/{id}/read`
Mark a specific notification as read.

---

## 💬 Direct Messages
Communicate with your confirmed matches.

### `GET /messages/{match_id}`
Fetch history with a specific match.

### `POST /messages/{match_id}`
Send a direct message.
- **Body**: `{ content }`
