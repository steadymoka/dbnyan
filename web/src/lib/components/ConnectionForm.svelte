<script lang="ts">
	import { untrack } from 'svelte';
	import type { Connection, ConnectionInput, SshAuth } from '$lib/api';

	type Props = {
		initial?: Partial<Connection>;
		onSubmit: (input: ConnectionInput) => Promise<void>;
		onDelete?: () => Promise<void>;
		submitLabel?: string;
	};
	let { initial = {}, onSubmit, onDelete, submitLabel = 'Save' }: Props = $props();

	// Seed local state from `initial` once at mount. Further edits are local.
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

<form onsubmit={submit} class="space-y-6">
	<section class="space-y-3">
		<h3 class="text-xs font-semibold tracking-wide text-gray-500 uppercase">MySQL</h3>
		<div class="grid grid-cols-2 gap-3">
			<label class="col-span-2">
				<span class="text-xs text-gray-600">Name</span>
				<input bind:value={name} required class="w-full rounded border px-2 py-1" />
			</label>
			<label class="col-span-2">
				<span class="text-xs text-gray-600">Folder (optional, e.g. "prod/api")</span>
				<input bind:value={folder} class="w-full rounded border px-2 py-1" />
			</label>
			<label>
				<span class="text-xs text-gray-600">Host</span>
				<input bind:value={host} required class="w-full rounded border px-2 py-1" />
			</label>
			<label>
				<span class="text-xs text-gray-600">Port</span>
				<input
					type="number"
					bind:value={port}
					required
					min="1"
					max="65535"
					class="w-full rounded border px-2 py-1"
				/>
			</label>
			<label>
				<span class="text-xs text-gray-600">Username</span>
				<input bind:value={username} required class="w-full rounded border px-2 py-1" />
			</label>
			<label>
				<span class="text-xs text-gray-600">Password</span>
				<input type="password" bind:value={password} class="w-full rounded border px-2 py-1" />
			</label>
			<label class="col-span-2">
				<span class="text-xs text-gray-600">Database (optional)</span>
				<input bind:value={database} class="w-full rounded border px-2 py-1" />
			</label>
		</div>
		<p class="text-xs text-amber-700">
			⚠ Passwords are stored in plaintext SQLite for MVP. Keychain later.
		</p>
	</section>

	<section class="space-y-3">
		<label class="flex items-center gap-2">
			<input type="checkbox" bind:checked={sshEnabled} />
			<span class="text-xs font-semibold tracking-wide text-gray-500 uppercase">
				Use SSH tunnel
			</span>
		</label>
		{#if sshEnabled}
			<div class="grid grid-cols-2 gap-3 rounded border border-gray-200 p-3">
				<label class="col-span-2">
					<span class="text-xs text-gray-600">SSH host</span>
					<input
						bind:value={sshHost}
						required={sshEnabled}
						class="w-full rounded border px-2 py-1"
					/>
				</label>
				<label>
					<span class="text-xs text-gray-600">SSH port</span>
					<input
						type="number"
						bind:value={sshPort}
						min="1"
						max="65535"
						class="w-full rounded border px-2 py-1"
					/>
				</label>
				<label>
					<span class="text-xs text-gray-600">SSH user</span>
					<input
						bind:value={sshUser}
						required={sshEnabled}
						class="w-full rounded border px-2 py-1"
					/>
				</label>
				<label class="col-span-2">
					<span class="text-xs text-gray-600">Auth method</span>
					<select bind:value={sshMethod} class="w-full rounded border px-2 py-1">
						<option value="key">key</option>
						<option value="password">password</option>
						<option value="agent">agent (system ssh-agent)</option>
					</select>
				</label>
				{#if sshMethod === 'password'}
					<label class="col-span-2">
						<span class="text-xs text-gray-600">SSH password</span>
						<input
							type="password"
							bind:value={sshPassword}
							required
							class="w-full rounded border px-2 py-1"
						/>
					</label>
				{:else if sshMethod === 'key'}
					<label class="col-span-2">
						<span class="text-xs text-gray-600">Private key path</span>
						<input
							bind:value={sshKeyPath}
							required
							placeholder="~/.ssh/id_ed25519"
							class="w-full rounded border px-2 py-1"
						/>
					</label>
					<label class="col-span-2">
						<span class="text-xs text-gray-600">Passphrase (optional)</span>
						<input
							type="password"
							bind:value={sshPassphrase}
							class="w-full rounded border px-2 py-1"
						/>
					</label>
				{/if}
			</div>
		{/if}
	</section>

	{#if error}
		<pre class="rounded bg-red-50 p-3 text-sm whitespace-pre-wrap text-red-700">{error}</pre>
	{/if}

	<div class="flex items-center justify-between">
		<div>
			{#if onDelete}
				<button
					type="button"
					onclick={handleDelete}
					disabled={submitting}
					class="rounded border border-red-300 px-3 py-1 text-sm text-red-700 hover:bg-red-50 disabled:opacity-50"
				>
					Delete
				</button>
			{/if}
		</div>
		<div class="flex gap-2">
			<a href="/" class="rounded border px-3 py-1 text-sm">Cancel</a>
			<button
				type="submit"
				disabled={submitting}
				class="rounded bg-blue-600 px-4 py-1 text-sm text-white disabled:opacity-50"
			>
				{submitting ? '...' : submitLabel}
			</button>
		</div>
	</div>
</form>
