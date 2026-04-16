export type SshAuth =
	| { method: 'password'; password: string }
	| { method: 'key'; key_path: string; passphrase?: string }
	| { method: 'agent' };

export type SshConfig = {
	host: string;
	port: number;
	user: string;
	auth: SshAuth;
};

export type SsmConfig = {
	target: string;
	region?: string;
	profile?: string;
};

export type Connection = {
	id: string;
	name: string;
	driver: string;
	host: string;
	port: number;
	username: string;
	password?: string;
	database?: string;
	folder?: string;
	color?: string;
	ssh?: SshConfig;
	aws_ssm?: SsmConfig;
	created_at: string;
	updated_at: string;
};

export type ConnectionInput = {
	name: string;
	host: string;
	port: number;
	username: string;
	password?: string;
	database?: string;
	folder?: string;
	color?: string;
	ssh?: SshConfig;
	aws_ssm?: SsmConfig;
};

export type TableInfo = { name: string; kind: string };
export type ColumnInfo = {
	name: string;
	data_type: string;
	nullable: boolean;
	default: string | null;
	key: string | null;
	extra: string | null;
};
export type RowSet = {
	columns: string[];
	rows: unknown[][];
	limit: number;
	returned: number;
};

export type QueryResult =
	| { kind: 'rows'; columns: string[]; rows: unknown[][]; returned: number; duration_ms: number }
	| {
			kind: 'affected';
			rows_affected: number;
			last_insert_id: number;
			duration_ms: number;
	  };

export type HistoryEntry = {
	id: string;
	connection_id: string;
	database_name: string | null;
	sql: string;
	success: boolean;
	error: string | null;
	rows_affected: number | null;
	rows_returned: number | null;
	duration_ms: number;
	executed_at: string;
};

async function handle(res: Response): Promise<unknown> {
	if (!res.ok) {
		const text = await res.text();
		let msg = text;
		try {
			msg = JSON.parse(text).error ?? text;
		} catch {
			/* keep raw */
		}
		throw new Error(`${res.status}: ${msg}`);
	}
	return res.status === 204 ? null : res.json();
}

const jsonInit = (method: string, body: unknown): RequestInit => ({
	method,
	headers: { 'Content-Type': 'application/json' },
	body: JSON.stringify(body)
});

const enc = encodeURIComponent;

export const api = {
	connections: {
		list: () => fetch('/api/connections').then(handle) as Promise<Connection[]>,
		get: (id: string) => fetch(`/api/connections/${enc(id)}`).then(handle) as Promise<Connection>,
		create: (input: ConnectionInput) =>
			fetch('/api/connections', jsonInit('POST', input)).then(handle) as Promise<Connection>,
		update: (id: string, input: ConnectionInput) =>
			fetch(`/api/connections/${enc(id)}`, jsonInit('PATCH', input)).then(
				handle
			) as Promise<Connection>,
		delete: (id: string) =>
			fetch(`/api/connections/${enc(id)}`, { method: 'DELETE' }).then(handle) as Promise<null>
	},
	databases: {
		list: (id: string) =>
			fetch(`/api/connections/${enc(id)}/databases`).then(handle) as Promise<string[]>,
		tables: (id: string, db: string) =>
			fetch(`/api/connections/${enc(id)}/databases/${enc(db)}/tables`).then(
				handle
			) as Promise<TableInfo[]>,
		schema: (id: string, db: string, table: string) =>
			fetch(
				`/api/connections/${enc(id)}/databases/${enc(db)}/tables/${enc(table)}/schema`
			).then(handle) as Promise<ColumnInfo[]>,
		rows: (id: string, db: string, table: string, limit = 200) =>
			fetch(
				`/api/connections/${enc(id)}/databases/${enc(db)}/tables/${enc(table)}/rows?limit=${limit}`
			).then(handle) as Promise<RowSet>
	},
	queries: {
		run: (id: string, sql: string, database?: string) =>
			fetch(`/api/connections/${enc(id)}/query`, jsonInit('POST', { sql, database })).then(
				handle
			) as Promise<QueryResult>
	},
	history: {
		list: (id: string, limit = 50) =>
			fetch(`/api/connections/${enc(id)}/history?limit=${limit}`).then(
				handle
			) as Promise<HistoryEntry[]>,
		delete: (id: string, hid: string) =>
			fetch(`/api/connections/${enc(id)}/history/${enc(hid)}`, { method: 'DELETE' }).then(
				handle
			) as Promise<{ deleted: boolean }>,
		clear: (id: string) =>
			fetch(`/api/connections/${enc(id)}/history`, { method: 'DELETE' }).then(handle) as Promise<{
				cleared: number;
			}>
	},
	sessions: {
		close: (id: string) =>
			fetch(`/api/connections/${enc(id)}/session`, { method: 'DELETE' }).then(
				handle
			) as Promise<{ closed: boolean }>
	},
	chat: {
		send: (
			id: string,
			body: { message: string; session_id?: string | null; database?: string | null }
		) =>
			fetch(`/api/connections/${enc(id)}/chat`, jsonInit('POST', body)).then(handle) as Promise<{
				session_id: string | null;
				text: string;
				is_error: boolean;
				duration_ms: number | null;
			}>
	}
};
