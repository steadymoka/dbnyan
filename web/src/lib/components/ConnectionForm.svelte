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

	type Method = 'direct' | 'ssh' | 'ssm';
	let method = $state<Method>(seed.ssh ? 'ssh' : seed.aws_ssm ? 'ssm' : 'direct');

	// SSH state
	let sshHost = $state(seed.ssh?.host ?? '');
	let sshPort = $state<number>(seed.ssh?.port ?? 22);
	let sshUser = $state(seed.ssh?.user ?? '');
	let sshMethod = $state<'password' | 'key' | 'agent'>(seed.ssh?.auth.method ?? 'key');
	let sshPassword = $state(seed.ssh?.auth.method === 'password' ? seed.ssh.auth.password : '');
	let sshKeyPath = $state(seed.ssh?.auth.method === 'key' ? seed.ssh.auth.key_path : '');
	let sshPassphrase = $state(
		seed.ssh?.auth.method === 'key' ? (seed.ssh.auth.passphrase ?? '') : ''
	);

	// SSM state
	let ssmTarget = $state(seed.aws_ssm?.target ?? '');
	let ssmRegion = $state(seed.aws_ssm?.region ?? '');
	let ssmProfile = $state(seed.aws_ssm?.profile ?? '');
	let ssmLocalPortStr = $state(seed.aws_ssm?.local_port?.toString() ?? '');

	let submitting = $state(false);
	let error = $state<string | null>(null);

	function buildSshAuth(): SshAuth {
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
				ssh:
					method === 'ssh'
						? { host: sshHost, port: sshPort, user: sshUser, auth: buildSshAuth() }
						: undefined,
				aws_ssm:
					method === 'ssm'
						? {
								target: ssmTarget,
								region: ssmRegion || undefined,
								profile: ssmProfile || undefined,
								local_port: ssmLocalPortStr.trim()
									? Number(ssmLocalPortStr.trim())
									: undefined
							}
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
				class="block w-full border-b border-rule bg-transparent py-1 text-[14px] text-ink placeholder:text-ink-ghost focus:border-rust focus:outline-none"
			/>
		</label>
		<label class="col-span-2 space-y-1">
			<span class="block text-[11px] text-ink-muted">Folder <span class="text-ink-faint">— optional, e.g. <span class="font-mono text-[11px]">prod/api</span> for nested</span></span>
			<input
				bind:value={folder}
				class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink focus:border-rust focus:outline-none"
			/>
		</label>
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
		<div class="col-span-2 space-y-1.5">
			<span class="block text-[11px] text-ink-muted">Color <span class="text-ink-faint">— optional</span></span>
			<div class="flex flex-wrap items-center gap-2 pt-1">
				<button
					type="button"
					class="grid h-6 w-6 cursor-pointer place-items-center rounded-full border border-rule bg-cream transition-transform hover:scale-110 {color ===
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
	</div>

	<p class="text-[11px] text-mustard">
		⚠ Passwords are stored in plaintext SQLite for now (Keychain later).
	</p>

	<div class="space-y-3 border-t border-rule pt-4">
		<div>
			<span class="block text-[11px] text-ink-muted">Connection method</span>
			<div class="mt-1.5 flex gap-1.5">
				{#each [{ id: 'direct', label: 'Direct' }, { id: 'ssh', label: 'SSH tunnel' }, { id: 'ssm', label: 'AWS SSM' }] as opt (opt.id)}
					<button
						type="button"
						class="cursor-pointer rounded-md border px-3 py-1 text-[12px] transition-colors {method ===
						opt.id
							? 'border-rust bg-rust-soft/40 text-rust'
							: 'border-rule text-ink-muted hover:border-ink-faint hover:text-ink'}"
						onclick={() => (method = opt.id as Method)}
					>
						{opt.label}
					</button>
				{/each}
			</div>
		</div>

		{#if method === 'ssh'}
			<div class="grid grid-cols-2 gap-x-4 gap-y-3 rounded-md bg-cream-soft/50 p-4">
				<label class="col-span-2 space-y-1">
					<span class="block text-[11px] text-ink-muted">SSH host</span>
					<input
						bind:value={sshHost}
						required
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
						required
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
		{:else if method === 'ssm'}
			<div class="grid grid-cols-2 gap-x-4 gap-y-3 rounded-md bg-cream-soft/50 p-4">
				<label class="col-span-2 space-y-1">
					<span class="block text-[11px] text-ink-muted">EC2 instance id <span class="text-ink-faint">— SSM target</span></span>
					<input
						bind:value={ssmTarget}
						required
						placeholder="i-00e64f3807d1c2061"
						class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink placeholder:text-ink-ghost focus:border-rust focus:outline-none"
					/>
				</label>
				<label class="col-span-1 space-y-1">
					<span class="block text-[11px] text-ink-muted">Region <span class="text-ink-faint">— optional</span></span>
					<input
						bind:value={ssmRegion}
						placeholder="ap-northeast-2"
						class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink placeholder:text-ink-ghost focus:border-rust focus:outline-none"
					/>
				</label>
				<label class="col-span-1 space-y-1">
					<span class="block text-[11px] text-ink-muted">Profile <span class="text-ink-faint">— optional</span></span>
					<input
						bind:value={ssmProfile}
						placeholder="default"
						class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink placeholder:text-ink-ghost focus:border-rust focus:outline-none"
					/>
				</label>
				<label class="col-span-2 space-y-1">
					<span class="block text-[11px] text-ink-muted">Local port <span class="text-ink-faint">— optional; fixed if set, random otherwise. Use to share with other tools.</span></span>
					<input
						type="number"
						bind:value={ssmLocalPortStr}
						placeholder="random"
						min="1"
						max="65535"
						class="block w-full border-b border-rule bg-transparent py-1 font-mono text-[12.5px] text-ink placeholder:text-ink-ghost focus:border-rust focus:outline-none"
					/>
				</label>
				<p class="col-span-2 text-[11px] text-ink-faint">
					Host/Port above describe the forward destination (e.g. RDS endpoint :3306).
					Requires <span class="font-mono">aws</span> CLI + <span class="font-mono">session-manager-plugin</span> on this machine.
				</p>
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
