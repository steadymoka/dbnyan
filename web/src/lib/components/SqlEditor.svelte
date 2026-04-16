<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { basicSetup, EditorView } from 'codemirror';
	import { keymap } from '@codemirror/view';
	import { sql } from '@codemirror/lang-sql';

	type Props = {
		value: string;
		onChange: (v: string) => void;
		onSubmit?: () => void;
	};
	let { value, onChange, onSubmit }: Props = $props();

	let container = $state<HTMLDivElement | null>(null);
	let view: EditorView | null = null;
	let applyingExternal = false;

	onMount(() => {
		if (!container) return;
		view = new EditorView({
			doc: value,
			parent: container,
			extensions: [
				basicSetup,
				sql(),
				keymap.of([
					{
						key: 'Mod-Enter',
						preventDefault: true,
						run: () => {
							onSubmit?.();
							return true;
						}
					}
				]),
				EditorView.updateListener.of((u) => {
					if (u.docChanged && !applyingExternal) {
						onChange(u.state.doc.toString());
					}
				}),
				EditorView.theme({
					'&': { height: '100%', backgroundColor: '#fff' },
					'.cm-scroller': {
						fontFamily:
							'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
						fontSize: '13px'
					},
					'.cm-content': { padding: '8px 0' },
					'.cm-gutters': { backgroundColor: '#f9fafb', borderRight: '1px solid #e5e7eb' },
					'.cm-activeLineGutter': { backgroundColor: '#eff6ff' },
					'.cm-activeLine': { backgroundColor: '#f8fafc' },
					'&.cm-focused': { outline: 'none' }
				})
			]
		});
	});

	onDestroy(() => {
		view?.destroy();
	});

	// Sync external value changes (e.g. "Use SQL" from generator, history click).
	$effect(() => {
		if (!view) return;
		const current = view.state.doc.toString();
		if (current !== value) {
			applyingExternal = true;
			view.dispatch({
				changes: { from: 0, to: current.length, insert: value }
			});
			applyingExternal = false;
		}
	});
</script>

<div bind:this={container} class="sql-editor-host h-full w-full"></div>

<style>
	.sql-editor-host :global(.cm-editor) {
		height: 100%;
	}
</style>
