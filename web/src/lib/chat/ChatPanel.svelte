<script lang="ts">
	import { api } from '$lib/api';
	import { chat as chatStore } from '$lib/stores/chat.svelte';
	import ChatInput from './ChatInput.svelte';
	import ChatMessage from './ChatMessage.svelte';

	type Props = {
		connectionId: string;
		database: string | null;
		tabId: string;
		onUseSql: (sql: string) => void;
	};
	let { connectionId, database, tabId, onUseSql }: Props = $props();

	// `$effect.pre` is the right place to seed the state (it runs on first
	// mount + whenever `tabId` changes, and state writes are allowed here).
	// Doing this inside a `$derived` would trip `state_unsafe_mutation`.
	$effect.pre(() => {
		chatStore.ensure(tabId);
	});

	const view = $derived(chatStore.get(tabId)!);

	let scroller: HTMLDivElement | null = $state(null);

	function nearBottom() {
		if (!scroller) return true;
		return scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < 32;
	}

	$effect(() => {
		// Re-run when messages or content length change.
		view.messages.length;
		const last = view.messages[view.messages.length - 1];
		last?.content;

		if (!scroller) return;
		if (nearBottom()) {
			queueMicrotask(() => {
				if (scroller) scroller.scrollTop = scroller.scrollHeight;
			});
		}
	});

	function send(text: string) {
		const userMsg = {
			id: crypto.randomUUID(),
			role: 'user' as const,
			content: text
		};
		chatStore.append(tabId, userMsg);

		const assistantMsg = {
			id: crypto.randomUUID(),
			role: 'assistant' as const,
			content: '',
			streaming: true
		};
		chatStore.append(tabId, assistantMsg);

		const abort = new AbortController();
		chatStore.setStreaming(tabId, true, abort);

		api.chat
			.stream(
				connectionId,
				{ message: text, session_id: view.sessionId, database },
				{
					signal: abort.signal,
					onSession: (sid) => chatStore.setSession(tabId, sid),
					onDelta: (chunk) => chatStore.appendToLast(tabId, chunk),
					onDone: () => {
						chatStore.patchLast(tabId, { streaming: false });
						chatStore.setStreaming(tabId, false, null);
					},
					onError: (err) => {
						chatStore.patchLast(tabId, { streaming: false, error: err });
						chatStore.setStreaming(tabId, false, null);
					}
				}
			)
			.finally(() => {
				// Safety net in case handlers didn't fire (shouldn't happen).
				if (chatStore.get(tabId)?.streaming) {
					chatStore.patchLast(tabId, { streaming: false });
					chatStore.setStreaming(tabId, false, null);
				}
			});
	}

	function abort() {
		view.abort?.abort();
		chatStore.patchLast(tabId, { streaming: false });
		chatStore.setStreaming(tabId, false, null);
	}

	function newSession() {
		if (view.streaming) abort();
		chatStore.clear(tabId);
	}
</script>

<section class="flex flex-1 min-w-0 flex-col bg-cream">
	<header class="flex items-center justify-between border-b border-rule px-4 py-3">
		<span class="font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">chat</span>
		{#if view.messages.length > 0}
			<button
				class="cursor-pointer font-mono text-[10px] tracking-widest text-ink-faint uppercase hover:text-rust"
				onclick={newSession}
				title="start a new conversation (drops context)"
			>
				new
			</button>
		{/if}
	</header>

	<div bind:this={scroller} class="flex-1 overflow-auto">
		{#if view.messages.length === 0}
			<div class="px-4 py-6 text-center font-mono text-[11px] text-ink-faint italic">
				(ask claude about your schema, or to write SQL)
			</div>
		{:else}
			{#each view.messages as m (m.id)}
				<ChatMessage message={m} {onUseSql} />
			{/each}
		{/if}
	</div>

	<ChatInput streaming={view.streaming} onSend={send} onAbort={abort} />
</section>
