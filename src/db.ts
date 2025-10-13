import { Database } from "bun:sqlite";

const db = new Database("sipps.sqlite");

// URL-safe characters for short ID generation (similar to nanoid)
const ALPHABET =
	"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

// Generate a short ID using Bun's crypto (similar to nanoid)
export function generateShortId(length: number = 10): string {
	const bytes = new Uint8Array(length);
	crypto.getRandomValues(bytes);

	let id = "";
	for (let i = 0; i < length; i++) {
		id += ALPHABET[bytes[i] % ALPHABET.length];
	}
	return id;
}

export interface Snippet {
	id?: number;
	shortId: string;
	content: string;
	language: string;
	name: string;
}

export async function initDb() {
	db.query(`
  CREATE TABLE IF NOT EXISTS snippets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shortId TEXT NOT NULL,
    content TEXT NOT NULL,
    name TEXT NOT NULL,
    language TEXT NOT NULL
  )
`).run();
}

// Create a new snippet
export function createSnippet(
	snippet: Omit<Snippet, "id" | "shortId"> & { shortId?: string },
): Snippet {
	// Generate a short ID if not provided
	const shortId = snippet.shortId || generateShortId();

	const query = db.query(
		"INSERT INTO snippets (shortId, content, name, language) VALUES (?, ?, ?, ?) RETURNING *",
	);
	return query.get(
		shortId,
		snippet.content,
		snippet.name,
		snippet.language,
	) as Snippet;
}

// Read a snippet by ID
export function getSnippetById(id: number): Snippet | null {
	const query = db.query("SELECT * FROM snippets WHERE id = ?");
	return (query.get(id) as Snippet) || null;
}

// Read a snippet by shortId
export function getSnippetByShortId(shortId: string): Snippet | null {
	const query = db.query("SELECT * FROM snippets WHERE shortId = ?");
	return (query.get(shortId) as Snippet) || null;
}

// Read all snippets
export function getAllSnippets(): Snippet[] {
	const query = db.query("SELECT * FROM snippets ORDER BY id DESC");
	return query.all() as Snippet[];
}

// Update a snippet
export function updateSnippet(
	id: number,
	updates: Partial<Omit<Snippet, "id">>,
): Snippet | null {
	const existing = getSnippetById(id);
	if (!existing) return null;

	const fields: string[] = [];
	const values: any[] = [];

	if (updates.shortId !== undefined) {
		fields.push("shortId = ?");
		values.push(updates.shortId);
	}
	if (updates.content !== undefined) {
		fields.push("content = ?");
		values.push(updates.content);
	}
	if (updates.name !== undefined) {
		fields.push("name = ?");
		values.push(updates.name);
	}
	if (updates.language !== undefined) {
		fields.push("language = ?");
		values.push(updates.language);
	}

	if (fields.length === 0) return existing;

	values.push(id);
	const query = db.query(
		`UPDATE snippets SET ${fields.join(", ")} WHERE id = ? RETURNING *`,
	);
	return query.get(...values) as Snippet;
}

// Delete a snippet
export function deleteSnippet(id: number): boolean {
	const query = db.query("DELETE FROM snippets WHERE id = ?");
	const result = query.run(id);
	return result.changes > 0;
}

// Delete a snippet by shortId
export function deleteSnippetByShortId(shortId: string): boolean {
	const query = db.query("DELETE FROM snippets WHERE shortId = ?");
	const result = query.run(shortId);
	return result.changes > 0;
}
