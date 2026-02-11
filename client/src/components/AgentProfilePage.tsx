import { RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import * as api from "../api";

interface Profile {
	user_id: string;
	personality_summary: string;
	interests: string;
	core_values: string;
	communication_style: string;
	looking_for: string;
	deal_breakers: string;
	raw_notes: string;
	updated_at: string;
}

export default function AgentProfilePage() {
	const [profile, setProfile] = useState<Profile | null>(null);
	const [loading, setLoading] = useState(true);
	const [updating, setUpdating] = useState(false);

	useEffect(() => {
		loadProfile();
	}, []);

	const loadProfile = async () => {
		try {
			const data = await api.getAgentProfile();
			setProfile(data);
		} catch (err) {
			console.error("Failed to load agent profile:", err);
		} finally {
			setLoading(false);
		}
	};

	const handleUpdate = async () => {
		setUpdating(true);
		try {
			await api.triggerProfileUpdate();
			// Wait a bit for the background update
			setTimeout(async () => {
				await loadProfile();
				setUpdating(false);
			}, 3000);
		} catch (err) {
			console.error("Failed to trigger update:", err);
			setUpdating(false);
		}
	};

	const fields = [
		{ key: "personality_summary", label: "🧠 Personality", icon: "🧠" },
		{ key: "interests", label: "✨ Interests", icon: "✨" },
		{ key: "core_values", label: "💎 Values", icon: "💎" },
		{ key: "communication_style", label: "💬 Communication Style", icon: "💬" },
		{ key: "looking_for", label: "❤️ Looking For", icon: "❤️" },
		{ key: "deal_breakers", label: "🚫 Deal Breakers", icon: "🚫" },
		{ key: "raw_notes", label: "📝 Additional Notes", icon: "📝" },
	];

	if (loading) {
		return (
			<div className="loading-center">
				<div className="spinner" />
			</div>
		);
	}

	const isEmpty = profile && !profile.personality_summary && !profile.interests;

	return (
		<div>
			<div className="page-header">
				<div
					style={{
						display: "flex",
						justifyContent: "space-between",
						alignItems: "center",
					}}
				>
					<div>
						<h2>🧠 Agent Knowledge</h2>
						<p>What your AI agent has learned about you</p>
					</div>
					<button
						type="button"
						className="btn btn-secondary"
						onClick={handleUpdate}
						disabled={updating}
						style={{ display: "flex", alignItems: "center", gap: 8 }}
					>
						<RefreshCw size={14} className={updating ? "spinning" : ""} />
						{updating ? "Analyzing..." : "Refresh Knowledge"}
					</button>
				</div>
			</div>
			<div className="page-body">
				{isEmpty ? (
					<div className="empty-state">
						<div className="empty-icon">🧠</div>
						<h3>Your agent is still learning</h3>
						<p>
							Chat with your agent to help it understand you better. The more
							you share, the better matches it can find!
						</p>
					</div>
				) : (
					<div className="profile-section">
						{fields.map(({ key, label }) => {
							const value = (profile as any)?.[key] || "";
							return (
								<div key={key} className="profile-field">
									<div className="field-label">{label}</div>
									<div className={`field-value ${value ? "" : "empty"}`}>
										{value || "Not yet discovered — keep chatting!"}
									</div>
								</div>
							);
						})}
						{profile?.updated_at && (
							<div
								style={{
									fontSize: 11,
									color: "var(--text-muted)",
									marginTop: 16,
									textAlign: "right",
								}}
							>
								Last updated: {new Date(profile.updated_at).toLocaleString()}
							</div>
						)}
					</div>
				)}
			</div>
		</div>
	);
}
