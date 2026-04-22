export type ChatMessage = {
	id: string;
	role: 'user' | 'assistant';
	content: string;
	streaming?: boolean;
	error?: string;
};

export type ChatState = {
	messages: ChatMessage[];
	sessionId: string | null;
	streaming: boolean;
	abort: AbortController | null;
};

function emptyState(): ChatState {
	return {
		messages: [],
		sessionId: null,
		streaming: false,
		abort: null
	};
}

class ChatStore {
	private states = $state<Record<string, ChatState>>({});

	/** Read-only lookup — safe to call inside `$derived`. Returns undefined
	 * until a writer (or `ensure`) has seeded the tab. */
	get(tabId: string): ChatState | undefined {
		return this.states[tabId];
	}

	/** Seed a blank state for `tabId` if none exists. Never call this from
	 * inside `$derived` — mutation during derivation triggers
	 * `state_unsafe_mutation`. Use an `$effect.pre` in the consumer. */
	ensure(tabId: string): ChatState {
		let s = this.states[tabId];
		if (!s) {
			s = emptyState();
			this.states[tabId] = s;
		}
		return s;
	}

	append(tabId: string, message: ChatMessage) {
		this.ensure(tabId).messages.push(message);
	}

	patchLast(tabId: string, patch: Partial<ChatMessage>) {
		const s = this.states[tabId];
		if (!s) return;
		const last = s.messages[s.messages.length - 1];
		if (!last) return;
		Object.assign(last, patch);
	}

	appendToLast(tabId: string, text: string) {
		const s = this.states[tabId];
		if (!s) return;
		const last = s.messages[s.messages.length - 1];
		if (!last) return;
		last.content += text;
	}

	setSession(tabId: string, sessionId: string | null) {
		this.ensure(tabId).sessionId = sessionId;
	}

	setStreaming(tabId: string, streaming: boolean, abort: AbortController | null) {
		const s = this.ensure(tabId);
		s.streaming = streaming;
		s.abort = abort;
	}

	clear(tabId: string) {
		this.states[tabId]?.abort?.abort();
		this.states[tabId] = emptyState();
	}

	dropTab(tabId: string) {
		this.states[tabId]?.abort?.abort();
		delete this.states[tabId];
	}
}

export const chat = new ChatStore();
