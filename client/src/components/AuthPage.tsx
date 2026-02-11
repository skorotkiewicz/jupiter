import { useState } from "react";
import * as api from "../api";

interface AuthPageProps {
	onLogin: () => void;
}

export default function AuthPage({ onLogin }: AuthPageProps) {
	const [isRegister, setIsRegister] = useState(false);
	const [username, setUsername] = useState("");
	const [email, setEmail] = useState("");
	const [password, setPassword] = useState("");
	const [displayName, setDisplayName] = useState("");
	const [error, setError] = useState("");
	const [loading, setLoading] = useState(false);

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError("");
		setLoading(true);

		try {
			if (isRegister) {
				await api.register(username, email, password, displayName || undefined);
			} else {
				await api.login(username, password);
			}
			onLogin();
		} catch (err: any) {
			setError(err.message || "Something went wrong");
		} finally {
			setLoading(false);
		}
	};

	return (
		<div className="auth-page">
			<div className="auth-card">
				<div className="logo-section">
					<span className="planet">🪐</span>
					<h1>Jupiter</h1>
					<p>AI-powered matchmaking — let your agent find your perfect match</p>
				</div>

				{error && <div className="error-message">{error}</div>}

				<form onSubmit={handleSubmit}>
					<div className="form-group">
						<label htmlFor="username">Username</label>
						<input
							id="username"
							type="text"
							value={username}
							onChange={(e) => setUsername(e.target.value)}
							placeholder="Choose a username"
							required
							autoComplete="username"
						/>
					</div>

					{isRegister && (
						<>
							<div className="form-group">
								<label htmlFor="email">Email</label>
								<input
									id="email"
									type="email"
									value={email}
									onChange={(e) => setEmail(e.target.value)}
									placeholder="your@email.com"
									required
									autoComplete="email"
								/>
							</div>
							<div className="form-group">
								<label htmlFor="displayName">Display Name</label>
								<input
									id="displayName"
									type="text"
									value={displayName}
									onChange={(e) => setDisplayName(e.target.value)}
									placeholder="How should we call you?"
									autoComplete="name"
								/>
							</div>
						</>
					)}

					<div className="form-group">
						<label htmlFor="password">Password</label>
						<input
							id="password"
							type="password"
							value={password}
							onChange={(e) => setPassword(e.target.value)}
							placeholder="••••••••"
							required
							autoComplete={isRegister ? "new-password" : "current-password"}
						/>
					</div>

					<button type="submit" className="btn btn-primary" disabled={loading}>
						{loading
							? "Please wait..."
							: isRegister
								? "Create Account"
								: "Sign In"}
					</button>
				</form>

				<div className="auth-footer">
					{isRegister ? (
						<span>
							Already have an account?{" "}
							<a
								onClick={() => {
									setIsRegister(false);
									setError("");
								}}
							>
								Sign in
							</a>
						</span>
					) : (
						<span>
							New to Jupiter?{" "}
							<a
								onClick={() => {
									setIsRegister(true);
									setError("");
								}}
							>
								Create account
							</a>
						</span>
					)}
				</div>
			</div>
		</div>
	);
}
