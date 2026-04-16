import { api, type Connection } from '$lib/api';

export type View = 'browse' | 'query';

export type Tab = {
	id: string;
	connectionId: string;
	label: string;
	color: string | null;
	// per-tab UI state, persisted with the tab so refresh restores it
	db: string | null;
	table: string | null;
	view: View;
	sql: string;
};

const STORAGE_KEY = 'dbnyan.tabs.v1';

type Persisted = { tabs: Tab[]; activeId: string | null };

function emptyTab(c: Connection): Tab {
	return {
		id: crypto.randomUUID(),
		connectionId: c.id,
		label: c.name,
		color: c.color ?? null,
		db: c.database ?? null,
		table: null,
		view: 'browse',
		sql: ''
	};
}

class TabsStore {
	tabs = $state<Tab[]>([]);
	activeId = $state<string | null>(null);
	private loaded = false;

	/** Load persisted state from localStorage. Safe to call multiple times. */
	load() {
		if (this.loaded || typeof localStorage === 'undefined') return;
		this.loaded = true;
		try {
			const raw = localStorage.getItem(STORAGE_KEY);
			if (!raw) return;
			const parsed: Persisted = JSON.parse(raw);
			this.tabs = (parsed.tabs ?? []).map((t) => ({
				id: t.id,
				connectionId: t.connectionId,
				label: t.label,
				color: t.color ?? null,
				db: t.db ?? null,
				table: t.table ?? null,
				view: t.view ?? 'browse',
				sql: t.sql ?? ''
			}));
			this.activeId = parsed.activeId ?? this.tabs[0]?.id ?? null;
		} catch {
			/* ignore */
		}
	}

	private save() {
		if (typeof localStorage === 'undefined') return;
		const data: Persisted = { tabs: this.tabs, activeId: this.activeId };
		try {
			localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
		} catch {
			/* ignore */
		}
	}

	open(c: Connection) {
		const existing = this.tabs.find((t) => t.connectionId === c.id);
		if (existing) {
			this.activeId = existing.id;
			this.save();
			return;
		}
		const tab = emptyTab(c);
		this.tabs.push(tab);
		this.activeId = tab.id;
		this.save();
	}

	close(tabId: string) {
		const idx = this.tabs.findIndex((t) => t.id === tabId);
		if (idx === -1) return;
		const closing = this.tabs[idx];
		this.tabs.splice(idx, 1);
		if (this.activeId === tabId) {
			this.activeId = this.tabs[idx]?.id ?? this.tabs[idx - 1]?.id ?? null;
		}
		this.save();
		const stillInUse = this.tabs.some((t) => t.connectionId === closing.connectionId);
		if (!stillInUse) {
			api.sessions.close(closing.connectionId).catch(() => {
				/* best-effort */
			});
		}
	}

	activate(tabId: string) {
		this.activeId = tabId;
		this.save();
	}

	/** Mutate per-tab UI state by tab id. */
	update(
		tabId: string,
		patch: Partial<Pick<Tab, 'db' | 'table' | 'view' | 'sql' | 'label' | 'color'>>
	) {
		const t = this.tabs.find((x) => x.id === tabId);
		if (!t) return;
		Object.assign(t, patch);
		this.save();
	}

	/** Drop tabs whose backing connection no longer exists; refresh labels + color. */
	syncWithConnections(saved: Connection[]) {
		const byId = new Map(saved.map((c) => [c.id, c]));
		this.tabs = this.tabs.filter((t) => {
			const c = byId.get(t.connectionId);
			if (!c) return false;
			t.label = c.name;
			t.color = c.color ?? null;
			return true;
		});
		if (this.activeId && !this.tabs.find((t) => t.id === this.activeId)) {
			this.activeId = this.tabs.at(-1)?.id ?? null;
		}
		this.save();
	}
}

export const tabs = new TabsStore();
