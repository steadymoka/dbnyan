<script lang="ts">
	type Props = {
		disabled?: boolean;
		streaming: boolean;
		onSend: (text: string) => void;
		onAbort: () => void;
	};
	let { disabled = false, streaming, onSend, onAbort }: Props = $props();

	let text = $state('');

	function submit() {
		const t = text.trim();
		if (!t || streaming || disabled) return;
		onSend(t);
		text = '';
	}

	function onKey(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
			e.preventDefault();
			submit();
		}
	}
</script>

<div class="relative border-t border-rule bg-cream-soft/60 p-3">
	<textarea
		class="block min-h-[44px] w-full resize-none rounded-md border border-rule bg-cream px-3 py-2 pr-3 pb-9 font-sans text-[13px] text-ink placeholder:text-ink-faint placeholder:italic focus:border-rust focus:outline-none disabled:opacity-60"
		placeholder="ask claude anything about this db…  (⌘⏎)"
		bind:value={text}
		onkeydown={onKey}
		{disabled}
		rows="3"
		spellcheck="false"
	></textarea>
	{#if streaming}
		<button
			class="absolute right-4 bottom-4 cursor-pointer rounded bg-crimson px-2.5 py-1 font-mono text-[10px] tracking-[0.18em] text-cream uppercase transition-colors hover:bg-crimson/90"
			onclick={onAbort}
			title="stop streaming"
		>
			stop
		</button>
	{:else}
		<button
			class="absolute right-4 bottom-4 cursor-pointer rounded bg-ink px-2.5 py-1 font-mono text-[10px] tracking-[0.18em] text-cream uppercase transition-colors hover:bg-rust disabled:cursor-not-allowed disabled:opacity-40"
			onclick={submit}
			disabled={disabled || !text.trim()}
		>
			send
		</button>
	{/if}
</div>
