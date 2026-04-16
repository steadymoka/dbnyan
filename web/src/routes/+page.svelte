<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { replaceState } from '$app/navigation';
	import { tabs, type Tab } from '$lib/stores/tabs.svelte';
	import { api } from '$lib/api';
	import { colorHex } from '$lib/colors';
	import NewTabModal from '$lib/components/NewTabModal.svelte';
	import TabContent from '$lib/components/TabContent.svelte';

	let modalOpen = $state(false);
	let initialized = $state(false);

	let dragTabId = $state<string | null>(null);
	let dropHint = $state<{ tabId: string; before: boolean } | null>(null);

	function onTabDragStart(e: DragEvent, tabId: string) {
		if (!e.dataTransfer) return;
		e.dataTransfer.effectAllowed = 'move';
		e.dataTransfer.setData('application/x-dbnyan-tab', tabId);
		dragTabId = tabId;
	}

	function onTabDragEnd() {
		dragTabId = null;
		dropHint = null;
	}

	function onTabDragOver(e: DragEvent, tabId: string) {
		// Only reorder when dragging another tab — ignore other drag types
		if (!dragTabId) return;
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		if (dragTabId === tabId) {
			dropHint = null;
			return;
		}
		const el = e.currentTarget as HTMLElement;
		const rect = el.getBoundingClientRect();
		const before = e.clientX < rect.left + rect.width / 2;
		dropHint = { tabId, before };
	}

	function onTabDragLeave(tabId: string) {
		if (dropHint?.tabId === tabId) dropHint = null;
	}

	function onTabDrop(e: DragEvent, tabId: string) {
		if (!dragTabId) return;
		e.preventDefault();
		const src = dragTabId;
		const el = e.currentTarget as HTMLElement;
		const rect = el.getBoundingClientRect();
		const before = e.clientX < rect.left + rect.width / 2;
		dragTabId = null;
		dropHint = null;
		if (src && src !== tabId) tabs.reorder(src, tabId, before);
	}

	onMount(() => {
		tabs.load();
		applyUrlToState().finally(() => {
			initialized = true;
		});
		const onPop = () => {
			applyUrlToState();
		};
		window.addEventListener('popstate', onPop);
		return () => window.removeEventListener('popstate', onPop);
	});

	const active = $derived(tabs.tabs.find((t) => t.id === tabs.activeId) ?? null);

	$effect(() => {
		if (!initialized) return;
		const a = active;
		const a_db = a?.db ?? '';
		const a_table = a?.table ?? '';
		const a_view = a?.view ?? '';
		const a_cid = a?.connectionId ?? '';
		void a_db;
		void a_table;
		void a_view;
		void a_cid;
		syncUrlFrom(a);
	});

	function syncUrlFrom(a: Tab | null) {
		const params = new URLSearchParams();
		if (a) {
			params.set('cid', a.connectionId);
			if (a.db) params.set('db', a.db);
			if (a.table) params.set('t', a.table);
			if (a.view === 'query') params.set('v', 'q');
		}
		const newSearch = params.toString() ? `?${params}` : '';
		const newUrl = `${page.url.pathname}${newSearch}`;
		const cur = `${page.url.pathname}${page.url.search}`;
		if (cur !== newUrl) {
			replaceState(newUrl, {});
		}
	}

	async function applyUrlToState() {
		const params = page.url.searchParams;
		const cid = params.get('cid');
		if (!cid) return;
		const db = params.get('db');
		const table = params.get('t');
		const view: 'browse' | 'query' = params.get('v') === 'q' ? 'query' : 'browse';

		let tab = tabs.tabs.find((t) => t.connectionId === cid) ?? null;
		if (!tab) {
			try {
				const conn = await api.connections.get(cid);
				tabs.open(conn);
				tab = tabs.tabs.find((t) => t.connectionId === cid) ?? null;
			} catch {
				return;
			}
		} else {
			tabs.activate(tab.id);
		}
		if (tab) {
			tabs.update(tab.id, { db: db ?? null, table: table ?? null, view });
		}
	}
</script>

<div class="flex h-screen flex-col">
	<!-- chrome / tab bar -->
	<div class="flex items-end border-b border-rule bg-cream-soft pl-3 select-none">
		<a
			href="/"
			class="mr-3 mb-2 flex items-baseline leading-none whitespace-nowrap text-ink no-underline"
			onclick={(e) => {
				if (active) {
					e.preventDefault();
					tabs.activeId = null;
				}
			}}
		>
			<span class="font-display text-[18px] italic">
				<span class="font-light">db</span><span class="font-medium">nyan</span>
			</span>
			<span class="font-display text-[18px] text-rust leading-none">.</span>
		</a>

		<div class="mb-2 mr-2 h-5 w-px bg-rule"></div>

		<div class="flex flex-1 items-end gap-px overflow-x-auto overflow-y-hidden pt-2 pr-2">
			{#each tabs.tabs as t (t.id)}
				{@const swatch = colorHex(t.color)}
				{@const active = t.id === tabs.activeId}
				<div
					class="group/tab relative flex h-9 shrink-0 items-stretch rounded-t-md border border-b-0 transition-colors {active
						? 'z-10 -mb-px border-rule bg-cream'
						: 'border-transparent hover:bg-cream/60'}"
				>
					{#if active}
						<span
							class="absolute top-0 right-2 left-2 h-[2px] rounded-b-sm"
							style={swatch ? `background: ${swatch}` : ''}
							class:bg-rust={!swatch}
							aria-hidden="true"
						></span>
					{/if}
					<button
						class="flex cursor-pointer items-center gap-2 pr-1 pl-3 text-[12.5px] {active
							? 'font-medium text-ink'
							: 'text-ink-muted hover:text-ink'}"
						onclick={() => tabs.activate(t.id)}
					>
						{#if !active && swatch}
							<span
								class="block h-1.5 w-1.5 shrink-0 rounded-full"
								style="background: {swatch}"
								aria-hidden="true"
							></span>
						{/if}
						<span class="max-w-[160px] truncate" title={t.label}>{t.label}</span>
					</button>
					<button
						class="my-auto mr-1.5 grid h-5 w-5 cursor-pointer place-items-center rounded text-ink-faint transition-all hover:bg-crimson-soft hover:text-crimson {active
							? 'opacity-100'
							: 'opacity-0 group-hover/tab:opacity-100'}"
						aria-label="close tab"
						onclick={() => tabs.close(t.id)}
					>
						<span class="text-[13px] leading-none">×</span>
					</button>
				</div>
			{/each}

			<button
				class="mb-1 ml-1 grid h-7 w-7 cursor-pointer place-items-center rounded-md text-ink-faint transition-colors hover:bg-cream-deep hover:text-rust"
				onclick={() => (modalOpen = true)}
				title="open a new tab"
				aria-label="open a new tab"
			>
				<span class="text-[18px] leading-none font-light">+</span>
			</button>
		</div>
	</div>

	<main class="flex-1 overflow-hidden">
		{#if active}
			{#key active.id}
				<TabContent tabId={active.id} />
			{/key}
		{:else}
			<div class="flex h-full items-center justify-center">
				<div class="max-w-md px-8 text-center">
					<h1 class="font-display text-[64px] leading-none text-ink">
						<span class="font-light italic">db</span><span class="font-medium italic"
							>nyan</span
						><span class="font-display text-rust">.</span>
					</h1>
					<p class="mt-3 font-mono text-[11px] tracking-[0.25em] text-ink-faint uppercase">
						a small, hand-built MySQL admin
					</p>
					<button
						class="mt-10 inline-flex cursor-pointer items-center gap-2 rounded-full border border-ink px-6 py-2.5 text-[13px] font-medium text-ink transition-colors hover:bg-ink hover:text-cream"
						onclick={() => (modalOpen = true)}
					>
						<span class="font-display text-base leading-none">+</span> open a connection
					</button>
					<p class="mt-12 font-mono text-[10px] tracking-widest text-ink-ghost uppercase">
						⌘⏎ run · drag to organize
					</p>
				</div>
			</div>
		{/if}
	</main>
</div>

{#if modalOpen}
	<NewTabModal onclose={() => (modalOpen = false)} />
{/if}
