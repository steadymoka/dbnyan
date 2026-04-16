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
		let m = text.match(/```\s*sql\s*\r?\n([\s\S]*?)```/i);
		if (m) return m[1].trim();
		m = text.match(/```\s*sql\s*\r?\n([\s\S]*?)<\/sql>/i);
		if (m) return m[1].trim();
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

<section class="border-b border-rule bg-cream-soft/60">
	<div class="flex items-start gap-3 px-4 pt-3 pb-2">
		<div class="flex shrink-0 flex-col items-center gap-1 pt-1">
			<span class="font-display text-[14px] italic text-rust">ai</span>
			<span class="h-3 w-px bg-rule"></span>
		</div>
		<textarea
			class="block min-h-[40px] flex-1 resize-none rounded-md border border-rule bg-cream px-3 py-2 font-sans text-[13px] text-ink placeholder:text-ink-faint placeholder:italic focus:border-rust focus:outline-none"
			placeholder="describe the query you want…  (⌘⏎)"
			bind:value={prompt}
			onkeydown={onKey}
			disabled={loading}
			rows="2"
			spellcheck="false"
		></textarea>
		<button
			class="shrink-0 cursor-pointer rounded-md bg-ink px-4 py-2 font-mono text-[11px] tracking-[0.18em] text-cream uppercase transition-colors hover:bg-rust disabled:cursor-not-allowed disabled:opacity-40"
			onclick={generate}
			disabled={loading || !prompt.trim()}
		>
			{loading ? '…' : 'generate'}
		</button>
	</div>

	{#if error}
		<pre
			class="mx-4 mb-3 rounded bg-crimson-soft p-3 font-mono text-[12px] whitespace-pre-wrap text-crimson">{error}</pre>
	{:else if generated}
		<div class="mx-4 mb-3 overflow-hidden rounded-md border border-rule bg-cream">
			<div
				class="flex items-center justify-between border-b border-rule bg-cream-soft/60 px-3 py-1"
			>
				<span class="font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">
					generated sql
					{#if generated.durationMs}
						· {(generated.durationMs / 1000).toFixed(1)}s
					{/if}
					{#if !generated.raw.match(/```sql/i)}
						· <span class="text-mustard">no sql block detected</span>
					{/if}
				</span>
				<div class="flex items-center gap-3 text-[11px]">
					<button
						class="cursor-pointer text-ink-muted transition-colors hover:text-ink"
						onclick={copy}
					>
						copy
					</button>
					<button
						class="cursor-pointer rounded bg-rust px-2 py-0.5 font-medium text-cream transition-colors hover:bg-rust-deep"
						onclick={() => onUseSql(generated!.sql)}
					>
						use →
					</button>
				</div>
			</div>
			<pre
				class="max-h-48 overflow-auto px-3 py-2 font-mono text-[12px] whitespace-pre-wrap text-ink">{generated.sql}</pre>
		</div>
	{/if}
</section>
