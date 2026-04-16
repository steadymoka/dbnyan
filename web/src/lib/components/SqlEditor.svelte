<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { basicSetup, EditorView } from 'codemirror';
	import { keymap } from '@codemirror/view';
	import { sql } from '@codemirror/lang-sql';
	import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
	import { tags as t } from '@lezer/highlight';

	type Props = {
		value: string;
		onChange: (v: string) => void;
		onSubmit?: () => void;
	};
	let { value, onChange, onSubmit }: Props = $props();

	let container = $state<HTMLDivElement | null>(null);
	let view: EditorView | null = null;
	let applyingExternal = false;

	// Editorial palette — matches our design tokens
	const editorialHighlight = HighlightStyle.define([
		{ tag: t.keyword, color: '#C44536', fontStyle: 'italic' }, // rust
		{ tag: t.operator, color: '#5C564E' },
		{ tag: t.string, color: '#3E6B4A' }, // moss
		{ tag: t.number, color: '#B8851E' }, // mustard
		{ tag: t.bool, color: '#B8851E' },
		{ tag: t.null, color: '#A39C92' },
		{ tag: t.comment, color: '#A39C92', fontStyle: 'italic' },
		{ tag: t.variableName, color: '#1A1814' },
		{ tag: t.typeName, color: '#5C564E' },
		{ tag: t.function(t.variableName), color: '#1A1814', fontWeight: '600' },
		{ tag: t.punctuation, color: '#5C564E' }
	]);

	onMount(() => {
		if (!container) return;
		view = new EditorView({
			doc: value,
			parent: container,
			extensions: [
				basicSetup,
				sql(),
				syntaxHighlighting(editorialHighlight),
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
					'&': {
						height: '100%',
						backgroundColor: '#FAF7F2',
						color: '#1A1814'
					},
					'.cm-scroller': {
						fontFamily:
							"'JetBrains Mono', ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Monaco, Consolas, monospace",
						fontSize: '13px',
						lineHeight: '1.55'
					},
					'.cm-content': { padding: '10px 0', caretColor: '#C44536' },
					'.cm-cursor': { borderLeftColor: '#C44536', borderLeftWidth: '2px' },
					'.cm-gutters': {
						backgroundColor: '#F2EDE5',
						color: '#A39C92',
						borderRight: '1px solid #E5DFD4'
					},
					'.cm-activeLineGutter': { backgroundColor: '#E8E1D4', color: '#5C564E' },
					'.cm-activeLine': { backgroundColor: 'rgba(232, 225, 212, 0.4)' },
					'.cm-selectionBackground, &.cm-focused > .cm-scroller .cm-selectionBackground, .cm-content ::selection':
						{ background: 'rgba(196, 69, 54, 0.18) !important' },
					'&.cm-focused': { outline: 'none' },
					'.cm-line': { padding: '0 12px' }
				})
			]
		});
	});

	onDestroy(() => {
		view?.destroy();
	});

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
