import { Bell, Brain, Heart, LogOut, MessageSquare } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import * as api from "./api";
import AgentProfilePage from "./components/AgentProfilePage";
import AuthPage from "./components/AuthPage";
import ChatPage from "./components/ChatPage";
import DirectMessages from "./components/DirectMessages";
import MatchesPage from "./components/MatchesPage";
import NotificationsPage from "./components/NotificationsPage";

type Page = "chat" | "matches" | "notifications" | "agent" | "dm";

export default function App() {
	const [loggedIn, setLoggedIn] = useState(api.isLoggedIn());
	const [currentPage, setCurrentPage] = useState<Page>("chat");
	const [unreadCount, setUnreadCount] = useState(0);
	const [dmMatchId, setDmMatchId] = useState<number | null>(null);
	const [dmMatchName, setDmMatchName] = useState("");
	const user = api.getCurrentUser();

	const fetchUnread = useCallback(async () => {
		try {
			const data = await api.getUnreadCount();
			setUnreadCount(data.count);
		} catch {
			// ignore
		}
	}, []);

	useEffect(() => {
		if (loggedIn) {
			fetchUnread();
			const interval = setInterval(() => {
				fetchUnread();
			}, 10000);
			return () => clearInterval(interval);
		}
	}, [loggedIn, fetchUnread]);

	const handleLogin = () => {
		setLoggedIn(true);
	};

	const handleLogout = () => {
		api.logout();
		setLoggedIn(false);
	};

	const handleOpenDM = (matchId: number, userName: string) => {
		setDmMatchId(matchId);
		setDmMatchName(userName);
		setCurrentPage("dm");
	};

	const handleBackFromDM = () => {
		setCurrentPage("matches");
		setDmMatchId(null);
	};

	if (!loggedIn) {
		return <AuthPage onLogin={handleLogin} />;
	}

	const navItems = [
		{
			id: "chat" as Page,
			label: "Chat with Agent",
			icon: <MessageSquare size={18} />,
		},
		{ id: "matches" as Page, label: "Matches", icon: <Heart size={18} /> },
		{
			id: "notifications" as Page,
			label: "Notifications",
			icon: <Bell size={18} />,
			badge: unreadCount > 0 ? unreadCount : undefined,
		},
		{
			id: "agent" as Page,
			label: "Agent Knowledge",
			icon: <Brain size={18} />,
		},
	];

	const renderPage = () => {
		switch (currentPage) {
			case "chat":
				return <ChatPage />;
			case "matches":
				return <MatchesPage onOpenDM={handleOpenDM} />;
			case "notifications":
				return <NotificationsPage />;
			case "agent":
				return <AgentProfilePage />;
			case "dm":
				if (dmMatchId !== null) {
					return (
						<DirectMessages
							matchId={dmMatchId}
							matchName={dmMatchName}
							currentUserId={user?.id || ""}
							onBack={handleBackFromDM}
						/>
					);
				}
				return <MatchesPage onOpenDM={handleOpenDM} />;
			default:
				return <ChatPage />;
		}
	};

	return (
		<div className="app">
			<aside className="sidebar">
				<div className="sidebar-brand">
					<span className="logo">🪐</span>
					<h1>Jupiter</h1>
				</div>

				<nav className="sidebar-nav">
					{navItems.map((item) => (
						<button
							key={item.id}
							type="button"
							className={`nav-item ${currentPage === item.id ? "active" : ""}`}
							onClick={() => setCurrentPage(item.id)}
						>
							{item.icon}
							{item.label}
							{item.badge && <span className="badge">{item.badge}</span>}
						</button>
					))}
				</nav>

				<div className="sidebar-footer">
					<div className="user-info">
						<div className="user-avatar">
							{(user?.display_name || user?.username || "?")[0].toUpperCase()}
						</div>
						<div className="user-details">
							<div className="name">{user?.display_name || user?.username}</div>
							<div className="status">Online</div>
						</div>
						<button
							className="logout-btn"
							type="button"
							onClick={handleLogout}
							title="Sign out"
						>
							<LogOut size={16} />
						</button>
					</div>
				</div>
			</aside>

			<main className="main-content">{renderPage()}</main>
		</div>
	);
}
