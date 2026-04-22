<script lang="ts">
	import { api, type Favorite, type HistoryEntry, type QueryResult } from '$lib/api';
	import ChatPanel from '$lib/chat/ChatPanel.svelte';
	import { tabs as tabsStore } from '$lib/stores/tabs.svelte';
	import RowGrid from './RowGrid.svelte';
	import SqlEditor from './SqlEditor.svelte';

	type Props = {
		tabId: string;
		connectionId: string;
		database: string | null;
	};
	let { tabId, connectionId, database }: Props = $props();

	const tab = $derived(tabsStore.tabs.find((t) => t.id === tabId));
	const queryTabs = $derived(tab?.queryTabs ?? []);
	const activeQueryId = $derived(tab?.activeQueryTabId ?? null);
	const activeQuery = $derived(queryTabs.find((q) => q.id === activeQueryId) ?? null);
	const sql = $derived(activeQuery?.sql ?? '');

	// Per-query volatile state
	let resultsById = $state<Record<string, QueryResult | null>>({});
	let errorsById = $state<Record<string, string | null>>({});
	let runningById = $state<Record<string, boolean>>({});

	const result = $derived(activeQueryId ? (resultsById[activeQueryId] ?? null) : null);
	const error = $derived(activeQueryId ? (errorsById[activeQueryId] ?? null) : null);
	const running = $derived(activeQueryId ? (runningById[activeQueryId] ?? false) : false);

	let history = $state<HistoryEntry[]>([]);
	let favorites = $state<Favorite[]>([]);

	function msg(e: unknown): string {
		return e instanceof Error ? e.message : String(e);
	}

	async function run() {
		const qid = activeQueryId;
		if (!qid || !activeQuery) return;
		const trimmed = activeQuery.sql.trim();
		if (!trimmed || runningById[qid]) return;
		runningById[qid] = true;
		errorsById[qid] = null;
		resultsById[qid] = null;
		try {
			resultsById[qid] = await api.queries.run(connectionId, trimmed, database ?? undefined);
		} catch (e) {
			errorsById[qid] = msg(e);
		} finally {
			runningById[qid] = false;
			loadHistory();
		}
	}

	async function loadHistory() {
		try {
			history = await api.history.list(connectionId, 50);
		} catch {
			/* ignore */
		}
	}

	async function loadFavorites() {
		try {
			favorites = await api.favorites.list(connectionId);
		} catch {
			/* ignore */
		}
	}

	async function addFavorite() {
		if (!activeQuery || !activeQuery.sql.trim()) return;
		const defaultName = `query ${favorites.length + 1}`;
		const name = prompt('Favorite name', defaultName)?.trim();
		if (!name) return;
		try {
			await api.favorites.create(connectionId, { name, sql: activeQuery.sql });
			await loadFavorites();
		} catch (e) {
			alert(`Failed to save favorite: ${msg(e)}`);
		}
	}

	async function renameFavorite(f: Favorite, e: Event) {
		e.stopPropagation();
		const name = prompt('New name', f.name)?.trim();
		if (!name || name === f.name) return;
		try {
			await api.favorites.update(connectionId, f.id, { name });
			await loadFavorites();
		} catch (err) {
			alert(`Rename failed: ${msg(err)}`);
		}
	}

	async function deleteFavorite(f: Favorite, e: Event) {
		e.stopPropagation();
		if (!confirm(`Delete favorite "${f.name}"?`)) return;
		try {
			await api.favorites.delete(connectionId, f.id);
			favorites = favorites.filter((x) => x.id !== f.id);
		} catch (err) {
			alert(`Delete failed: ${msg(err)}`);
		}
	}

	function loadFavorite(f: Favorite) {
		setSql(f.sql);
	}

	function setSql(value: string) {
		if (!activeQueryId) return;
		tabsStore.updateQuerySql(tabId, activeQueryId, value);
	}

	function loadFromHistory(h: HistoryEntry) {
		setSql(h.sql);
	}

	async function deleteHistory(h: HistoryEntry, e: Event) {
		e.stopPropagation();
		try {
			await api.history.delete(connectionId, h.id);
			history = history.filter((x) => x.id !== h.id);
		} catch {
			/* ignore */
		}
	}

	// Cleanup volatile state for closed query subtabs
	$effect(() => {
		const ids = new Set(queryTabs.map((q) => q.id));
		for (const k of Object.keys(resultsById)) {
			if (!ids.has(k)) {
				delete resultsById[k];
				delete errorsById[k];
				delete runningById[k];
			}
		}
	});

	$effect(() => {
		connectionId;
		loadHistory();
		loadFavorites();
	});
</script>

<div class="flex h-full overflow-hidden">
	<div class="flex flex-1 flex-col overflow-hidden">
		<!-- editor (active query) with floating Run -->
		<div
			class="relative h-48 shrink-0 border-b border-rule"
			onkeydown={(e) => {
				if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
					e.preventDefault();
					run();
				}
			}}
			role="presentation"
		>
			{#key activeQueryId}
				<SqlEditor value={sql} onChange={setSql} onSubmit={run} />
			{/key}
			<button
				class="absolute right-3 bottom-3 cursor-pointer rounded-md bg-ink px-3.5 py-1.5 font-mono text-[10.5px] tracking-[0.18em] text-cream uppercase shadow-[0_4px_12px_-4px_rgba(26,24,20,0.35)] transition-colors hover:bg-rust disabled:cursor-not-allowed disabled:opacity-40"
				onclick={run}
				disabled={running || !sql.trim()}
				title="⌘⏎ to run"
			>
				{running ? '…' : 'run'}
			</button>
			<span
				class="pointer-events-none absolute right-[88px] bottom-[14px] font-mono text-[9px] tracking-widest text-ink-ghost uppercase select-none"
			>
				⌘⏎
			</span>
		</div>

		<!-- result (active query) -->
		<div class="flex-1 overflow-auto bg-cream">
			{#if result}
				<div
					class="flex items-center justify-end border-b border-rule bg-cream-soft/40 px-4 py-1 font-mono text-[10px] tracking-widest text-ink-faint uppercase"
				>
					{#if result.kind === 'rows'}
						{result.returned} row{result.returned === 1 ? '' : 's'}
					{:else}
						{result.rows_affected} affected
					{/if}
					· {result.duration_ms}ms
				</div>
			{/if}
			{#if error}
				<pre
					class="m-3 rounded bg-crimson-soft p-3 font-mono text-[12px] whitespace-pre-wrap text-crimson">{error}</pre>
			{:else if result}
				{#if result.kind === 'rows'}
					<RowGrid columns={result.columns} rows={result.rows} empty="(query returned no rows)" />
				{:else}
					<div
						class="m-3 rounded border border-moss/30 bg-moss-soft p-3 font-mono text-[12px] text-moss"
					>
						{result.rows_affected} row{result.rows_affected === 1 ? '' : 's'} affected
						{#if result.last_insert_id !== 0}
							· last_insert_id = {result.last_insert_id}
						{/if}
					</div>
				{/if}
			{:else if !running}
				<div class="px-5 py-4 font-mono text-[11px] text-ink-faint italic">
					write a query and press ⌘⏎
				</div>
			{/if}
		</div>
	</div>

	<aside class="flex w-[580px] shrink-0 border-l border-rule">
		<ChatPanel {connectionId} {database} {tabId} onUseSql={setSql} />
		<div class="flex w-[240px] shrink-0 flex-col border-l border-rule bg-cream-soft">
			<!-- favorites -->
			<section class="flex max-h-[40%] flex-col overflow-hidden border-b border-rule">
			<header class="flex items-center justify-between border-b border-rule px-4 py-3">
				<span class="font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">
					Favorites
				</span>
				<button
					class="cursor-pointer text-[14px] leading-none text-ink-faint hover:text-rust disabled:opacity-30"
					title={activeQuery && activeQuery.sql.trim()
						? 'save current query as favorite'
						: 'write a query first'}
					onclick={addFavorite}
					disabled={!activeQuery?.sql.trim()}
					aria-label="add favorite"
				>
					+
				</button>
			</header>
			<div class="flex-1 overflow-auto">
				{#if favorites.length === 0}
					<div class="px-4 py-3 font-mono text-[11px] text-ink-faint italic">
						(none yet — write a query and click +)
					</div>
				{:else}
					{#each favorites as f (f.id)}
						<div class="group/fav relative border-b border-rule/60 hover:bg-cream">
							<button
								class="block w-full cursor-pointer px-4 py-2 pr-14 text-left"
								onclick={() => loadFavorite(f)}
								title="click to load into editor"
							>
								<div class="flex items-center gap-1.5 text-[12px] text-ink">
									<span class="text-rust">★</span>
									<span class="truncate font-medium">{f.name}</span>
								</div>
								<div class="mt-0.5 truncate font-mono text-[11px] text-ink-faint">{f.sql}</div>
							</button>
							<div class="absolute top-2 right-2 flex items-center gap-1 opacity-0 transition-opacity group-hover/fav:opacity-100">
								<button
									class="cursor-pointer rounded px-1 text-[10px] text-ink-faint hover:bg-cream-deep hover:text-ink"
									onclick={(e) => renameFavorite(f, e)}
									title="rename"
								>
									rename
								</button>
								<button
									class="cursor-pointer rounded px-1 text-[14px] leading-none text-ink-faint hover:bg-crimson-soft hover:text-crimson"
									onclick={(e) => deleteFavorite(f, e)}
									title="delete"
									aria-label="delete favorite"
								>
									×
								</button>
							</div>
						</div>
					{/each}
				{/if}
			</div>
		</section>

		<!-- history -->
		<header class="flex items-center justify-between border-b border-rule px-4 py-3">
			<span class="font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">
				History
			</span>
			{#if history.length > 0}
				<button
					class="cursor-pointer font-mono text-[10px] tracking-widest text-ink-faint uppercase hover:text-crimson"
					title="clear history for this connection"
					onclick={async () => {
						if (!confirm('Clear all history for this connection?')) return;
						await api.history.clear(connectionId);
						history = [];
					}}
				>
					clear
				</button>
			{/if}
		</header>
		<div class="flex-1 overflow-auto">
			{#if history.length === 0}
				<div class="px-4 py-3 font-mono text-[11px] text-ink-faint italic">
					(no queries yet)
				</div>
			{:else}
				{#each history as h (h.id)}
					<div class="group/row relative border-b border-rule/60 hover:bg-cream">
						<button
							class="block w-full cursor-pointer px-4 py-2.5 pr-8 text-left"
							onclick={() => loadFromHistory(h)}
							title="click to load into editor"
						>
							<div
								class="flex items-center gap-2 font-mono text-[10px] tracking-widest text-ink-faint uppercase"
							>
								<span class={h.success ? 'text-moss' : 'text-crimson'}>
									{h.success ? '✓' : '✗'}
								</span>
								<span>{h.duration_ms}ms</span>
								{#if h.rows_returned !== null}
									<span>· {h.rows_returned} row{h.rows_returned === 1 ? '' : 's'}</span>
								{:else if h.rows_affected !== null}
									<span>· {h.rows_affected} affected</span>
								{/if}
							</div>
							<div class="mt-0.5 truncate font-mono text-[11.5px] text-ink">{h.sql}</div>
						</button>
						<button
							class="absolute top-2.5 right-2 cursor-pointer rounded px-1 text-[14px] leading-none text-ink-faint opacity-0 transition-opacity group-hover/row:opacity-100 hover:bg-crimson-soft hover:text-crimson"
							onclick={(e) => deleteHistory(h, e)}
							title="delete"
							aria-label="delete history entry"
						>
							×
						</button>
					</div>
				{/each}
			{/if}
		</div>
		</div>
	</aside>
</div>
