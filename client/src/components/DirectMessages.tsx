import { ArrowLeft, Send } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "../api";

interface DM {
	id: number;
	match_id: number;
	sender_id: string;
	content: string;
	created_at: string;
}

interface DirectMessagesProps {
	matchId: number;
	matchName: string;
	currentUserId: string;
	onBack: () => void;
}

export default function DirectMessages({
	matchId,
	matchName,
	currentUserId,
	onBack,
}: DirectMessagesProps) {
	const [messages, setMessages] = useState<DM[]>([]);
	const [input, setInput] = useState("");
	const [loading, setLoading] = useState(true);
	const [sending, setSending] = useState(false);
	const messagesEndRef = useRef<HTMLDivElement>(null);
	const textareaRef = useRef<HTMLTextAreaElement>(null);

	const loadMessages = useCallback(async () => {
		try {
			const data = await api.getDirectMessages(matchId);
			setMessages(data);
		} catch (err) {
			console.error("Failed to load messages:", err);
		} finally {
			setLoading(false);
		}
	}, [matchId]);

	useEffect(() => {
		loadMessages();
		// Poll for new messages every 3 seconds
		const interval = setInterval(() => {
			loadMessages();
		}, 3000);
		return () => {
			clearInterval(interval);
		};
	}, [loadMessages]);

	useEffect(() => {
		scrollToBottom();
	}, [messages]);

	const scrollToBottom = () => {
		messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
	};

	const handleSend = async () => {
		const content = input.trim();
		if (!content || sending) return;

		setInput("");
		setSending(true);

		try {
			const msg = await api.sendDirectMessage(matchId, content);
			setMessages((prev) => [...prev, msg]);
		} catch (err) {
			console.error("Failed to send message:", err);
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

	const formatTime = (dateStr: string) => {
		try {
			return new Date(dateStr).toLocaleTimeString([], {
				hour: "2-digit",
				minute: "2-digit",
			});
		} catch {
			return "";
		}
	};

	return (
		<div className="dm-container">
			<div className="dm-header">
				<button className="back-btn" onClick={onBack}>
					<ArrowLeft size={20} />
				</button>
				<div
					className="user-avatar"
					style={{ width: 32, height: 32, fontSize: 12 }}
				>
					{matchName[0].toUpperCase()}
				</div>
				<div>
					<div style={{ fontWeight: 600, fontSize: 15 }}>{matchName}</div>
					<div style={{ fontSize: 11, color: "var(--text-muted)" }}>
						Matched
					</div>
				</div>
			</div>

			<div className="chat-messages">
				{loading ? (
					<div className="loading-center">
						<div className="spinner" />
					</div>
				) : messages.length === 0 ? (
					<div className="empty-state">
						<div className="empty-icon">💬</div>
						<h3>Start the conversation!</h3>
						<p>Your agents agreed you'd be great together. Say hello!</p>
					</div>
				) : (
					messages.map((msg) => {
						const isMine = msg.sender_id === currentUserId;
						return (
							<div
								key={msg.id}
								className={`chat-message ${isMine ? "user" : "assistant"}`}
							>
								<div className="avatar">
									{isMine ? "👤" : matchName[0].toUpperCase()}
								</div>
								<div>
									<div className="bubble">{msg.content}</div>
									<div className="time">{formatTime(msg.created_at)}</div>
								</div>
							</div>
						);
					})
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
						placeholder={`Message ${matchName}...`}
						rows={1}
						disabled={sending}
					/>
					<button
						type="button"
						className="send-btn"
						onClick={handleSend}
						disabled={!input.trim() || sending}
					>
						<Send size={20} />
					</button>
				</div>
			</div>
		</div>
	);
}
