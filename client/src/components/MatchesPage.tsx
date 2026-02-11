import { Heart, MessageCircle, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import * as api from "../api";

interface Match {
	id: number;
	user_a_id: string;
	user_b_id: string;
	agent_a_approves: boolean;
	agent_b_approves: boolean;
	is_matched: boolean;
	created_at: string;
	updated_at: string;
	other_user?: {
		id: string;
		username: string;
		display_name: string;
		bio: string;
	};
}

interface MatchesPageProps {
	onOpenDM: (matchId: number, userName: string) => void;
}

export default function MatchesPage({ onOpenDM }: MatchesPageProps) {
	const [matches, setMatches] = useState<Match[]>([]);
	const [loading, setLoading] = useState(true);
	const [matchingInProgress, setMatchingInProgress] = useState(false);
	const [matchingResult, setMatchingResult] = useState<string | null>(null);

	useEffect(() => {
		loadMatches();
	}, []);

	const loadMatches = async () => {
		try {
			const data = await api.getMatches();
			setMatches(data);
		} catch (err) {
			console.error("Failed to load matches:", err);
		} finally {
			setLoading(false);
		}
	};

	const handleTriggerMatching = async () => {
		setMatchingInProgress(true);
		setMatchingResult(null);
		try {
			const result = await api.triggerMatching();
			setMatchingResult(
				`Evaluated ${result.evaluated} users — ${result.new_recommendations} new recommendations, ${result.new_matches} new matches!`,
			);
			await loadMatches();
		} catch (err: any) {
			setMatchingResult(err.message || "Matching failed");
		} finally {
			setMatchingInProgress(false);
		}
	};

	const confirmedMatches = matches.filter((m) => m.is_matched);
	const pendingMatches = matches.filter((m) => !m.is_matched);

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
				<div
					style={{
						display: "flex",
						justifyContent: "space-between",
						alignItems: "center",
					}}
				>
					<div>
						<h2>💫 Matches</h2>
						<p>Your agent finds compatible people by talking to their agents</p>
					</div>
					<button
						type="button"
						className="btn btn-primary"
						onClick={handleTriggerMatching}
						disabled={matchingInProgress}
						style={{ width: "auto", padding: "10px 20px" }}
					>
						<RefreshCw
							size={16}
							className={matchingInProgress ? "spinning" : ""}
						/>
						{matchingInProgress ? "Agents talking..." : "Find Matches"}
					</button>
				</div>
				{matchingResult && (
					<div
						style={{
							marginTop: 12,
							padding: "10px 14px",
							background: "rgba(124, 92, 252, 0.1)",
							border: "1px solid rgba(124, 92, 252, 0.2)",
							borderRadius: 8,
							fontSize: 13,
							color: "#c084fc",
						}}
					>
						{matchingResult}
					</div>
				)}
			</div>

			<div className="page-body">
				{confirmedMatches.length > 0 && (
					<div className="profile-section">
						<h3>
							<Heart size={16} /> Confirmed Matches
						</h3>
						<div className="card-grid">
							{confirmedMatches.map((m) => (
								<div key={m.id} className="match-card matched">
									<div className="match-header">
										<div className="match-avatar">
											{(m.other_user?.display_name || "?")[0].toUpperCase()}
										</div>
										<div className="match-info">
											<h3>
												{m.other_user?.display_name ||
													m.other_user?.username ||
													"Unknown"}
											</h3>
											<div className="match-status confirmed">
												<span>✓</span> Both agents agree — it's a match!
											</div>
										</div>
									</div>
									{m.other_user?.bio && (
										<p
											style={{
												fontSize: 13,
												color: "var(--text-secondary)",
												lineHeight: 1.5,
											}}
										>
											{m.other_user.bio}
										</p>
									)}
									<div className="match-actions">
										<button
											type="button"
											className="btn btn-success"
											onClick={() =>
												onOpenDM(
													m.id,
													m.other_user?.display_name ||
														m.other_user?.username ||
														"Match",
												)
											}
										>
											<MessageCircle size={16} /> Chat
										</button>
									</div>
								</div>
							))}
						</div>
					</div>
				)}

				{pendingMatches.length > 0 && (
					<div className="profile-section">
						<h3>⏳ Pending</h3>
						<div className="card-grid">
							{pendingMatches.map((m) => (
								<div key={m.id} className="match-card">
									<div className="match-header">
										<div className="match-avatar" style={{ opacity: 0.7 }}>
											{(m.other_user?.display_name || "?")[0].toUpperCase()}
										</div>
										<div className="match-info">
											<h3>
												{m.other_user?.display_name ||
													m.other_user?.username ||
													"Unknown"}
											</h3>
											<div className="match-status pending">
												<span>⏳</span> Waiting for their agent to review...
											</div>
										</div>
									</div>
								</div>
							))}
						</div>
					</div>
				)}

				{matches.length === 0 && (
					<div className="empty-state">
						<div className="empty-icon">💫</div>
						<h3>No matches yet</h3>
						<p>
							Chat with your agent first so it can learn about you. Then hit
							"Find Matches" and let the agents do their magic!
						</p>
					</div>
				)}
			</div>
		</div>
	);
}
