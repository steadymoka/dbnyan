<script lang="ts">
	import { api } from '$lib/api';

	type Props = {
		connectionId: string;
		database: string | null;
		onUseSql: (sql: string) => void;
	};
	let { connectionId, database, onUseSql }: Props = $props();

	let prompt = $state('');
	let generated = $state<{ sql: string; raw: string; durationMs: number | null } | null>(null);
	let error = $state<string | null>(null);
	let loading = $state(false);

	async function generate() {
		const p = prompt.trim();
		if (!p || loading) return;
		loading = true;
		error = null;
		generated = null;
		try {
			// One-shot: never pass session_id, so each call is a fresh schema context.
			const res = await api.chat.send(connectionId, { message: p, database });
			const sql = extractSql(res.text);
			generated = {
				sql: sql ?? res.text,
				raw: res.text,
				durationMs: res.duration_ms
			};
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function onKey(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
			e.preventDefault();
			generate();
		}
	}

	function extractSql(text: string): string | null {
		// ```sql ... ```
		let m = text.match(/```\s*sql\s*\r?\n([\s\S]*?)```/i);
		if (m) return m[1].trim();
		// ```sql ... </sql>   (model occasionally closes with </sql>)
		m = text.match(/```\s*sql\s*\r?\n([\s\S]*?)<\/sql>/i);
		if (m) return m[1].trim();
		// bare ``` block (no lang tag)
		m = text.match(/```\s*\r?\n([\s\S]*?)```/);
		if (m) return m[1].trim();
		return null;
	}

	async function copy() {
		if (!generated) return;
		try {
			await navigator.clipboard.writeText(generated.sql);
		} catch {
			/* ignore */
		}
	}
</script>

<section class="border-b border-gray-200 bg-blue-50/40">
	<div class="flex items-start gap-2 p-2">
		<textarea
			class="block min-h-12 flex-1 resize-none rounded border border-gray-300 px-2 py-1 text-xs focus:border-blue-400 focus:outline-none"
			placeholder="describe the query you want…  (⌘⏎ to generate)"
			bind:value={prompt}
			onkeydown={onKey}
			disabled={loading}
			rows="2"
			spellcheck="false"
		></textarea>
		<button
			class="shrink-0 rounded bg-blue-600 px-3 py-1.5 text-xs font-medium text-white disabled:opacity-50"
			onclick={generate}
			disabled={loading || !prompt.trim()}
		>
			{loading ? 'Generating…' : 'Generate'}
		</button>
	</div>

	{#if error}
		<pre class="mx-2 mb-2 rounded bg-red-50 p-2 text-xs whitespace-pre-wrap text-red-700">{error}</pre>
	{:else if generated}
		<div class="mx-2 mb-2 overflow-hidden rounded border border-gray-200 bg-white">
			<pre
				class="max-h-48 overflow-auto p-2 font-mono text-xs whitespace-pre-wrap">{generated.sql}</pre>
			<div class="flex items-center justify-between border-t border-gray-100 bg-gray-50 px-2 py-1">
				<span class="text-[10px] text-gray-400">
					claude{generated.durationMs ? ` · ${(generated.durationMs / 1000).toFixed(1)}s` : ''}
					{#if !generated.raw.match(/```sql/i)}
						<span class="text-amber-600"> · no sql block detected</span>
					{/if}
				</span>
				<div class="flex gap-2">
					<button class="text-xs text-gray-500 hover:text-gray-800" onclick={copy}>copy</button>
					<button
						class="rounded bg-blue-600 px-2 py-0.5 text-xs text-white"
						onclick={() => onUseSql(generated!.sql)}
					>
						Use
					</button>
				</div>
			</div>
		</div>
	{/if}
</section>
