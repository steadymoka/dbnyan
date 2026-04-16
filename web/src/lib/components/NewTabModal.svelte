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

	// Drag & drop state for moving connections between folders
	let dragId = $state<string | null>(null);
	let dropFolder = $state<string | null | undefined>(undefined);

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

	type Group = { folder: string | null; key: string; items: Connection[] };

	const grouped = $derived.by<Group[]>(() => {
		const m = new Map<string, Connection[]>();
		for (const c of saved) {
			const k = c.folder ?? '';
			if (!m.has(k)) m.set(k, []);
			m.get(k)!.push(c);
		}
		const groups: Group[] = [...m.entries()].map(([key, items]) => ({
			folder: key === '' ? null : key,
			key: key === '' ? '(no folder)' : key,
			items
		}));
		// Always keep an empty (no folder) bucket so users can drop items to ungroup.
		if (!groups.find((g) => g.folder === null)) {
			groups.push({ folder: null, key: '(no folder)', items: [] });
		}
		groups.sort((a, b) => {
			// (no folder) goes last for tidiness
			if (a.folder === null) return 1;
			if (b.folder === null) return -1;
			return a.key.localeCompare(b.key);
		});
		return groups;
	});

	function openTab(c: Connection) {
		tabs.open(c);
		onclose();
	}

	function asInput(c: Connection): ConnectionInput {
		return {
			name: c.name,
			host: c.host,
			port: c.port,
			username: c.username,
			password: c.password,
			database: c.database,
			folder: c.folder,
			ssh: c.ssh
		};
	}

	async function clone(c: Connection, e: Event) {
		e.stopPropagation();
		const cloned = await api.connections.create({
			...asInput(c),
			name: `${c.name} copy`
		});
		await refresh();
		view = { kind: 'edit', conn: cloned };
	}

	// --- Drag & drop ---

	function onDragStart(e: DragEvent, c: Connection) {
		if (!e.dataTransfer) return;
		e.dataTransfer.setData('text/plain', c.id);
		e.dataTransfer.effectAllowed = 'move';
		dragId = c.id;
	}

	function onDragEnd() {
		dragId = null;
		dropFolder = undefined;
	}

	function onDragOver(e: DragEvent, folder: string | null) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		dropFolder = folder;
	}

	function onDragLeave(folder: string | null) {
		if (dropFolder === folder) dropFolder = undefined;
	}

	async function onDrop(e: DragEvent, folder: string | null) {
		e.preventDefault();
		const id = e.dataTransfer?.getData('text/plain') || dragId;
		dropFolder = undefined;
		dragId = null;
		if (!id) return;
		const c = saved.find((x) => x.id === id);
		if (!c) return;
		if ((c.folder ?? null) === folder) return;
		await api.connections.update(id, {
			...asInput(c),
			folder: folder ?? undefined
		});
		await refresh();
	}

	// --- Edit / save / delete ---

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
					<p class="mb-2 text-[11px] text-gray-400">
						drag to move between folders · hover for clone/edit
					</p>
					{#each grouped as g (g.key)}
						<section
							class="mb-4 rounded border-2 border-dashed transition-colors {dropFolder === g.folder
								? 'border-blue-400 bg-blue-50/40'
								: 'border-transparent'}"
							ondragover={(e) => onDragOver(e, g.folder)}
							ondragleave={() => onDragLeave(g.folder)}
							ondrop={(e) => onDrop(e, g.folder)}
							role="group"
						>
							<h3 class="mb-1 px-1 text-xs tracking-wide text-gray-500 uppercase">{g.key}</h3>
							{#if g.items.length === 0}
								<div class="rounded border border-dashed border-gray-200 px-3 py-2 text-xs text-gray-400">
									(empty — drop here to ungroup)
								</div>
							{:else}
								<ul class="divide-y rounded border">
									{#each g.items as c (c.id)}
										<li
											class="group flex items-center justify-between px-3 py-2 hover:bg-gray-50 {dragId ===
											c.id
												? 'opacity-40'
												: ''}"
											draggable="true"
											ondragstart={(e) => onDragStart(e, c)}
											ondragend={onDragEnd}
										>
											<button class="flex-1 cursor-pointer text-left" onclick={() => openTab(c)}>
												<div class="text-sm font-medium">{c.name}</div>
												<div class="text-xs text-gray-500">
													{c.username}@{c.host}:{c.port}{c.database ? `/${c.database}` : ''}
													{#if c.ssh}· ssh→{c.ssh.host}{/if}
												</div>
											</button>
											<div class="ml-2 flex items-center gap-2 text-xs opacity-0 group-hover:opacity-100">
												<button
													class="text-gray-500 hover:text-gray-900"
													title="duplicate"
													onclick={(e) => clone(c, e)}
												>
													clone
												</button>
												<button
													class="text-gray-500 hover:text-gray-900"
													onclick={(e) => {
														e.stopPropagation();
														view = { kind: 'edit', conn: c };
													}}
												>
													edit
												</button>
											</div>
										</li>
									{/each}
								</ul>
							{/if}
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
