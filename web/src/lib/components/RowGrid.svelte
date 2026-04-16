<script lang="ts">
	type Props = {
		columns: string[];
		rows: unknown[][];
		empty?: string;
		sortColumn?: string | null;
		sortDir?: 'asc' | 'desc';
		onSort?: (col: string) => void;
	};
	let {
		columns,
		rows,
		empty = '(no rows)',
		sortColumn = null,
		sortDir = 'asc',
		onSort
	}: Props = $props();

	let expanded = $state<{ col: string; value: unknown } | null>(null);

	function fmtCell(v: unknown): string {
		if (v === null || v === undefined) return 'NULL';
		if (typeof v === 'object') return JSON.stringify(v);
		return String(v);
	}

	function fmtFull(v: unknown): string {
		if (v === null || v === undefined) return 'NULL';
		if (typeof v === 'object') return JSON.stringify(v, null, 2);
		return String(v);
	}

	function openCell(col: string, value: unknown) {
		// If the user is selecting text to copy, don't pop the modal.
		if (window.getSelection()?.toString()) return;
		expanded = { col, value };
	}

	$effect(() => {
		if (!expanded) return;
		const onKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape') expanded = null;
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	async function copyValue() {
		if (!expanded) return;
		try {
			await navigator.clipboard.writeText(fmtFull(expanded.value));
		} catch {
			/* ignore */
		}
	}
</script>

{#if rows.length === 0}
	<div class="px-5 py-4 font-mono text-xs tracking-wide text-ink-faint italic">{empty}</div>
{:else}
	<div class="overflow-x-auto">
		<table class="w-full font-mono text-[12px]">
			<thead>
				<tr class="border-b border-rule">
					{#each columns as c (c)}
						{@const active = sortColumn === c}
						<th
							class="bg-cream-soft/60 p-0 text-left text-[10px] font-semibold tracking-[0.18em] whitespace-nowrap text-ink-muted uppercase"
						>
							{#if onSort}
								<button
									class="flex w-full cursor-pointer items-center justify-between gap-1.5 px-3 py-2 transition-colors hover:bg-cream-deep/60 {active
										? 'text-rust'
										: 'text-ink-muted'}"
									onclick={() => onSort?.(c)}
									title="click to sort"
								>
									<span>{c}</span>
									<span class="text-[9px] {active ? 'text-rust' : 'text-ink-ghost opacity-0 group-hover:opacity-100'}">
										{active ? (sortDir === 'asc' ? '▲' : '▼') : '↕'}
									</span>
								</button>
							{:else}
								<div class="px-3 py-2">{c}</div>
							{/if}
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each rows as row, i (i)}
					<tr class="border-b border-rule/60 transition-colors hover:bg-rust-soft/40">
						{#each row as cell, j (j)}
							{@const text = fmtCell(cell)}
							<td
								class="max-w-xs cursor-pointer truncate px-3 py-1.5 align-top {cell === null
									? 'text-ink-faint italic'
									: 'text-ink'}"
								title={text}
								onclick={() => openCell(columns[j], cell)}
							>
								{text}
							</td>
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

{#if expanded}
	<div
		class="fixed inset-0 z-50 flex items-start justify-center bg-ink/30 p-12 backdrop-blur-sm"
		onclick={() => (expanded = null)}
		role="presentation"
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="flex max-h-[80vh] w-full max-w-2xl flex-col overflow-hidden rounded-lg border border-rule bg-cream shadow-[0_24px_64px_-24px_rgba(26,24,20,0.35)]"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			tabindex="-1"
		>
			<header class="flex items-center justify-between border-b border-rule px-5 py-3">
				<div class="flex items-baseline gap-2">
					<span class="font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">
						column
					</span>
					<span class="font-mono text-[13px] font-medium text-ink">{expanded.col}</span>
					{#if expanded.value === null || expanded.value === undefined}
						<span class="font-mono text-[10px] tracking-widest text-ink-faint uppercase italic">
							null
						</span>
					{:else if typeof expanded.value === 'string'}
						<span class="font-mono text-[10px] tracking-widest text-ink-faint uppercase">
							{expanded.value.length} chars
						</span>
					{/if}
				</div>
				<div class="flex items-center gap-3 text-[11px]">
					<button
						class="cursor-pointer text-ink-muted transition-colors hover:text-ink"
						onclick={copyValue}
					>
						copy
					</button>
					<button
						class="cursor-pointer text-xl leading-none text-ink-faint transition-colors hover:text-rust"
						onclick={() => (expanded = null)}
						aria-label="close"
					>
						×
					</button>
				</div>
			</header>
			<pre
				class="flex-1 overflow-auto px-5 py-4 font-mono text-[12.5px] leading-relaxed whitespace-pre-wrap break-words {expanded.value ===
					null || expanded.value === undefined
					? 'text-ink-faint italic'
					: 'text-ink'}">{fmtFull(expanded.value)}</pre>
		</div>
	</div>
{/if}
