<script lang="ts">
	import { untrack } from 'svelte';
	import type { Connection, ConnectionInput, SshAuth } from '$lib/api';

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

{#snippet field(label: string)}
	<span class="block font-mono text-[10px] tracking-[0.22em] text-ink-faint uppercase">
		{label}
	</span>
{/snippet}

<form onsubmit={submit} class="space-y-7">
	<section class="space-y-4">
		<h3 class="font-display text-[18px] italic text-ink">mysql</h3>
		<div class="grid grid-cols-2 gap-x-4 gap-y-3">
			<label class="col-span-2 space-y-1">
				{@render field('name')}
				<input
					bind:value={name}
					required
					class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-display text-[18px] text-ink placeholder:text-ink-ghost focus:border-rust focus:outline-none"
				/>
			</label>
			<label class="col-span-2 space-y-1">
				{@render field('folder (optional, e.g. "prod/api")')}
				<input
					bind:value={folder}
					class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
				/>
			</label>
			<label class="col-span-1 space-y-1">
				{@render field('host')}
				<input
					bind:value={host}
					required
					class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
				/>
			</label>
			<label class="col-span-1 space-y-1">
				{@render field('port')}
				<input
					type="number"
					bind:value={port}
					required
					min="1"
					max="65535"
					class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
				/>
			</label>
			<label class="col-span-1 space-y-1">
				{@render field('username')}
				<input
					bind:value={username}
					required
					class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
				/>
			</label>
			<label class="col-span-1 space-y-1">
				{@render field('password')}
				<input
					type="password"
					bind:value={password}
					class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
				/>
			</label>
			<label class="col-span-2 space-y-1">
				{@render field('database (optional)')}
				<input
					bind:value={database}
					class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
				/>
			</label>
		</div>
		<p class="font-mono text-[10px] tracking-widest text-mustard uppercase">
			⚠ passwords are stored in plaintext sqlite (mvp). keychain later.
		</p>
	</section>

	<section class="space-y-3 border-t border-rule pt-5">
		<label class="flex cursor-pointer items-center gap-3">
			<input type="checkbox" bind:checked={sshEnabled} class="accent-rust" />
			<span class="font-display text-[18px] italic text-ink">ssh tunnel</span>
		</label>
		{#if sshEnabled}
			<div class="grid grid-cols-2 gap-x-4 gap-y-3 rounded-md border border-rule bg-cream-soft/40 p-4">
				<label class="col-span-2 space-y-1">
					{@render field('ssh host')}
					<input
						bind:value={sshHost}
						required={sshEnabled}
						class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
					/>
				</label>
				<label class="col-span-1 space-y-1">
					{@render field('ssh port')}
					<input
						type="number"
						bind:value={sshPort}
						min="1"
						max="65535"
						class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
					/>
				</label>
				<label class="col-span-1 space-y-1">
					{@render field('ssh user')}
					<input
						bind:value={sshUser}
						required={sshEnabled}
						class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
					/>
				</label>
				<label class="col-span-2 space-y-1">
					{@render field('auth method')}
					<select
						bind:value={sshMethod}
						class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
					>
						<option value="key">key</option>
						<option value="password">password</option>
						<option value="agent">agent (system ssh-agent)</option>
					</select>
				</label>
				{#if sshMethod === 'password'}
					<label class="col-span-2 space-y-1">
						{@render field('ssh password')}
						<input
							type="password"
							bind:value={sshPassword}
							required
							class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
						/>
					</label>
				{:else if sshMethod === 'key'}
					<label class="col-span-2 space-y-1">
						{@render field('private key path')}
						<input
							bind:value={sshKeyPath}
							required
							placeholder="~/.ssh/id_ed25519"
							class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink placeholder:text-ink-ghost focus:border-rust focus:outline-none"
						/>
					</label>
					<label class="col-span-2 space-y-1">
						{@render field('passphrase (optional)')}
						<input
							type="password"
							bind:value={sshPassphrase}
							class="block w-full border-b border-rule bg-transparent px-0 py-1.5 font-mono text-[13px] text-ink focus:border-rust focus:outline-none"
						/>
					</label>
				{/if}
			</div>
		{/if}
	</section>

	{#if error}
		<pre
			class="rounded bg-crimson-soft p-3 font-mono text-[12px] whitespace-pre-wrap text-crimson">{error}</pre>
	{/if}

	<div class="flex items-center justify-between border-t border-rule pt-5">
		<div>
			{#if onDelete}
				<button
					type="button"
					onclick={handleDelete}
					disabled={submitting}
					class="cursor-pointer rounded-md border border-crimson/40 px-3 py-1.5 font-mono text-[10px] tracking-[0.18em] text-crimson uppercase transition-colors hover:bg-crimson hover:text-cream disabled:cursor-not-allowed disabled:opacity-40"
				>
					delete
				</button>
			{/if}
		</div>
		<div class="flex gap-2">
			<button
				type="submit"
				disabled={submitting}
				class="cursor-pointer rounded-md bg-ink px-5 py-1.5 font-mono text-[10px] tracking-[0.18em] text-cream uppercase transition-colors hover:bg-rust disabled:cursor-not-allowed disabled:opacity-40"
			>
				{submitting ? '…' : submitLabel}
			</button>
		</div>
	</div>
</form>
