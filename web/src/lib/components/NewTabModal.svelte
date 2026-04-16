<script lang="ts">
	import { api, type Connection, type ConnectionInput } from '$lib/api';
	import { tabs } from '$lib/stores/tabs.svelte';
	import ConnectionForm from './ConnectionForm.svelte';

	type Props = { onclose: () => void };
	let { onclose }: Props = $props();

	type View = { kind: 'picker' } | { kind: 'new' } | { kind: 'edit'; conn: Connection };
	let view = $state<View>({ kind: 'picker' });

	let saved = $state<Connection[]>([]);
	let loadErr = $state<string | null>(null);

	async function refresh() {
		try {
			saved = await api.connections.list();
			tabs.syncWithConnections(saved);
			loadErr = null;
		} catch (e) {
			loadErr = e instanceof Error ? e.message : String(e);
		}
	}

	$effect(() => {
		refresh();
	});

	$effect(() => {
		const onKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape') onclose();
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	const grouped = $derived.by(() => {
		const m = new Map<string, Connection[]>();
		for (const c of saved) {
			const k = c.folder || '(no folder)';
			if (!m.has(k)) m.set(k, []);
			m.get(k)!.push(c);
		}
		return [...m.entries()].sort(([a], [b]) => a.localeCompare(b));
	});

	function openTab(c: Connection) {
		tabs.open(c);
		onclose();
	}

	async function onCreate(input: ConnectionInput) {
		const c = await api.connections.create(input);
		tabs.open(c);
		onclose();
	}

	async function onSaveEdit(input: ConnectionInput) {
		if (view.kind !== 'edit') return;
		await api.connections.update(view.conn.id, input);
		await refresh();
		view = { kind: 'picker' };
	}

	async function onDeleteEdit() {
		if (view.kind !== 'edit') return;
		await api.connections.delete(view.conn.id);
		await refresh();
		view = { kind: 'picker' };
	}
</script>

<div
	class="fixed inset-0 z-50 flex items-start justify-center bg-black/30 p-12"
	onclick={onclose}
	role="presentation"
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		class="max-h-[80vh] w-full max-w-xl overflow-auto rounded-lg bg-white shadow-xl"
		onclick={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div class="flex items-center justify-between border-b p-4">
			<h2 class="font-semibold">
				{#if view.kind === 'picker'}New tab
				{:else if view.kind === 'new'}New connection
				{:else}Edit connection
				{/if}
			</h2>
			<button class="text-gray-400 hover:text-gray-700" onclick={onclose}>×</button>
		</div>
		<div class="p-4">
			{#if view.kind === 'picker'}
				{#if loadErr}
					<pre class="mb-3 rounded bg-red-50 p-3 text-sm whitespace-pre-wrap text-red-700">{loadErr}</pre>
				{/if}
				{#if saved.length === 0}
					<p class="text-sm text-gray-500">no saved connections yet.</p>
				{:else}
					{#each grouped as [folder, items] (folder)}
						<section class="mb-4">
							<h3 class="mb-1 text-xs tracking-wide text-gray-500 uppercase">{folder}</h3>
							<ul class="divide-y rounded border">
								{#each items as c (c.id)}
									<li class="flex items-center justify-between px-3 py-2 hover:bg-gray-50">
										<button class="flex-1 text-left" onclick={() => openTab(c)}>
											<div class="text-sm font-medium">{c.name}</div>
											<div class="text-xs text-gray-500">
												{c.username}@{c.host}:{c.port}{c.database ? `/${c.database}` : ''}
												{#if c.ssh}· ssh→{c.ssh.host}{/if}
											</div>
										</button>
										<button
											class="ml-2 text-xs text-gray-400 hover:text-gray-700"
											onclick={() => (view = { kind: 'edit', conn: c })}
										>
											edit
										</button>
									</li>
								{/each}
							</ul>
						</section>
					{/each}
				{/if}
				<div class="mt-4">
					<button
						class="rounded bg-blue-600 px-3 py-1 text-sm text-white"
						onclick={() => (view = { kind: 'new' })}
					>
						+ Create new connection
					</button>
				</div>
			{:else if view.kind === 'new'}
				<ConnectionForm onSubmit={onCreate} submitLabel="Create & open" />
				<button
					class="mt-3 text-xs text-gray-500 hover:underline"
					onclick={() => (view = { kind: 'picker' })}
				>
					← back
				</button>
			{:else}
				<ConnectionForm
					initial={view.conn}
					onSubmit={onSaveEdit}
					onDelete={onDeleteEdit}
					submitLabel="Save"
				/>
				<button
					class="mt-3 text-xs text-gray-500 hover:underline"
					onclick={() => (view = { kind: 'picker' })}
				>
					← back
				</button>
			{/if}
		</div>
	</div>
</div>
