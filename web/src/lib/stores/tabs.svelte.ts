import { api, type Connection } from '$lib/api';

export type View = 'browse' | 'query';

export type QueryTab = {
	id: string;
	sql: string;
};

export type Tab = {
	id: string;
	connectionId: string;
	label: string;
	color: string | null;
	db: string | null;
	table: string | null;
	view: View;
	queryTabs: QueryTab[];
	activeQueryTabId: string | null;
};

const STORAGE_KEY = 'dbnyan.tabs.v1';

type Persisted = {
	tabs: (Tab & { sql?: string })[];
	activeId: string | null;
};

function newQueryTab(sql = ''): QueryTab {
	return { id: crypto.randomUUID(), sql };
}

function emptyTab(c: Connection): Tab {
	const q = newQueryTab();
	return {
		id: crypto.randomUUID(),
		connectionId: c.id,
		label: c.name,
		color: c.color ?? null,
		db: c.database ?? null,
		table: null,
		view: 'browse',
		queryTabs: [q],
		activeQueryTabId: q.id
	};
}

class TabsStore {
	tabs = $state<Tab[]>([]);
	activeId = $state<string | null>(null);
	private loaded = false;

	load() {
		if (this.loaded || typeof localStorage === 'undefined') return;
		this.loaded = true;
		try {
			const raw = localStorage.getItem(STORAGE_KEY);
			if (!raw) return;
			const parsed: Persisted = JSON.parse(raw);
			this.tabs = (parsed.tabs ?? []).map((t) => {
				let queryTabs = t.queryTabs;
				let activeQueryTabId = t.activeQueryTabId;
				if (!queryTabs || queryTabs.length === 0) {
					// migrate old single-sql tabs
					const q = newQueryTab(t.sql ?? '');
					queryTabs = [q];
					activeQueryTabId = q.id;
				} else if (!activeQueryTabId) {
					activeQueryTabId = queryTabs[0].id;
				}
				return {
					id: t.id,
					connectionId: t.connectionId,
					label: t.label,
					color: t.color ?? null,
					db: t.db ?? null,
					table: t.table ?? null,
					view: t.view ?? 'browse',
					queryTabs,
					activeQueryTabId
				};
			});
			this.activeId = parsed.activeId ?? this.tabs[0]?.id ?? null;
		} catch {
			/* ignore */
		}
	}

	private save() {
		if (typeof localStorage === 'undefined') return;
		const data = { tabs: this.tabs, activeId: this.activeId };
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

	update(
		tabId: string,
		patch: Partial<Pick<Tab, 'db' | 'table' | 'view' | 'label' | 'color'>>
	) {
		const t = this.tabs.find((x) => x.id === tabId);
		if (!t) return;
		Object.assign(t, patch);
		this.save();
	}

	// --- query subtabs ---

	addQueryTab(tabId: string, sql = '') {
		const t = this.tabs.find((x) => x.id === tabId);
		if (!t) return;
		const q = newQueryTab(sql);
		t.queryTabs.push(q);
		t.activeQueryTabId = q.id;
		this.save();
	}

	closeQueryTab(tabId: string, qid: string) {
		const t = this.tabs.find((x) => x.id === tabId);
		if (!t) return;
		if (t.queryTabs.length <= 1) return;
		const idx = t.queryTabs.findIndex((q) => q.id === qid);
		if (idx === -1) return;
		t.queryTabs.splice(idx, 1);
		if (t.activeQueryTabId === qid) {
			t.activeQueryTabId = t.queryTabs[idx]?.id ?? t.queryTabs[idx - 1]?.id ?? null;
		}
		this.save();
	}

	activateQueryTab(tabId: string, qid: string) {
		const t = this.tabs.find((x) => x.id === tabId);
		if (!t) return;
		t.activeQueryTabId = qid;
		this.save();
	}

	updateQuerySql(tabId: string, qid: string, sql: string) {
		const t = this.tabs.find((x) => x.id === tabId);
		if (!t) return;
		const q = t.queryTabs.find((x) => x.id === qid);
		if (!q) return;
		q.sql = sql;
		this.save();
	}

	// --- sync helpers ---

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
