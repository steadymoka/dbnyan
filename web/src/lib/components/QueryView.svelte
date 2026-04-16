<script lang="ts">
	import { api, type HistoryEntry, type QueryResult } from '$lib/api';
	import { tabs } from '$lib/stores/tabs.svelte';
	import RowGrid from './RowGrid.svelte';
	import SqlEditor from './SqlEditor.svelte';
	import SqlGenerator from './SqlGenerator.svelte';

	type Props = {
		tabId: string;
		connectionId: string;
		database: string | null;
		sql: string;
	};
	let { tabId, connectionId, database, sql }: Props = $props();

	let result = $state<QueryResult | null>(null);
	let error = $state<string | null>(null);
	let running = $state(false);

	let history = $state<HistoryEntry[]>([]);

	function msg(e: unknown): string {
		return e instanceof Error ? e.message : String(e);
	}

	async function run() {
		const trimmed = sql.trim();
		if (!trimmed || running) return;
		running = true;
		error = null;
		result = null;
		try {
			result = await api.queries.run(connectionId, trimmed, database ?? undefined);
		} catch (e) {
			error = msg(e);
		} finally {
			running = false;
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

	function setSql(value: string) {
		tabs.update(tabId, { sql: value });
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

	$effect(() => {
		connectionId;
		loadHistory();
	});
</script>

<div class="flex h-full overflow-hidden">
	<div class="flex flex-1 flex-col overflow-hidden">
		<SqlGenerator {connectionId} {database} onUseSql={setSql} />

		<div class="flex items-center gap-2 border-b border-gray-200 bg-gray-50 px-3 py-2 text-sm">
			<button
				class="rounded bg-blue-600 px-3 py-1 text-xs font-medium text-white disabled:opacity-50"
				onclick={run}
				disabled={running || !sql.trim()}
			>
				{running ? 'Running…' : 'Run'}
			</button>
			<span class="text-xs text-gray-400">⌘⏎ to run</span>
			{#if result}
				<span class="ml-auto text-xs text-gray-500">
					{#if result.kind === 'rows'}
						{result.returned} row{result.returned === 1 ? '' : 's'}
					{:else}
						{result.rows_affected} affected{result.last_insert_id
							? ` · last id ${result.last_insert_id}`
							: ''}
					{/if}
					· {result.duration_ms}ms
				</span>
			{/if}
		</div>

		<div class="h-44 shrink-0 border-b border-gray-200">
			<SqlEditor value={sql} onChange={setSql} onSubmit={run} />
		</div>

		<div class="flex-1 overflow-auto">
			{#if error}
				<pre class="m-3 rounded bg-red-50 p-3 text-sm whitespace-pre-wrap text-red-700">{error}</pre>
			{:else if result}
				{#if result.kind === 'rows'}
					<RowGrid columns={result.columns} rows={result.rows} empty="(query returned no rows)" />
				{:else}
					<div class="m-3 rounded bg-green-50 p-3 text-sm text-green-800">
						{result.rows_affected} row{result.rows_affected === 1 ? '' : 's'} affected
						{#if result.last_insert_id !== 0}
							· last_insert_id = {result.last_insert_id}
						{/if}
					</div>
				{/if}
			{:else if !running}
				<div class="px-4 py-3 text-sm text-gray-400">
					write a query and press <kbd class="rounded border bg-gray-50 px-1 text-xs">⌘⏎</kbd>
				</div>
			{/if}
		</div>
	</div>

	<aside class="flex w-72 shrink-0 flex-col border-l border-gray-200 bg-gray-50">
		<header class="flex items-center justify-between border-b border-gray-200 px-3 py-2 text-xs">
			<span class="tracking-wide text-gray-500 uppercase">History</span>
			{#if history.length > 0}
				<button
					class="text-gray-400 hover:text-red-600"
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
				<div class="px-3 py-2 text-xs text-gray-400">(no queries yet)</div>
			{:else}
				{#each history as h (h.id)}
					<div class="group relative border-b border-gray-100 hover:bg-white">
						<button
							class="block w-full px-3 py-2 pr-7 text-left"
							onclick={() => loadFromHistory(h)}
							title="click to load into editor"
						>
							<div class="flex items-center gap-1 text-xs text-gray-500">
								<span class={h.success ? 'text-green-600' : 'text-red-600'}>
									{h.success ? '✓' : '✗'}
								</span>
								<span>{h.duration_ms}ms</span>
								{#if h.rows_returned !== null}
									<span>· {h.rows_returned} row{h.rows_returned === 1 ? '' : 's'}</span>
								{:else if h.rows_affected !== null}
									<span>· {h.rows_affected} affected</span>
								{/if}
							</div>
							<div class="truncate font-mono text-xs text-gray-800">{h.sql}</div>
						</button>
						<button
							class="absolute top-2 right-2 text-gray-300 opacity-0 group-hover:opacity-100 hover:text-red-500"
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
	</aside>
</div>
