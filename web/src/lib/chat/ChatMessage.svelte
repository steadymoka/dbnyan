<script lang="ts">
	import { parse, type Segment } from './markdown';
	import type { ChatMessage } from '$lib/stores/chat.svelte';

	type Props = {
		message: ChatMessage;
		onUseSql: (sql: string) => void;
	};
	let { message, onUseSql }: Props = $props();

	const segments = $derived.by<Segment[]>(() =>
		message.content ? parse(message.content) : []
	);

	async function copy(sql: string) {
		try {
			await navigator.clipboard.writeText(sql);
		} catch {
			/* ignore */
		}
	}
</script>

{#if message.role === 'user'}
	<div class="flex justify-end px-3 py-2">
		<div class="max-w-[85%] rounded-lg bg-rust px-3 py-2 font-sans text-[13px] whitespace-pre-wrap text-cream">
			{message.content}
		</div>
	</div>
{:else}
	<div class="flex flex-col gap-2 px-3 py-2">
		<span class="font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">claude</span>
		<div class="chat-prose flex flex-col gap-2 font-sans text-[13px] text-ink">
			{#each segments as seg, i (i)}
				{#if seg.kind === 'html'}
					<!-- eslint-disable-next-line svelte/no-at-html-tags -->
					<div>{@html seg.html}</div>
				{:else}
					<div class="overflow-hidden rounded-md border border-rule bg-cream">
						<div class="flex items-center justify-between border-b border-rule bg-cream-soft/60 px-3 py-1">
							<span class="font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">
								sql
							</span>
							<div class="flex items-center gap-3 text-[11px]">
								<button
									class="cursor-pointer text-ink-muted transition-colors hover:text-ink"
									onclick={() => copy(seg.sql)}
								>
									copy
								</button>
								<button
									class="cursor-pointer rounded bg-rust px-2 py-0.5 font-medium text-cream transition-colors hover:bg-rust-deep"
									onclick={() => onUseSql(seg.sql)}
								>
									use →
								</button>
							</div>
						</div>
						<pre class="max-h-64 overflow-auto px-3 py-2 font-mono text-[12px] whitespace-pre-wrap text-ink">{seg.sql}</pre>
					</div>
				{/if}
			{/each}
			{#if message.streaming}
				<span class="inline-block h-[1em] w-[7px] animate-pulse bg-ink-faint align-[-2px]"></span>
			{/if}
		</div>
		{#if message.error}
			<pre class="rounded bg-crimson-soft p-2 font-mono text-[11.5px] whitespace-pre-wrap text-crimson">{message.error}</pre>
		{/if}
	</div>
{/if}

<style>
	.chat-prose :global(p) {
		margin: 0;
	}
	.chat-prose :global(p + p) {
		margin-top: 0.5em;
	}
	.chat-prose :global(ul),
	.chat-prose :global(ol) {
		margin: 0;
		padding-left: 1.25em;
	}
	.chat-prose :global(li) {
		margin: 0.15em 0;
	}
	.chat-prose :global(code) {
		font-family: ui-monospace, SFMono-Regular, monospace;
		font-size: 0.92em;
		padding: 0.08em 0.3em;
		border-radius: 3px;
		background: var(--color-cream-soft, rgba(0, 0, 0, 0.04));
	}
	.chat-prose :global(pre) {
		margin: 0;
		padding: 0.5em 0.75em;
		overflow: auto;
		font-size: 12px;
	}
	.chat-prose :global(a) {
		color: inherit;
		text-decoration: underline;
	}
	.chat-prose :global(h1),
	.chat-prose :global(h2),
	.chat-prose :global(h3) {
		margin: 0.6em 0 0.25em;
		font-weight: 600;
		font-size: 13px;
	}
</style>
