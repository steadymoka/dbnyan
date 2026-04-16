<script lang="ts">
	import {
		api,
		type ColumnInfo,
		type Connection,
		type ConnectionInput,
		type RowSet,
		type TableInfo
	} from '$lib/api';
	import { tabs } from '$lib/stores/tabs.svelte';
	import { colorHex } from '$lib/colors';
	import ConnectionForm from './ConnectionForm.svelte';
	import RowGrid from './RowGrid.svelte';
	import QueryView from './QueryView.svelte';

	type Props = { tabId: string };
	let { tabId }: Props = $props();

	const tab = $derived(tabs.tabs.find((t) => t.id === tabId));
	const connectionId = $derived(tab?.connectionId ?? '');
	const selectedDb = $derived(tab?.db ?? null);
	const selectedTable = $derived(tab?.table ?? null);
	const view = $derived(tab?.view ?? 'browse');

	let conn = $state<Connection | null>(null);
	let connErr = $state<string | null>(null);

	let dbs = $state<string[]>([]);
	let dbsErr = $state<string | null>(null);
	let dbsLoading = $state(false);

	let tables = $state<TableInfo[] | null>(null);
	let tablesErr = $state<string | null>(null);
	let tablesLoading = $state(false);

	let schema = $state<ColumnInfo[] | null>(null);
	let schemaErr = $state<string | null>(null);

	let rowSet = $state<RowSet | null>(null);
	let rowsErr = $state<string | null>(null);
	let rowsLoading = $state(false);

	$effect(() => {
		const id = connectionId;
		if (!id) return;
		conn = null;
		connErr = null;
		api.connections
			.get(id)
			.then((c) => {
				if (id !== connectionId) return;
				conn = c;
				loadDatabases(id);
			})
			.catch((e) => {
				if (id === connectionId) connErr = msg(e);
			});
	});

	async function loadDatabases(id: string) {
		dbsLoading = true;
		dbsErr = null;
		dbs = [];
		try {
			const list = await api.databases.list(id);
			if (id !== connectionId) return;
			dbs = list;
			if (selectedDb && list.includes(selectedDb)) {
				loadTables(id, selectedDb);
			} else if (selectedDb && !list.includes(selectedDb)) {
				tabs.update(tabId, { db: null, table: null });
			}
		} catch (e) {
			if (id === connectionId) dbsErr = msg(e);
		} finally {
			if (id === connectionId) dbsLoading = false;
		}
	}

	async function loadTables(id: string, db: string) {
		tablesLoading = true;
		tablesErr = null;
		tables = null;
		try {
			const list = await api.databases.tables(id, db);
			if (id !== connectionId || db !== selectedDb) return;
			tables = list;
			if (selectedTable && list.find((t) => t.name === selectedTable)) {
				loadTable(id, db, selectedTable);
			} else if (selectedTable) {
				tabs.update(tabId, { table: null });
			}
		} catch (e) {
			if (id === connectionId && db === selectedDb) tablesErr = msg(e);
		} finally {
			if (id === connectionId && db === selectedDb) tablesLoading = false;
		}
	}

	async function loadTable(id: string, db: string, table: string) {
		schema = null;
		schemaErr = null;
		rowSet = null;
		rowsErr = null;
		rowsLoading = true;
		try {
			const [s, r] = await Promise.all([
				api.databases.schema(id, db, table),
				api.databases.rows(id, db, table, { limit: 200 })
			]);
			if (id !== connectionId || table !== selectedTable) return;
			schema = s;
			rowSet = r;
		} catch (e) {
			if (id === connectionId && table === selectedTable) {
				rowsErr = msg(e);
				schemaErr = msg(e);
			}
		} finally {
			if (id === connectionId && table === selectedTable) rowsLoading = false;
		}
	}

	function selectDb(db: string) {
		if (db === selectedDb) {
			dbMenuOpen = false;
			return;
		}
		tabs.update(tabId, { db, table: null });
		schema = null;
		rowSet = null;
		tables = null;
		dbMenuOpen = false;
		loadTables(connectionId, db);
	}

	function selectTable(name: string) {
		if (name === selectedTable) return;
		tabs.update(tabId, { table: name, view: 'browse' });
		if (selectedDb) loadTable(connectionId, selectedDb, name);
	}

	function setView(v: 'browse' | 'query') {
		tabs.update(tabId, { view: v });
	}

	const queryTabs = $derived(tab?.queryTabs ?? []);
	const activeQueryId = $derived(tab?.activeQueryTabId ?? null);

	function queryLabel(q: { sql: string }, idx: number): string {
		const trimmed = q.sql.trim();
		if (!trimmed) return `Query ${idx + 1}`;
		for (const raw of trimmed.split(/\r?\n/)) {
			const line = raw.trim();
			if (!line) continue;
			if (line.startsWith('--') || line.startsWith('#')) continue;
			return line.length > 22 ? line.slice(0, 22) + '…' : line;
		}
		return `Query ${idx + 1}`;
	}

	function activateQuery(qid: string) {
		if (view !== 'query') tabs.update(tabId, { view: 'query' });
		tabs.activateQueryTab(tabId, qid);
	}

	function addQuery() {
		if (view !== 'query') tabs.update(tabId, { view: 'query' });
		tabs.addQueryTab(tabId);
	}

	function closeQuerySub(qid: string, e: Event) {
		e.stopPropagation();
		tabs.closeQueryTab(tabId, qid);
	}

	function msg(e: unknown): string {
		return e instanceof Error ? e.message : String(e);
	}

	let dbMenuOpen = $state(false);
	let dbMenuEl = $state<HTMLElement | null>(null);

	let editing = $state(false);
	let editError = $state<string | null>(null);

	let browseTab = $state<'data' | 'schema'>('data');

	let sortCol = $state<string | null>(null);
	let sortDir = $state<'asc' | 'desc'>('asc');

	$effect(() => {
		// Reset sort when the selected table changes.
		selectedTable;
		sortCol = null;
		sortDir = 'asc';
	});

	async function reloadRows() {
		if (!selectedDb || !selectedTable) return;
		rowsLoading = true;
		rowsErr = null;
		try {
			rowSet = await api.databases.rows(connectionId, selectedDb, selectedTable, {
				limit: 200,
				sort: sortCol,
				dir: sortDir
			});
		} catch (e) {
			rowsErr = msg(e);
		} finally {
			rowsLoading = false;
		}
	}

	function onColumnSort(col: string) {
		if (sortCol !== col) {
			sortCol = col;
			sortDir = 'asc';
		} else if (sortDir === 'asc') {
			sortDir = 'desc';
		} else {
			sortCol = null;
			sortDir = 'asc';
		}
		reloadRows();
	}

	$effect(() => {
		if (!editing) return;
		const onKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape') editing = false;
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	async function onSaveEdit(input: ConnectionInput) {
		editError = null;
		const updated = await api.connections.update(connectionId, input);
		conn = updated;
		tabs.update(tabId, { label: updated.name, color: updated.color ?? null });
		editing = false;
		// Drop the active MySQL session so the next request uses the new settings
		await api.sessions.close(connectionId).catch(() => {});
		loadDatabases(connectionId);
	}

	async function onDeleteEdit() {
		editError = null;
		await api.connections.delete(connectionId);
		editing = false;
		tabs.close(tabId);
	}

	$effect(() => {
		if (!dbMenuOpen) return;
		const onDocClick = (e: MouseEvent) => {
			if (dbMenuEl && !dbMenuEl.contains(e.target as Node)) {
				dbMenuOpen = false;
			}
		};
		const onEsc = (e: KeyboardEvent) => {
			if (e.key === 'Escape') dbMenuOpen = false;
		};
		document.addEventListener('mousedown', onDocClick);
		document.addEventListener('keydown', onEsc);
		return () => {
			document.removeEventListener('mousedown', onDocClick);
			document.removeEventListener('keydown', onEsc);
		};
	});
</script>

<div class="flex h-full bg-cream">
	<aside class="flex w-[260px] shrink-0 flex-col border-r border-rule bg-cream-soft">
		<header class="group/conn relative border-b border-rule px-4 py-4">
			{#if conn}
				{@const swatch = colorHex(conn.color)}
				<div class="flex items-center gap-2.5">
					{#if swatch}
						<span
							class="block h-2.5 w-2.5 shrink-0 rounded-full"
							style="background: {swatch}"
							aria-hidden="true"
						></span>
					{/if}
					<div class="font-display text-[18px] leading-tight tracking-tight text-ink">
						{conn.name}
					</div>
				</div>
				<div class="mt-1 font-mono text-[11px] text-ink-faint">
					{conn.username}@{conn.host}:{conn.port}
				</div>
				{#if conn.folder}
					<div class="mt-0.5 font-mono text-[10px] text-ink-ghost">{conn.folder}</div>
				{/if}
				<button
					class="absolute top-3 right-3 cursor-pointer rounded px-1.5 py-0.5 text-[10px] tracking-widest text-ink-faint uppercase opacity-0 transition-opacity hover:bg-cream-deep hover:text-ink group-hover/conn:opacity-100"
					onclick={() => (editing = true)}
				>
					edit
				</button>
			{:else if connErr}
				<pre class="font-mono text-[11px] whitespace-pre-wrap text-crimson">{connErr}</pre>
			{:else}
				<div class="font-mono text-[11px] tracking-widest text-ink-faint uppercase">
					connecting…
				</div>
			{/if}
		</header>

		<section class="relative border-b border-rule" bind:this={dbMenuEl}>
			{#if dbsLoading}
				<div class="px-4 py-3 font-mono text-[11px] tracking-widest text-ink-faint uppercase">
					connecting…
				</div>
			{:else if dbsErr}
				<pre
					class="m-3 rounded bg-crimson-soft p-3 font-mono text-[11px] whitespace-pre-wrap text-crimson">{dbsErr}</pre>
			{:else}
				<button
					class="flex w-full cursor-pointer items-center justify-between px-4 py-3 text-left transition-colors hover:bg-cream-deep/60"
					onclick={() => (dbMenuOpen = !dbMenuOpen)}
					disabled={dbs.length === 0}
				>
					<span class="flex flex-col">
						<span class="font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">
							database
						</span>
						<span
							class="mt-0.5 truncate font-mono text-[14px] {selectedDb
								? 'text-ink'
								: 'text-ink-faint italic'}"
						>
							{selectedDb ?? 'pick one'}
						</span>
					</span>
					<span class="font-mono text-xs text-ink-faint">{dbMenuOpen ? '▴' : '▾'}</span>
				</button>
				{#if dbMenuOpen}
					<div
						class="absolute top-full right-2 left-2 z-20 mt-1 max-h-72 overflow-auto rounded-md border border-rule bg-cream py-1 shadow-[0_8px_24px_-12px_rgba(26,24,20,0.15)]"
						role="menu"
					>
						{#each dbs as db (db)}
							<button
								class="block w-full cursor-pointer truncate px-3 py-1.5 text-left font-mono text-[13px] transition-colors {db ===
								selectedDb
									? 'bg-rust-soft/60 text-rust'
									: 'text-ink hover:bg-cream-soft'}"
								onclick={() => selectDb(db)}
								role="menuitem"
							>
								{db}
							</button>
						{/each}
					</div>
				{/if}
			{/if}
		</section>

		<section class="flex flex-1 flex-col overflow-hidden">
			<div class="px-4 pt-4 pb-1.5 font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">
				Tables{selectedDb ? ` · ${selectedDb}` : ''}
			</div>
			<div class="flex-1 overflow-auto px-2 pb-2">
				{#if !selectedDb}
					<div class="px-3 py-2 font-mono text-[11px] text-ink-faint italic">pick a database</div>
				{:else if tablesLoading}
					<div class="px-3 py-2 font-mono text-[11px] text-ink-faint italic">loading…</div>
				{:else if tablesErr}
					<pre
						class="m-2 rounded bg-crimson-soft p-3 font-mono text-[11px] whitespace-pre-wrap text-crimson">{tablesErr}</pre>
				{:else if tables}
					{#if tables.length === 0}
						<div class="px-3 py-2 font-mono text-[11px] text-ink-faint italic">(no tables)</div>
					{/if}
					{#each tables as t (t.name)}
						<button
							class="group/row flex w-full cursor-pointer items-center gap-2 truncate rounded px-3 py-1.5 text-left font-mono text-[12.5px] transition-colors {t.name ===
							selectedTable
								? 'bg-cream-deep text-ink'
								: 'text-ink-muted hover:bg-cream-deep/40 hover:text-ink'}"
							onclick={() => selectTable(t.name)}
							title="{t.kind}: {t.name}"
						>
							<span
								class="h-3 w-[2px] rounded-full transition-colors {t.name === selectedTable
									? 'bg-rust'
									: 'bg-transparent'}"
							></span>
							<span class="truncate">{t.name}</span>
							{#if t.kind !== 'BASE TABLE'}
								<span class="ml-auto text-[9px] tracking-widest text-ink-faint uppercase">
									{t.kind === 'VIEW' ? 'view' : t.kind.toLowerCase()}
								</span>
							{/if}
						</button>
					{/each}
				{/if}
			</div>
		</section>
	</aside>

	<main class="flex flex-1 flex-col overflow-hidden bg-cream">
		<nav class="flex items-center gap-1 border-b border-rule bg-cream px-3 py-1.5">
			<button
				class="cursor-pointer rounded px-3 py-1 font-mono text-[10px] tracking-[0.22em] uppercase transition-colors {view ===
				'browse'
					? 'bg-cream-deep text-ink'
					: 'text-ink-faint hover:bg-cream-soft hover:text-ink'}"
				onclick={() => setView('browse')}
			>
				Browse
			</button>
			<button
				class="cursor-pointer rounded px-3 py-1 font-mono text-[10px] tracking-[0.22em] uppercase transition-colors {view ===
				'query'
					? 'bg-cream-deep text-ink'
					: 'text-ink-faint hover:bg-cream-soft hover:text-ink'}"
				onclick={() => setView('query')}
			>
				Query
			</button>

			{#if view === 'query'}
				<div class="mx-2 h-4 w-px bg-rule" aria-hidden="true"></div>
				<div class="flex flex-1 items-center gap-px overflow-x-auto overflow-y-hidden">
					{#each queryTabs as q, i (q.id)}
						{@const active = q.id === activeQueryId}
						<div
							class="group/q relative flex shrink-0 items-stretch rounded transition-colors {active
								? 'bg-cream-deep'
								: 'hover:bg-cream-soft'}"
						>
							<button
								class="cursor-pointer py-1 pr-1 pl-2.5 font-mono text-[11px] {active
									? 'font-medium text-ink'
									: 'text-ink-faint hover:text-ink'}"
								onclick={() => activateQuery(q.id)}
							>
								<span class="block max-w-[160px] truncate" title={q.sql || `Query ${i + 1}`}>
									{queryLabel(q, i)}
								</span>
							</button>
							{#if queryTabs.length > 1}
								<button
									class="my-auto mr-1 grid h-4 w-4 cursor-pointer place-items-center rounded text-ink-faint transition-all hover:bg-crimson-soft hover:text-crimson {active
										? 'opacity-100'
										: 'opacity-0 group-hover/q:opacity-100'}"
									onclick={(e) => closeQuerySub(q.id, e)}
									aria-label="close query"
								>
									<span class="text-[11px] leading-none">×</span>
								</button>
							{/if}
						</div>
					{/each}
					<button
						class="ml-1 grid h-6 w-6 cursor-pointer place-items-center rounded text-ink-faint transition-colors hover:bg-cream-deep hover:text-rust"
						onclick={addQuery}
						title="new query (compare side by side)"
						aria-label="new query"
					>
						<span class="text-[14px] leading-none">+</span>
					</button>
				</div>
			{/if}
		</nav>

		{#if view === 'query'}
			<div class="flex-1 overflow-hidden">
				<QueryView {tabId} {connectionId} database={selectedDb} />
			</div>
		{:else if !selectedTable}
			<div class="flex h-full items-center justify-center">
				<div class="text-center">
					<p class="font-display text-2xl text-ink-ghost italic">
						{#if !conn}
							&nbsp;
						{:else if !selectedDb}
							pick a database
						{:else}
							pick a table
						{/if}
					</p>
					<p class="mt-2 font-mono text-[10px] tracking-widest text-ink-faint uppercase">
						from the left
					</p>
				</div>
			</div>
		{:else}
			<header class="flex items-center justify-between border-b border-rule px-5 py-2">
				<div>
					<div class="flex items-baseline gap-1.5 font-mono text-[14px]">
						<span class="text-ink-faint">{selectedDb}</span>
						<span class="text-ink-ghost">/</span>
						<span class="font-medium text-ink">{selectedTable}</span>
					</div>
					{#if browseTab === 'data' && rowSet}
						<div class="mt-1 font-mono text-[10px] tracking-widest text-ink-faint uppercase">
							{rowSet.returned} row{rowSet.returned === 1 ? '' : 's'} · limit {rowSet.limit}
						</div>
					{:else if browseTab === 'schema' && schema}
						<div class="mt-1 font-mono text-[10px] tracking-widest text-ink-faint uppercase">
							{schema.length} column{schema.length === 1 ? '' : 's'}
						</div>
					{/if}
				</div>
				<div class="flex items-center gap-1">
					<button
						class="cursor-pointer rounded px-2.5 py-1 font-mono text-[10px] tracking-[0.22em] uppercase transition-colors {browseTab ===
						'data'
							? 'bg-cream-deep text-ink'
							: 'text-ink-faint hover:bg-cream-soft hover:text-ink'}"
						onclick={() => (browseTab = 'data')}
					>
						Data
					</button>
					<button
						class="cursor-pointer rounded px-2.5 py-1 font-mono text-[10px] tracking-[0.22em] uppercase transition-colors {browseTab ===
						'schema'
							? 'bg-cream-deep text-ink'
							: 'text-ink-faint hover:bg-cream-soft hover:text-ink'}"
						onclick={() => (browseTab = 'schema')}
					>
						Schema
					</button>
				</div>
			</header>

			<div class="flex-1 overflow-auto">
				{#if browseTab === 'data'}
					{#if rowsLoading}
						<div class="px-5 py-3 font-mono text-[11px] text-ink-faint italic">loading…</div>
					{:else if rowsErr}
						<pre
							class="m-3 rounded bg-crimson-soft p-3 font-mono text-[12px] whitespace-pre-wrap text-crimson">{rowsErr}</pre>
					{:else if rowSet}
						<RowGrid
							columns={rowSet.columns}
							rows={rowSet.rows}
							empty="(empty)"
							sortColumn={sortCol}
							{sortDir}
							onSort={onColumnSort}
						/>
					{/if}
				{:else if schemaErr}
					<pre
						class="m-3 rounded bg-crimson-soft p-3 font-mono text-[12px] whitespace-pre-wrap text-crimson">{schemaErr}</pre>
				{:else if schema}
					<table class="w-full font-mono text-[12px]">
						<thead>
							<tr class="border-b border-rule">
								{#each ['column', 'type', 'null', 'key', 'default', 'extra'] as h (h)}
									<th
										class="px-3 py-2 text-left text-[10px] font-semibold tracking-[0.18em] text-ink-muted uppercase"
									>
										{h}
									</th>
								{/each}
							</tr>
						</thead>
						<tbody>
							{#each schema as col (col.name)}
								<tr class="border-b border-rule/60">
									<td class="px-3 py-1.5 font-medium text-ink">{col.name}</td>
									<td class="px-3 py-1.5 text-ink-muted">{col.data_type}</td>
									<td class="px-3 py-1.5 text-rust">
										{col.nullable ? '✓' : ''}
									</td>
									<td class="px-3 py-1.5">
										{#if col.key === 'PRI'}
											<span class="rounded bg-mustard/15 px-1.5 py-0.5 text-[10px] tracking-widest text-mustard uppercase">pri</span>
										{:else if col.key}
											<span class="text-ink-muted">{col.key}</span>
										{/if}
									</td>
									<td class="px-3 py-1.5 text-ink-muted">{col.default ?? ''}</td>
									<td class="px-3 py-1.5 text-ink-muted">{col.extra ?? ''}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{/if}
			</div>
		{/if}
	</main>
</div>

{#if editing && conn}
	<div
		class="fixed inset-0 z-50 flex items-start justify-center bg-ink/30 p-12 backdrop-blur-sm"
		onclick={() => (editing = false)}
		role="presentation"
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="max-h-[80vh] w-full max-w-xl overflow-auto rounded-lg border border-rule bg-cream shadow-[0_24px_64px_-24px_rgba(26,24,20,0.35)]"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			tabindex="-1"
		>
			<div class="flex items-center justify-between border-b border-rule px-5 py-3">
				<h2 class="text-[14px] font-medium text-ink">Edit connection</h2>
				<button
					class="cursor-pointer text-xl leading-none text-ink-faint transition-colors hover:text-rust"
					onclick={() => (editing = false)}
					aria-label="close"
				>
					×
				</button>
			</div>
			<div class="px-5 py-4">
				{#if editError}
					<pre class="mb-4 rounded bg-crimson-soft p-3 font-mono text-[12px] whitespace-pre-wrap text-crimson">{editError}</pre>
				{/if}
				<ConnectionForm
					initial={conn}
					onSubmit={onSaveEdit}
					onDelete={onDeleteEdit}
					submitLabel="Save"
				/>
			</div>
		</div>
	</div>
{/if}
