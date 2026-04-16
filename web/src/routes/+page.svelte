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
	<div
		class="flex items-end gap-1 border-b border-rule bg-cream-soft px-3 pt-3 pb-0 select-none"
	>
		<a
			href="/"
			class="mr-3 mb-1.5 leading-none whitespace-nowrap text-ink no-underline"
			onclick={(e) => {
				if (active) {
					e.preventDefault();
					tabs.activate('');
					tabs.activeId = null;
				}
			}}
		>
			<span class="font-display text-[20px] italic">
				<span class="font-light">db</span><span class="font-medium">nyan</span>
			</span>
			<span class="text-rust">.</span>
		</a>

		<div class="flex flex-1 items-end overflow-x-auto">
			{#each tabs.tabs as t (t.id)}
				{@const swatch = colorHex(t.color)}
				<div
					class="group/tab relative flex shrink-0 items-stretch rounded-t-[6px] border border-b-0 transition-colors {t.id ===
					tabs.activeId
						? 'border-rule bg-cream -mb-px'
						: 'border-transparent bg-transparent hover:bg-cream/60'}"
				>
					<button
						class="flex cursor-pointer items-center gap-2 px-3 py-2 text-[13px] {t.id ===
						tabs.activeId
							? 'font-medium text-ink'
							: 'text-ink-muted hover:text-ink'}"
						onclick={() => tabs.activate(t.id)}
					>
						{#if swatch}
							<span
								class="block h-2 w-2 shrink-0 rounded-full"
								style="background: {swatch}"
								aria-hidden="true"
							></span>
						{/if}
						<span>{t.label}</span>
					</button>
					<button
						class="mr-2 my-auto rounded-full px-1 text-[14px] leading-none text-ink-faint transition-colors hover:bg-crimson-soft hover:text-crimson {t.id ===
						tabs.activeId
							? 'opacity-100'
							: 'opacity-0 group-hover/tab:opacity-100'}"
						aria-label="close tab"
						onclick={() => tabs.close(t.id)}
					>
						×
					</button>
				</div>
			{/each}

			<button
				class="ml-1 mb-1.5 cursor-pointer rounded px-2 py-1 text-[13px] text-ink-faint hover:text-rust"
				onclick={() => (modalOpen = true)}
				title="open a new tab"
			>
				+ new
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
