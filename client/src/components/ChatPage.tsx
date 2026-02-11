import { Send } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "../api";

interface Message {
	id?: number;
	role: string;
	content: string;
	created_at?: string;
}

export default function ChatPage() {
	const [messages, setMessages] = useState<Message[]>([]);
	const [input, setInput] = useState("");
	const [loading, setLoading] = useState(false);
	const [sending, setSending] = useState(false);
	const messagesEndRef = useRef<HTMLDivElement>(null);
	const textareaRef = useRef<HTMLTextAreaElement>(null);

	const loadHistory = useCallback(async () => {
		setLoading(true);
		try {
			const history = await api.getChatHistory();
			setMessages(history);
		} catch (err) {
			console.error("Failed to load chat history:", err);
		} finally {
			setLoading(false);
		}
	}, []);

	const scrollToBottom = useCallback(() => {
		messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
	}, []);

	useEffect(() => {
		loadHistory();
	}, [loadHistory]);

	useEffect(() => {
		scrollToBottom();
	}, [messages, scrollToBottom]);

	const handleSend = async () => {
		const content = input.trim();
		if (!content || sending) return;

		setInput("");
		setSending(true);

		// Optimistic add user message
		const userMsg: Message = {
			role: "user",
			content,
			created_at: new Date().toISOString(),
		};
		setMessages((prev) => [...prev, userMsg]);

		try {
			const response = await api.sendMessage(content);
			setMessages((prev) => [
				...prev.slice(0, -1), // remove optimistic
				response.user_message,
				response.agent_message,
			]);
		} catch (err: any) {
			setMessages((prev) => [
				...prev,
				{
					role: "assistant",
					content: `Sorry, something went wrong: ${err.message}`,
				},
			]);
		} finally {
			setSending(false);
			textareaRef.current?.focus();
		}
	};

	const handleKeyDown = (e: React.KeyboardEvent) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	};

	const formatTime = (dateStr?: string) => {
		if (!dateStr) return "";
		try {
			const d = new Date(dateStr);
			return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
		} catch {
			return "";
		}
	};

	if (loading) {
		return (
			<div className="chat-container">
				<div className="loading-center">
					<div className="spinner" />
				</div>
			</div>
		);
	}

	return (
		<div className="chat-container">
			<div className="chat-messages">
				{messages.length === 0 && (
					<div className="empty-state">
						<div className="empty-icon">🪐</div>
						<h3>Welcome to Jupiter!</h3>
						<p>
							Say hello to your personal AI agent. It will get to know you,
							learn about your interests, and help find your perfect match.
						</p>
					</div>
				)}

				{messages.map((msg, i) => (
					<div key={i} className={`chat-message ${msg.role}`}>
						<div className="avatar">
							{msg.role === "assistant" ? "🪐" : "👤"}
						</div>
						<div>
							<div className="bubble">{msg.content}</div>
							<div className="time">{formatTime(msg.created_at)}</div>
						</div>
					</div>
				))}

				{sending && (
					<div className="chat-message assistant">
						<div className="avatar">🪐</div>
						<div className="bubble">
							<div className="typing-indicator">
								<span />
								<span />
								<span />
							</div>
						</div>
					</div>
				)}

				<div ref={messagesEndRef} />
			</div>

			<div className="chat-input-area">
				<div className="chat-input-wrapper">
					<textarea
						ref={textareaRef}
						value={input}
						onChange={(e) => setInput(e.target.value)}
						onKeyDown={handleKeyDown}
						placeholder="Tell your agent about yourself..."
						rows={1}
						disabled={sending}
					/>
					<button
						type="button"
						className="send-btn"
						onClick={handleSend}
						disabled={!input.trim() || sending}
						title="Send message"
					>
						<Send size={20} />
					</button>
				</div>
			</div>
		</div>
	);
}
