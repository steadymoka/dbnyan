<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { replaceState } from '$app/navigation';
	import { tabs, type Tab } from '$lib/stores/tabs.svelte';
	import { api } from '$lib/api';
	import NewTabModal from '$lib/components/NewTabModal.svelte';
	import TabContent from '$lib/components/TabContent.svelte';

	let modalOpen = $state(false);
	let initialized = $state(false);

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

	// Sync active tab state → URL (one-way, after init).
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
		void a_cid; // make deps explicit
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
				return; // unknown connection — leave URL/state as is, user sees empty
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
	<div class="flex items-center border-b bg-gray-50 text-sm">
		<div class="border-r px-3 py-2 font-semibold text-gray-700">dbnyan</div>
		<div class="flex flex-1 items-center overflow-x-auto">
			{#each tabs.tabs as t (t.id)}
				<div
					class="flex shrink-0 items-center border-r {t.id === tabs.activeId
						? 'bg-white'
						: 'hover:bg-gray-100'}"
				>
					<button class="py-2 pr-1 pl-3 text-sm" onclick={() => tabs.activate(t.id)}>
						{t.label}
					</button>
					<button
						class="mr-2 rounded px-1 text-gray-400 hover:bg-gray-200 hover:text-red-600"
						aria-label="close tab"
						onclick={() => tabs.close(t.id)}
					>
						×
					</button>
				</div>
			{/each}
			<button
				class="px-3 py-2 text-sm text-gray-500 hover:bg-gray-100"
				onclick={() => (modalOpen = true)}
			>
				+ new tab
			</button>
		</div>
	</div>

	<main class="flex-1 overflow-auto">
		{#if active}
			{#key active.id}
				<TabContent tabId={active.id} />
			{/key}
		{:else}
			<div class="flex h-full items-center justify-center">
				<button
					class="rounded border border-dashed border-gray-300 px-6 py-4 text-sm text-gray-500 hover:border-gray-400 hover:text-gray-700"
					onclick={() => (modalOpen = true)}
				>
					+ open a connection
				</button>
			</div>
		{/if}
	</main>
</div>

{#if modalOpen}
	<NewTabModal onclose={() => (modalOpen = false)} />
{/if}
