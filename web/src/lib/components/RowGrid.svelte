<script lang="ts">
	type Props = {
		columns: string[];
		rows: unknown[][];
		empty?: string;
	};
	let { columns, rows, empty = '(no rows)' }: Props = $props();

	function fmtCell(v: unknown): string {
		if (v === null || v === undefined) return 'NULL';
		if (typeof v === 'object') return JSON.stringify(v);
		return String(v);
	}
</script>

{#if rows.length === 0}
	<div class="px-4 py-3 text-sm text-gray-400">{empty}</div>
{:else}
	<div class="overflow-x-auto">
		<table class="w-full font-mono text-xs">
			<thead class="bg-gray-50">
				<tr>
					{#each columns as c (c)}
						<th class="border-b border-gray-200 px-2 py-1 text-left font-semibold whitespace-nowrap"
							>{c}</th
						>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each rows as row, i (i)}
					<tr class="border-b border-gray-100 hover:bg-blue-50/40">
						{#each row as cell, j (j)}
							{@const text = fmtCell(cell)}
							<td
								class="max-w-xs truncate border-r border-gray-100 px-2 py-1 align-top {cell === null
									? 'text-gray-400 italic'
									: ''}"
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
