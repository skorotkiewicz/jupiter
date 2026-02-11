import { Check } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import * as api from "../api";

interface Notification {
	id: number;
	user_id: string;
	notification_type: string;
	title: string;
	message: string;
	related_user_id?: string;
	is_read: boolean;
	created_at: string;
}

export default function NotificationsPage() {
	const [notifications, setNotifications] = useState<Notification[]>([]);
	const [loading, setLoading] = useState(true);

	const loadNotifications = useCallback(async () => {
		try {
			const data = await api.getNotifications();
			setNotifications(data);
		} catch (err) {
			console.error("Failed to load notifications:", err);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		loadNotifications();
	}, [loadNotifications]);

	const handleMarkRead = async (id: number) => {
		try {
			await api.markNotificationRead(id);
			setNotifications((prev) =>
				prev.map((n) => (n.id === id ? { ...n, is_read: true } : n)),
			);
		} catch (err) {
			console.error("Failed to mark as read:", err);
		}
	};

	const formatTime = (dateStr: string) => {
		try {
			const d = new Date(dateStr);
			const now = new Date();
			const diff = now.getTime() - d.getTime();
			const mins = Math.floor(diff / 60000);
			if (mins < 1) return "Just now";
			if (mins < 60) return `${mins}m ago`;
			const hours = Math.floor(mins / 60);
			if (hours < 24) return `${hours}h ago`;
			const days = Math.floor(hours / 24);
			return `${days}d ago`;
		} catch {
			return dateStr;
		}
	};

	if (loading) {
		return (
			<div className="loading-center">
				<div className="spinner" />
			</div>
		);
	}

	return (
		<div>
			<div className="page-header">
				<h2>🔔 Notifications</h2>
				<p>Updates from your agent and matches</p>
			</div>
			<div className="page-body">
				{notifications.length === 0 ? (
					<div className="empty-state">
						<div className="empty-icon">🔔</div>
						<h3>No notifications yet</h3>
						<p>
							When your agent finds potential matches or you get a mutual match,
							you'll see it here.
						</p>
					</div>
				) : (
					notifications.map((n) => (
						<div
							key={n.id}
							className={`notification-item ${n.is_read ? "" : "unread"}`}
							onClick={() => !n.is_read && handleMarkRead(n.id)}
						>
							<div
								className={`notif-icon ${n.notification_type === "match_confirmed" ? "match" : "proposal"}`}
							>
								{n.notification_type === "match_confirmed" ? "🎉" : "💫"}
							</div>
							<div className="notif-content">
								<h4>{n.title}</h4>
								<p>{n.message}</p>
								<div className="notif-time">{formatTime(n.created_at)}</div>
							</div>
							{!n.is_read && (
								<button
									type="button"
									className="btn btn-ghost"
									onClick={(e) => {
										e.stopPropagation();
										handleMarkRead(n.id);
									}}
									title="Mark as read"
								>
									<Check size={16} />
								</button>
							)}
						</div>
					))
				)}
			</div>
		</div>
	);
}
