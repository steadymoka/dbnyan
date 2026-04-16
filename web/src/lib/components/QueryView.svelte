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

		<div class="flex items-center gap-3 border-b border-rule bg-cream-soft/40 px-4 py-2">
			<button
				class="cursor-pointer rounded-md bg-ink px-4 py-1.5 font-mono text-[11px] tracking-[0.18em] text-cream uppercase transition-colors hover:bg-rust disabled:cursor-not-allowed disabled:opacity-40"
				onclick={run}
				disabled={running || !sql.trim()}
			>
				{running ? '…' : 'run'}
			</button>
			<span class="font-mono text-[10px] tracking-widest text-ink-faint uppercase">
				⌘⏎
			</span>
			{#if result}
				<span
					class="ml-auto font-mono text-[10px] tracking-widest text-ink-faint uppercase"
				>
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

		<div class="h-44 shrink-0 border-b border-rule">
			<SqlEditor value={sql} onChange={setSql} onSubmit={run} />
		</div>

		<div class="flex-1 overflow-auto bg-cream">
			{#if error}
				<pre
					class="m-3 rounded bg-crimson-soft p-3 font-mono text-[12px] whitespace-pre-wrap text-crimson">{error}</pre>
			{:else if result}
				{#if result.kind === 'rows'}
					<RowGrid
						columns={result.columns}
						rows={result.rows}
						empty="(query returned no rows)"
					/>
				{:else}
					<div class="m-3 rounded border border-moss/30 bg-moss-soft p-3 font-mono text-[12px] text-moss">
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

	<aside class="flex w-[280px] shrink-0 flex-col border-l border-rule bg-cream-soft">
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
							<div class="flex items-center gap-2 font-mono text-[10px] tracking-widest text-ink-faint uppercase">
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
	</aside>
</div>
