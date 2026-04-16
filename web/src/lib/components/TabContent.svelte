<script lang="ts">
	import { api, type ColumnInfo, type Connection, type RowSet, type TableInfo } from '$lib/api';
	import { tabs } from '$lib/stores/tabs.svelte';
	import RowGrid from './RowGrid.svelte';
	import QueryView from './QueryView.svelte';

	type Props = { tabId: string };
	let { tabId }: Props = $props();

	const tab = $derived(tabs.tabs.find((t) => t.id === tabId));
	const connectionId = $derived(tab?.connectionId ?? '');
	const selectedDb = $derived(tab?.db ?? null);
	const selectedTable = $derived(tab?.table ?? null);
	const view = $derived(tab?.view ?? 'browse');
	const sql = $derived(tab?.sql ?? '');

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
				api.databases.rows(id, db, table, 200)
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

	let dbMenuOpen = $state(false);
	let dbMenuEl = $state<HTMLElement | null>(null);

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

	function selectTable(name: string) {
		if (name === selectedTable) return;
		tabs.update(tabId, { table: name, view: 'browse' });
		if (selectedDb) loadTable(connectionId, selectedDb, name);
	}

	function setView(v: 'browse' | 'query') {
		tabs.update(tabId, { view: v });
	}

	function msg(e: unknown): string {
		return e instanceof Error ? e.message : String(e);
	}
</script>

<div class="flex h-full">
	<aside class="flex w-64 shrink-0 flex-col border-r border-gray-200 bg-gray-50 text-sm">
		<header class="border-b border-gray-200 px-3 py-2">
			{#if conn}
				<div class="truncate font-medium" title={conn.name}>{conn.name}</div>
				<div class="truncate text-xs text-gray-500">
					{conn.username}@{conn.host}:{conn.port}
				</div>
			{:else if connErr}
				<pre class="text-xs whitespace-pre-wrap text-red-700">{connErr}</pre>
			{:else}
				<div class="text-gray-400">loading…</div>
			{/if}
		</header>

		<section class="relative border-b border-gray-200" bind:this={dbMenuEl}>
			{#if dbsLoading}
				<div class="px-3 py-2 text-xs text-gray-400">connecting…</div>
			{:else if dbsErr}
				<pre class="m-2 rounded bg-red-50 p-2 text-xs whitespace-pre-wrap text-red-700">{dbsErr}</pre>
			{:else}
				<button
					class="flex w-full items-center justify-between px-3 py-2 text-left hover:bg-gray-100"
					onclick={() => (dbMenuOpen = !dbMenuOpen)}
					disabled={dbs.length === 0}
				>
					<span class="flex items-baseline gap-1 truncate">
						<span class="text-[10px] tracking-wide text-gray-400 uppercase">db</span>
						<span class="truncate font-medium {selectedDb ? '' : 'text-gray-400'}">
							{selectedDb ?? 'pick a database'}
						</span>
					</span>
					<span class="text-xs text-gray-400">▾</span>
				</button>
				{#if dbMenuOpen}
					<div
						class="absolute top-full left-2 right-2 z-20 mt-1 max-h-72 overflow-auto rounded-md border border-gray-200 bg-white py-1 shadow-lg"
						role="menu"
					>
						{#each dbs as db (db)}
							<button
								class="block w-full truncate px-3 py-1 text-left text-sm hover:bg-blue-50 {db ===
								selectedDb
									? 'bg-blue-100 font-medium text-blue-800'
									: ''}"
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
			<div class="px-3 pt-3 pb-1 text-xs tracking-wide text-gray-500 uppercase">
				Tables{selectedDb ? ` · ${selectedDb}` : ''}
			</div>
			<div class="flex-1 overflow-auto px-1 pb-2">
				{#if !selectedDb}
					<div class="px-2 py-1 text-xs text-gray-400">pick a database</div>
				{:else if tablesLoading}
					<div class="px-2 py-1 text-xs text-gray-400">loading…</div>
				{:else if tablesErr}
					<pre class="px-2 py-1 text-xs whitespace-pre-wrap text-red-700">{tablesErr}</pre>
				{:else if tables}
					{#if tables.length === 0}
						<div class="px-2 py-1 text-xs text-gray-400">(no tables)</div>
					{/if}
					{#each tables as t (t.name)}
						<button
							class="block w-full truncate rounded px-2 py-1 text-left hover:bg-gray-200 {t.name ===
							selectedTable
								? 'bg-blue-100 font-medium text-blue-800'
								: ''}"
							onclick={() => selectTable(t.name)}
							title="{t.kind}: {t.name}"
						>
							{t.name}
							{#if t.kind !== 'BASE TABLE'}
								<span class="ml-1 text-xs text-gray-400">{t.kind}</span>
							{/if}
						</button>
					{/each}
				{/if}
			</div>
		</section>
	</aside>

	<main class="flex flex-1 flex-col overflow-hidden">
		<nav class="flex items-center gap-1 border-b border-gray-200 bg-white px-2 py-1">
			<button
				class="rounded px-2 py-1 text-xs {view === 'browse'
					? 'bg-gray-100 font-medium text-gray-800'
					: 'text-gray-500 hover:bg-gray-100'}"
				onclick={() => setView('browse')}
			>
				Browse
			</button>
			<button
				class="rounded px-2 py-1 text-xs {view === 'query'
					? 'bg-gray-100 font-medium text-gray-800'
					: 'text-gray-500 hover:bg-gray-100'}"
				onclick={() => setView('query')}
			>
				Query
			</button>
		</nav>

		{#if view === 'query'}
			<div class="flex-1 overflow-hidden">
				<QueryView {tabId} {connectionId} database={selectedDb} {sql} />
			</div>
		{:else if !selectedTable}
			<div class="flex h-full items-center justify-center text-sm text-gray-400">
				{#if !conn}
					&nbsp;
				{:else if !selectedDb}
					← pick a database
				{:else}
					← pick a table
				{/if}
			</div>
		{:else}
			<header class="border-b border-gray-200 px-4 py-2">
				<div class="font-mono text-sm">
					<span class="text-gray-500">{selectedDb}.</span><span class="font-medium"
						>{selectedTable}</span
					>
				</div>
				{#if rowSet}
					<div class="text-xs text-gray-500">
						{rowSet.returned} row{rowSet.returned === 1 ? '' : 's'} · limit {rowSet.limit}
					</div>
				{/if}
			</header>

			<div class="flex-1 overflow-auto">
				<section class="border-b border-gray-200">
					<div
						class="border-b border-gray-100 bg-gray-50 px-4 py-1 text-xs tracking-wide text-gray-500 uppercase"
					>
						Data
					</div>
					{#if rowsLoading}
						<div class="px-4 py-3 text-sm text-gray-400">loading…</div>
					{:else if rowsErr}
						<pre class="px-4 py-3 text-sm whitespace-pre-wrap text-red-700">{rowsErr}</pre>
					{:else if rowSet}
						<RowGrid columns={rowSet.columns} rows={rowSet.rows} empty="(empty)" />
					{/if}
				</section>

				<section>
					<div
						class="border-b border-gray-100 bg-gray-50 px-4 py-1 text-xs tracking-wide text-gray-500 uppercase"
					>
						Schema
					</div>
					{#if schemaErr && !rowsErr}
						<pre class="px-4 py-3 text-sm whitespace-pre-wrap text-red-700">{schemaErr}</pre>
					{:else if schema}
						<table class="w-full font-mono text-xs">
							<thead class="bg-gray-50">
								<tr>
									<th class="border-b border-gray-200 px-2 py-1 text-left font-semibold">column</th>
									<th class="border-b border-gray-200 px-2 py-1 text-left font-semibold">type</th>
									<th class="border-b border-gray-200 px-2 py-1 text-left font-semibold">null</th>
									<th class="border-b border-gray-200 px-2 py-1 text-left font-semibold">key</th>
									<th class="border-b border-gray-200 px-2 py-1 text-left font-semibold">default</th>
									<th class="border-b border-gray-200 px-2 py-1 text-left font-semibold">extra</th>
								</tr>
							</thead>
							<tbody>
								{#each schema as col (col.name)}
									<tr class="border-b border-gray-100">
										<td class="px-2 py-1 font-medium">{col.name}</td>
										<td class="px-2 py-1 text-gray-700">{col.data_type}</td>
										<td class="px-2 py-1">
											{col.nullable ? '✓' : ''}
										</td>
										<td class="px-2 py-1 text-gray-700">{col.key ?? ''}</td>
										<td class="px-2 py-1 text-gray-700">{col.default ?? ''}</td>
										<td class="px-2 py-1 text-gray-700">{col.extra ?? ''}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					{/if}
				</section>
			</div>
		{/if}
	</main>
</div>
