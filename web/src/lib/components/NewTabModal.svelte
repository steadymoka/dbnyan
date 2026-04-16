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
	let dropPath = $state<string | null | undefined>(undefined); // null = no-folder zone, string = path

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

	type Node = {
		name: string;
		fullPath: string;
		depth: number;
		children: Node[];
		items: Connection[];
	};

	const tree = $derived.by<Node>(() => {
		const root: Node = { name: '', fullPath: '', depth: -1, children: [], items: [] };
		const byPath = new Map<string, Node>();
		byPath.set('', root);
		for (const c of saved) {
			if (!c.folder) {
				root.items.push(c);
				continue;
			}
			const segments = c.folder.split('/').filter(Boolean);
			let parent = root;
			let pathSoFar = '';
			for (let i = 0; i < segments.length; i++) {
				const seg = segments[i];
				pathSoFar = pathSoFar ? `${pathSoFar}/${seg}` : seg;
				let node = byPath.get(pathSoFar);
				if (!node) {
					node = { name: seg, fullPath: pathSoFar, depth: i, children: [], items: [] };
					byPath.set(pathSoFar, node);
					parent.children.push(node);
				}
				parent = node;
			}
			parent.items.push(c);
		}
		const sortNode = (n: Node) => {
			n.children.sort((a, b) => a.name.localeCompare(b.name));
			n.items.sort((a, b) => a.name.localeCompare(b.name));
			n.children.forEach(sortNode);
		};
		sortNode(root);
		return root;
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
			ssh: c.ssh,
			aws_ssm: c.aws_ssm
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
		dropPath = undefined;
	}

	function onDragOver(e: DragEvent, path: string | null) {
		e.preventDefault();
		e.stopPropagation();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		dropPath = path;
	}

	function onDragLeave(path: string | null) {
		if (dropPath === path) dropPath = undefined;
	}

	async function onDrop(e: DragEvent, path: string | null) {
		e.preventDefault();
		e.stopPropagation();
		const id = e.dataTransfer?.getData('text/plain') || dragId;
		dropPath = undefined;
		dragId = null;
		if (!id) return;
		const c = saved.find((x) => x.id === id);
		if (!c) return;
		if ((c.folder ?? null) === path) return;
		await api.connections.update(id, {
			...asInput(c),
			folder: path ?? undefined
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

	const INDENT = 14;
</script>

{#snippet folderNode(node: Node)}
	{@const folderActive = dropPath === node.fullPath}
	{#if node.fullPath !== ''}
		<div
			class="rounded border-l-2 transition-colors {folderActive
				? 'border-rust bg-rust-soft/30'
				: 'border-rule/60'}"
			style="margin-left: {node.depth * INDENT}px"
			ondragover={(e) => onDragOver(e, node.fullPath)}
			ondragleave={() => onDragLeave(node.fullPath)}
			ondrop={(e) => onDrop(e, node.fullPath)}
			role="group"
		>
			<h3 class="px-2.5 py-1.5 text-[12px] font-medium text-ink">
				<span class="text-ink-faint">▾</span>
				{node.name}
			</h3>
			<div class="pb-2 pl-{(node.depth + 1) * 0}">
				{#each node.children as child (child.fullPath)}
					{@render folderNode(child)}
				{/each}
				{#if node.items.length > 0}
					<ul
						class="mx-2 overflow-hidden rounded-md border border-rule divide-y divide-rule/60"
						style="margin-left: {(node.depth + 1) * INDENT - node.depth * INDENT}px"
					>
						{#each node.items as c (c.id)}
							{@render connectionRow(c)}
						{/each}
					</ul>
				{/if}
			</div>
		</div>
	{:else}
		{#each node.children as child (child.fullPath)}
			{@render folderNode(child)}
		{/each}
	{/if}
{/snippet}

{#snippet connectionRow(c: Connection)}
	{@const swatch = colorHex(c.color)}
	<li
		class="group/row flex items-center gap-3 bg-cream px-3 py-2.5 transition-colors hover:bg-cream-soft {dragId ===
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
			<span
				class="block h-2.5 w-2.5 shrink-0 rounded-full border border-ink-ghost"
				aria-hidden="true"
			></span>
		{/if}
		<button class="flex-1 cursor-pointer text-left" onclick={() => openTab(c)}>
			<div class="text-[14px] leading-tight text-ink">{c.name}</div>
			<div class="mt-0.5 font-mono text-[11px] text-ink-faint">
				{c.username}@{c.host}:{c.port}{c.database ? `/${c.database}` : ''}
				{#if c.ssh}<span class="text-mustard">· ssh→{c.ssh.host}</span>
				{:else if c.aws_ssm}<span class="text-mustard">· ssm→{c.aws_ssm.target}</span>
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
{/snippet}

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
						Drag a row onto a folder · type <span class="font-mono">prod/api</span> in the form for nesting
					</p>

					<div class="space-y-2">
						{@render folderNode(tree)}

						<!-- (no folder) bucket — always visible at the bottom for ungroup drops -->
						{@const noFolderActive = dropPath === null}
						<div
							class="rounded border-l-2 transition-colors {noFolderActive
								? 'border-rust bg-rust-soft/30'
								: 'border-rule/40'}"
							ondragover={(e) => onDragOver(e, null)}
							ondragleave={() => onDragLeave(null)}
							ondrop={(e) => onDrop(e, null)}
							role="group"
						>
							<h3 class="px-2.5 py-1.5 text-[12px] font-medium text-ink-faint italic">
								no folder
							</h3>
							<div class="pb-2">
								{#if tree.items.length > 0}
									<ul class="mx-2 overflow-hidden rounded-md border border-rule divide-y divide-rule/60">
										{#each tree.items as c (c.id)}
											{@render connectionRow(c)}
										{/each}
									</ul>
								{:else}
									<div class="mx-2 rounded-md border border-dashed border-rule px-3 py-2 text-[11px] text-ink-faint italic">
										drop here to ungroup
									</div>
								{/if}
							</div>
						</div>
					</div>
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
