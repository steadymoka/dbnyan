<script lang="ts">
	import { api, type Connection, type ConnectionInput } from '$lib/api';
	import { tabs } from '$lib/stores/tabs.svelte';
	import { colorHex } from '$lib/colors';
	import ConnectionForm from './ConnectionForm.svelte';

	type Props = { onclose: () => void };
	let { onclose }: Props = $props();

	type View = { kind: 'picker' } | { kind: 'new' } | { kind: 'edit'; conn: Connection };
	let view = $state<View>({ kind: 'picker' });

	let saved = $state<Connection[]>([]);
	let loadErr = $state<string | null>(null);

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

	type Group = {
		folder: string | null; // full path e.g. "prod/api"; null for no-folder
		segments: string[]; // ["prod", "api"]
		depth: number; // 0 for top-level folder, 1 for one level deep, …
		items: Connection[];
	};

	const grouped = $derived.by<Group[]>(() => {
		const m = new Map<string, Connection[]>();
		for (const c of saved) {
			const k = c.folder ?? '';
			if (!m.has(k)) m.set(k, []);
			m.get(k)!.push(c);
		}
		const groups: Group[] = [];
		for (const [path, items] of m.entries()) {
			if (path === '') {
				groups.push({ folder: null, segments: [], depth: 0, items });
			} else {
				const segments = path.split('/').filter(Boolean);
				groups.push({
					folder: path,
					segments,
					depth: Math.max(0, segments.length - 1),
					items
				});
			}
		}
		// always keep an empty (no folder) bucket at the bottom
		if (!groups.find((g) => g.folder === null)) {
			groups.push({ folder: null, segments: [], depth: 0, items: [] });
		}
		groups.sort((a, b) => {
			if (a.folder === null) return 1;
			if (b.folder === null) return -1;
			return a.folder.localeCompare(b.folder);
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
			color: c.color,
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
	class="fixed inset-0 z-50 flex items-start justify-center bg-ink/30 p-12 backdrop-blur-sm"
	onclick={onclose}
	role="presentation"
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		class="max-h-[80vh] w-full max-w-xl overflow-auto rounded-lg border border-rule bg-cream shadow-[0_24px_64px_-24px_rgba(26,24,20,0.35)]"
		onclick={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div class="flex items-center justify-between border-b border-rule px-5 py-3">
			<h2 class="text-[14px] font-medium text-ink">
				{#if view.kind === 'picker'}New tab
				{:else if view.kind === 'new'}New connection
				{:else}Edit connection
				{/if}
			</h2>
			<button
				class="cursor-pointer text-xl leading-none text-ink-faint transition-colors hover:text-rust"
				onclick={onclose}
				aria-label="close"
			>
				×
			</button>
		</div>

		<div class="px-5 py-4">
			{#if view.kind === 'picker'}
				{#if loadErr}
					<pre
						class="mb-4 rounded bg-crimson-soft p-3 font-mono text-[12px] whitespace-pre-wrap text-crimson">{loadErr}</pre>
				{/if}
				{#if saved.length === 0}
					<p class="mb-4 text-[11px] text-ink-faint">no saved connections yet.</p>
				{:else}
					<p class="mb-3 text-[11px] text-ink-faint">
						Drag to reorder folder · use <span class="font-mono">/</span> for nested folders
					</p>
					{#each grouped as g (g.folder ?? '__root__')}
						<section
							class="mb-3 rounded-md border-2 border-dashed transition-colors {dropFolder ===
							g.folder
								? 'border-rust bg-rust-soft/30'
								: 'border-transparent'}"
							style="margin-left: {g.depth * 16}px"
							ondragover={(e) => onDragOver(e, g.folder)}
							ondragleave={() => onDragLeave(g.folder)}
							ondrop={(e) => onDrop(e, g.folder)}
							role="group"
						>
							<h3 class="mb-1.5 px-2 text-[11px] text-ink-muted">
								{#if g.folder === null}
									<span class="text-ink-faint italic">no folder</span>
								{:else}
									{#each g.segments as seg, i (i)}
										{#if i > 0}<span class="px-1 text-ink-ghost">/</span>{/if}
										<span class={i === g.segments.length - 1 ? 'text-ink-muted' : 'text-ink-ghost'}>
											{seg}
										</span>
									{/each}
								{/if}
							</h3>
							{#if g.items.length === 0}
								<div class="rounded-md border border-dashed border-rule px-3 py-2 text-[11px] text-ink-faint italic">
									empty — drop here to ungroup
								</div>
							{:else}
								<ul class="overflow-hidden rounded-md border border-rule divide-y divide-rule/60">
									{#each g.items as c (c.id)}
										{@const swatch = colorHex(c.color)}
										<li
											class="group/row flex items-center gap-3 bg-cream px-4 py-3 transition-colors hover:bg-cream-soft {dragId ===
											c.id
												? 'opacity-40'
												: ''}"
											draggable="true"
											ondragstart={(e) => onDragStart(e, c)}
											ondragend={onDragEnd}
										>
											{#if swatch}
												<span
													class="block h-2.5 w-2.5 shrink-0 rounded-full"
													style="background: {swatch}"
													aria-hidden="true"
												></span>
											{:else}
												<span class="block h-2.5 w-2.5 shrink-0 rounded-full border border-ink-ghost" aria-hidden="true"></span>
											{/if}
											<button
												class="flex-1 cursor-pointer text-left"
												onclick={() => openTab(c)}
											>
												<div class="text-[14px] leading-tight text-ink">
													{c.name}
												</div>
												<div class="mt-0.5 font-mono text-[11px] text-ink-faint">
													{c.username}@{c.host}:{c.port}{c.database
														? `/${c.database}`
														: ''}
													{#if c.ssh}
														<span class="text-mustard">· ssh→{c.ssh.host}</span>
													{/if}
												</div>
											</button>
											<div class="flex items-center gap-3 text-[11px] opacity-0 transition-opacity group-hover/row:opacity-100">
												<button
													class="cursor-pointer text-ink-muted hover:text-rust"
													title="duplicate"
													onclick={(e) => clone(c, e)}
												>
													clone
												</button>
												<button
													class="cursor-pointer text-ink-muted hover:text-rust"
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
				<div class="mt-5 border-t border-rule pt-4">
					<button
						class="cursor-pointer rounded-md bg-ink px-4 py-1.5 text-[12px] font-medium text-cream transition-colors hover:bg-rust"
						onclick={() => (view = { kind: 'new' })}
					>
						+ New connection
					</button>
				</div>
			{:else if view.kind === 'new'}
				<ConnectionForm onSubmit={onCreate} submitLabel="Create & open" />
				<button
					class="mt-3 cursor-pointer text-[11px] text-ink-faint hover:text-ink"
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
					class="mt-3 cursor-pointer text-[11px] text-ink-faint hover:text-ink"
					onclick={() => (view = { kind: 'picker' })}
				>
					← back
				</button>
			{/if}
		</div>
	</div>
</div>
