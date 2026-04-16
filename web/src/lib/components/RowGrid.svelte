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

	function fmtCell(v: unknown): string {
		if (v === null || v === undefined) return 'NULL';
		if (typeof v === 'object') return JSON.stringify(v);
		return String(v);
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
								class="max-w-xs truncate px-3 py-1.5 align-top {cell === null
									? 'text-ink-faint italic'
									: 'text-ink'}"
								title={text}
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
