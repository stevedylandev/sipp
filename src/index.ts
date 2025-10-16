import { serve } from "bun";
import index from "./index.html";
import snippet from "./snippet.html";
import { initDb, createSnippet, getSnippetByShortId } from "./db";

// Initialize the database
await initDb();

const server = serve({
	routes: {
		// Home page - create snippet form
		"/": index,
		// View snippet by shortId - this should be last to not override other routes
		"/s/:shortId": snippet,

		// Create a snippet
		"/api/snippets": {
			async POST(req) {
				try {
					const body = await req.json();

					// Validate required fields
					if (!body.name || !body.content) {
						return Response.json(
							{ error: "Missing required fields: name, content" },
							{ status: 400 },
						);
					}

					// Create the snippet
					const snippet = createSnippet({
						content: body.content,
						name: body.name,
					});

					return Response.json(snippet, { status: 201 });
				} catch (error) {
					return Response.json(
						{ error: `Failed to create snippet: ${error}` },
						{ status: 500 },
					);
				}
			},
		},
		// Fetch a snippet
		"/api/snippets/:shortId": {
			async GET(req) {
				try {
					const shortId = req.params.shortId;
					const snippet = getSnippetByShortId(shortId);

					if (!snippet) {
						return Response.json(
							{ error: "Snippet not found" },
							{ status: 404 },
						);
					}

					return Response.json(snippet);
				} catch (error) {
					return Response.json(
						{ error: `Failed to fetch snippet: ${error}` },
						{ status: 500 },
					);
				}
			},
		},
	},

	development: process.env.NODE_ENV !== "production" && {
		// Enable browser hot reloading in development
		hmr: true,

		// Echo console logs from the browser to the server
		console: true,
	},
});

console.log(`🚀 Server running at ${server.url}`);
