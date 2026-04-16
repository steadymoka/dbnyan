<script lang="ts">
	import { untrack } from 'svelte';
	import type { Connection, ConnectionInput, SshAuth } from '$lib/api';
	import { COLOR_LIST } from '$lib/colors';

	type Props = {
		initial?: Partial<Connection>;
		onSubmit: (input: ConnectionInput) => Promise<void>;
		onDelete?: () => Promise<void>;
		submitLabel?: string;
	};
	let { initial = {}, onSubmit, onDelete, submitLabel = 'save' }: Props = $props();

	const seed = untrack(() => initial);

	let name = $state(seed.name ?? '');
	let host = $state(seed.host ?? '');
	let port = $state<number>(seed.port ?? 3306);
	let username = $state(seed.username ?? '');
	let password = $state(seed.password ?? '');
	let database = $state(seed.database ?? '');
	let folder = $state(seed.folder ?? '');
	let color = $state<string | null>(seed.color ?? null);

	let sshEnabled = $state(!!seed.ssh);
	let sshHost = $state(seed.ssh?.host ?? '');
	let sshPort = $state<number>(seed.ssh?.port ?? 22);
	let sshUser = $state(seed.ssh?.user ?? '');
	let sshMethod = $state<'password' | 'key' | 'agent'>(seed.ssh?.auth.method ?? 'key');
	let sshPassword = $state(seed.ssh?.auth.method === 'password' ? seed.ssh.auth.password : '');
	let sshKeyPath = $state(seed.ssh?.auth.method === 'key' ? seed.ssh.auth.key_path : '');
	let sshPassphrase = $state(
		seed.ssh?.auth.method === 'key' ? (seed.ssh.auth.passphrase ?? '') : ''
	);

	let submitting = $state(false);
	let error = $state<string | null>(null);

	function buildAuth(): SshAuth {
		if (sshMethod === 'password') return { method: 'password', password: sshPassword };
		if (sshMethod === 'key')
			return {
				method: 'key',
				key_path: sshKeyPath,
				passphrase: sshPassphrase || undefined
			};
		return { method: 'agent' };
	}

	async function submit(e: Event) {
		e.preventDefault();
		submitting = true;
		error = null;
		try {
			const input: ConnectionInput = {
				name,
				host,
				port,
				username,
				password: password || undefined,
				database: database || undefined,
				folder: folder || undefined,
				color: color ?? undefined,
				ssh: sshEnabled
					? { host: sshHost, port: sshPort, user: sshUser, auth: buildAuth() }
					: undefined
			};
			await onSubmit(input);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}

	async function handleDelete() {
		if (!onDelete) return;
		if (!confirm('Delete this connection?')) return;
		submitting = true;
		error = null;
		try {
			await onDelete();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}
</script>

<form onsubmit={submit} class="space-y-5">
	<div class="grid grid-cols-2 gap-x-4 gap-y-4">
		<label class="col-span-2 space-y-1">
			<span class="block text-[11px] text-ink-muted">Name</span>
			<input
				bind:value={name}
				required
				placeholder="prod api"
				class="block w-full border-b border-rule bg-transparent py-1 font-sans text-[14px] text-ink placeholder:text-ink-ghost focus:border-rust focus:outline-none"
			/>
		</label>
		<label class="col-span-2 space-y-1">
			<span class="block text-[11px] text-ink-muted">Folder <span class="text-ink-faint">— optional, e.g. <span class="font-mono text-[11px]">prod/api</span> for nested</span></span>
			<input
				bind:value={folder}
				class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
			/>
		</label>
		<div class="col-span-2 space-y-1.5">
			<span class="block text-[11px] text-ink-muted">Color <span class="text-ink-faint">— optional</span></span>
			<div class="flex flex-wrap items-center gap-2 pt-1">
				<button
					type="button"
					class="grid h-6 w-6 place-items-center rounded-full border border-rule bg-cream transition-transform hover:scale-110 {color ===
					null
						? 'ring-2 ring-ink ring-offset-2 ring-offset-cream'
						: ''}"
					title="no color"
					onclick={() => (color = null)}
				>
					<span class="block h-2 w-2 rounded-full bg-ink-ghost"></span>
				</button>
				{#each COLOR_LIST as [n, hex] (n)}
					<button
						type="button"
						class="h-6 w-6 cursor-pointer rounded-full transition-transform hover:scale-110 {color ===
						n
							? 'ring-2 ring-ink ring-offset-2 ring-offset-cream'
							: ''}"
						style="background: {hex}"
						title={n}
						onclick={() => (color = n)}
						aria-label={n}
					></button>
				{/each}
			</div>
		</div>
		<label class="col-span-1 space-y-1">
			<span class="block text-[11px] text-ink-muted">Host</span>
			<input
				bind:value={host}
				required
				class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
			/>
		</label>
		<label class="col-span-1 space-y-1">
			<span class="block text-[11px] text-ink-muted">Port</span>
			<input
				type="number"
				bind:value={port}
				required
				min="1"
				max="65535"
				class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
			/>
		</label>
		<label class="col-span-1 space-y-1">
			<span class="block text-[11px] text-ink-muted">Username</span>
			<input
				bind:value={username}
				required
				class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
			/>
		</label>
		<label class="col-span-1 space-y-1">
			<span class="block text-[11px] text-ink-muted">Password</span>
			<input
				type="password"
				bind:value={password}
				class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
			/>
		</label>
		<label class="col-span-2 space-y-1">
			<span class="block text-[11px] text-ink-muted">Database <span class="text-ink-faint">— optional</span></span>
			<input
				bind:value={database}
				class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
			/>
		</label>
	</div>

	<p class="text-[11px] text-mustard">
		⚠ Passwords are stored in plaintext SQLite for now (Keychain later).
	</p>

	<div class="border-t border-rule pt-4">
		<label class="flex cursor-pointer items-center gap-2">
			<input type="checkbox" bind:checked={sshEnabled} class="accent-rust" />
			<span class="text-[12.5px] text-ink">Use SSH tunnel</span>
		</label>
		{#if sshEnabled}
			<div class="mt-3 grid grid-cols-2 gap-x-4 gap-y-3 rounded-md bg-cream-soft/50 p-4">
				<label class="col-span-2 space-y-1">
					<span class="block text-[11px] text-ink-muted">SSH host</span>
					<input
						bind:value={sshHost}
						required={sshEnabled}
						class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
					/>
				</label>
				<label class="col-span-1 space-y-1">
					<span class="block text-[11px] text-ink-muted">Port</span>
					<input
						type="number"
						bind:value={sshPort}
						min="1"
						max="65535"
						class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
					/>
				</label>
				<label class="col-span-1 space-y-1">
					<span class="block text-[11px] text-ink-muted">User</span>
					<input
						bind:value={sshUser}
						required={sshEnabled}
						class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
					/>
				</label>
				<label class="col-span-2 space-y-1">
					<span class="block text-[11px] text-ink-muted">Auth</span>
					<select
						bind:value={sshMethod}
						class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
					>
						<option value="key">key</option>
						<option value="password">password</option>
						<option value="agent">agent (system ssh-agent)</option>
					</select>
				</label>
				{#if sshMethod === 'password'}
					<label class="col-span-2 space-y-1">
						<span class="block text-[11px] text-ink-muted">SSH password</span>
						<input
							type="password"
							bind:value={sshPassword}
							required
							class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
						/>
					</label>
				{:else if sshMethod === 'key'}
					<label class="col-span-2 space-y-1">
						<span class="block text-[11px] text-ink-muted">Private key path</span>
						<input
							bind:value={sshKeyPath}
							required
							placeholder="~/.ssh/id_ed25519"
							class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink placeholder:text-ink-ghost focus:border-rust focus:outline-none"
						/>
					</label>
					<label class="col-span-2 space-y-1">
						<span class="block text-[11px] text-ink-muted">Passphrase <span class="text-ink-faint">— optional</span></span>
						<input
							type="password"
							bind:value={sshPassphrase}
							class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
						/>
					</label>
				{/if}
			</div>
		{/if}
	</div>

	{#if error}
		<pre
			class="rounded bg-crimson-soft p-3 font-mono text-[12px] whitespace-pre-wrap text-crimson">{error}</pre>
	{/if}

	<div class="flex items-center justify-between border-t border-rule pt-4">
		<div>
			{#if onDelete}
				<button
					type="button"
					onclick={handleDelete}
					disabled={submitting}
					class="cursor-pointer rounded-md border border-crimson/40 px-3 py-1.5 text-[12px] text-crimson transition-colors hover:bg-crimson hover:text-cream disabled:cursor-not-allowed disabled:opacity-40"
				>
					Delete
				</button>
			{/if}
		</div>
		<button
			type="submit"
			disabled={submitting}
			class="cursor-pointer rounded-md bg-ink px-5 py-1.5 text-[12px] font-medium text-cream transition-colors hover:bg-rust disabled:cursor-not-allowed disabled:opacity-40"
		>
			{submitting ? '…' : submitLabel}
		</button>
	</div>
</form>
