import { marked, type Token, type Tokens } from 'marked';
import DOMPurify from 'dompurify';

export type Segment =
	| { kind: 'html'; html: string }
	| { kind: 'sql'; sql: string };

/** Split markdown into sanitized-HTML + raw-SQL segments, so SQL blocks can
 * get Svelte-rendered buttons (copy / use →) without resorting to {@html}. */
export function parse(text: string): Segment[] {
	const tokens = marked.lexer(text);
	const out: Segment[] = [];
	let buf: Token[] = [];

	const flush = () => {
		if (buf.length === 0) return;
		const html = DOMPurify.sanitize(marked.parser(buf));
		out.push({ kind: 'html', html });
		buf = [];
	};

	for (const tok of tokens) {
		if (tok.type === 'code' && /^sql$/i.test((tok as Tokens.Code).lang ?? '')) {
			flush();
			out.push({ kind: 'sql', sql: (tok as Tokens.Code).text });
		} else {
			buf.push(tok);
		}
	}
	flush();
	return out;
}
