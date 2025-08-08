package main

import (
	"context"
	"encoding/json"
	"html/template"
	"io"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/sashabaranov/go-openai"       // OpenAI client, compatible with Ollama's API
	"github.com/sourcenetwork/defradb/client" // DefraDB client
	"github.com/sourcenetwork/defradb/node"   // DefraDB node
)

// This example, based on `github.com/chromem-go/examples/rag-wikipedia-ollama`,
// demonstrates how to use DefraDB as a vector database for a
// Retrieval-Augmented Generation (RAG) pipeline.
//
// The process is as follows:
// 1. Ask an LLM a question it likely doesn't know the answer to.
// 2. Load a set of documents into DefraDB. DefraDB will automatically create
//    vector embeddings for a specified field in these documents.
// 3. When a user asks a question, first convert the question into a vector embedding.
// 4. Use this query vector to search DefraDB for the most semantically similar
//    documents from the knowledge base.
// 5. Provide these retrieved documents as context to the LLM along with the
//    original question.
// 6. The LLM, now equipped with relevant information, can provide a much more
//    accurate and informed answer.
//
// Prerequisites:
// - An Ollama instance running locally. See: https://ollama.com/
// - The 'nomic-embed-text' model pulled in Ollama: `ollama pull nomic-embed-text`
// - The 'gemma:2b' model pulled in Ollama: `ollama pull gemma:2b`
// - A `wiki.jsonl` file in the same directory with sample data.

const (
	// We use a local LLM running in Ollama to answer the question.
	// Ollama provides an OpenAI-compatible API endpoint.
	ollamaBaseURL = "http://localhost:11434/v1"

	// We use Google's Gemma (2B), a small but capable model that runs well on
	// consumer hardware. It's fast and effective for this RAG use case.
	// Model details: https://huggingface.co/google/gemma-2b
	llmModel = "gemma:2b"

	// The question we want to ask. It's specific enough that a general-purpose
	// small LLM is unlikely to know the answer.
	question = "When did the Monarch Company exist?"

	// We use a local LLM running in Ollama for creating the embeddings.
	// This model is specifically designed for generating high-quality embeddings.
	// Model details: https://huggingface.co/nomic-ai/nomic-embed-text-v1.5
	embeddingModel = "nomic-embed-text"
)

func main() {
	ctx := context.Background()

	// // It can take a few seconds for Ollama to load a model into memory for the
	// // first time. We send a simple request to "warm it up" and ensure it's
	// // ready before we start the main workflow.

	// --- Step 1: Ask the LLM without RAG ---
	// We first ask the LLM our question directly to demonstrate that without any
	// external knowledge, it's unable to provide a correct answer.
	log.Println("================================================================================")
	log.Println("Asking the LLM without providing any external knowledge (no RAG)")
	log.Println("================================================================================")
	log.Println("Question: " + question)
	log.Println("Asking LLM...")
	reply := askLLM(ctx, nil, question)
	log.Printf("Initial reply from the LLM: \"%s\"\n\n", reply)

	// --- Step 2: Set up DefraDB and load knowledge base ---
	// Now, we'll use DefraDB to store our knowledge base and retrieve relevant
	// context for our question.
	log.Println("================================================================================")
	log.Println("Set up DefraDB and load knowledge base")
	log.Println("================================================================================")

	// For this example, we'll use an in-memory instance of DefraDB.
	// For production use, you would configure it with persistent storage like Badger.
	// We also disable the P2P and API servers as we are using DefraDB embedded
	// in our application.
	log.Println("Setting up DefraDB...")
	db, err := node.New(ctx, node.WithBadgerInMemory(true), node.WithDisableAPI(true), node.WithDisableP2P(true))
	if err != nil {
		// For a real application, more robust error handling would be needed.
		log.Fatalf("Failed to create DefraDB node: %v", err)
	}
	defer db.Close(ctx)
	err = db.Start(ctx)
	if err != nil {
		log.Fatalf("Failed to start DefraDB node: %v", err)
	}

	// We define a schema for our data. A schema in DefraDB is similar to a table
	// definition in a traditional database.
	// The key part for RAG is the `@embedding` directive.
	// - `text_v: [Float32!]`: This defines a field to store the vector embedding.
	// - `@embedding(...)`: This directive tells DefraDB to automatically generate
	//   an embedding for this field.
	// - `fields: ["text"]`: Specifies that the embedding should be generated from
	//   the content of the "text" field.
	// - `provider: "ollama"`: The embedding provider to use.
	// - `model: "nomic-embed-text"`: The specific model to use for generating embeddings.
	log.Println("Adding 'Wiki' collection schema to DefraDB...")
	_, err = db.DB.AddSchema(ctx, `type Wiki {
		text: String
		category: String
		text_v: [Float32!] @embedding(fields: ["text"], provider: "ollama", model: "nomic-embed-text")
	}`)
	if err != nil {
		// This might fail if the schema is already added. In a real app, you'd
		// check for this. For this example, we assume a clean start.
		log.Fatalf("Failed to add schema: %v", err)
	}

	// We'll load our knowledge base from a local JSONL file. Each line in the
	// file represents a document (a small Wiki article in this case).
	f, err := os.Open("wiki.jsonl")
	if err != nil {
		log.Fatalf("Failed to open wiki.jsonl. Make sure the file exists. Error: %v", err)
	}
	defer f.Close()

	d := json.NewDecoder(f)
	log.Println("Reading JSON lines from wiki.jsonl and adding to the 'Wiki' collection...")
	for {
		var article struct {
			Text     string `json:"text"`
			Category string `json:"category"`
		}
		err := d.Decode(&article)
		if err == io.EOF {
			break // Reached end of file
		} else if err != nil {
			log.Fatalf("Failed to decode JSON line: %v", err)
		}

		// The 'nomic-embed-text' model performs better when a specific prefix is
		// added to differentiate between documents for storage ("search_document")
		// and queries for retrieval ("search_query"). This is a model-specific
		// requirement and not needed for all embedding models.
		// We add the prefix here before storing the document.
		contentWithPrefix := "search_document: " + article.Text

		// We use a GraphQL mutation to create a new document in our 'Wiki' collection.
		// The `input` argument for a `create` mutation is a document (can also be a list of documents).
		// When this mutation is executed, DefraDB will:
		// 1. Take the value of `text`.
		// 2. Send it to the configured Ollama model (`nomic-embed-text`).
		// 3. Store the resulting vector embedding in the `text_v` field.
		//
		// Note that we could also generate the embedding manually and assign it to `text_v`.
		createResult := db.DB.ExecRequest(
			ctx,
			`mutation CreateWiki($input: [WikiMutationInputArg!]!) {
				create_Wiki(input: $input) {
					_docID
				}
			}`,
			client.WithVariables(map[string]any{
				// Since we are creating one document at a time, we provide
				// a single document object.
				"input": map[string]any{
					"text":     contentWithPrefix,
					"category": article.Category,
				},
			}),
		)
		if len(createResult.GQL.Errors) > 0 {
			// Log all errors for debugging.
			for _, gqlErr := range createResult.GQL.Errors {
				log.Printf("GraphQL error on create: %v\n", gqlErr)
			}
			log.Fatalf("Failed to create document in DefraDB.")
		}
	}
	log.Println("Finished loading data into DefraDB.")

	// --- Step 3: Perform Similarity Search to Retrieve Context ---
	log.Println("================================================================================")
	log.Println("Retrieving relevant documents from DefraDB")
	log.Println("================================================================================")
	start := time.Now()

	// As mentioned before, the 'nomic-embed-text' model requires a specific
	// prefix for queries.
	queryWithPrefix := "search_query: " + question

	// We need to manually create an embedding for our query. We use the same
	// model and provider that we configured in the DefraDB schema.
	//
	// Note that automatically generating the query embedding is on the development roadmap.
	log.Println("Creating embedding for the query...")
	openAIClient := openai.NewClientWithConfig(openai.ClientConfig{
		BaseURL:    ollamaBaseURL,
		HTTPClient: http.DefaultClient,
	})
	embeddingResp, err := openAIClient.CreateEmbeddings(ctx, openai.EmbeddingRequest{
		Input: []string{queryWithPrefix},
		Model: embeddingModel,
	})
	if err != nil {
		log.Fatalf("Failed to create query embedding: %v", err)
	}

	// Now we execute a GraphQL query to find the most relevant documents.
	// - `_similarity`: This is a special DefraDB operator that calculates the
	//   cosine similarity between a document's vector field (`text_v`) and a
	//   provided vector (`$queryVector`).
	// - `sim: _similarity(...)`: We alias the result of the similarity calculation
	//   to a field named `sim`.
	// - `order: {_alias: {sim: DESC}}`: We order the results by the similarity
	//   score in descending order, so the most relevant documents come first.
	// - `limit: 2`: We ask for the top 2 most similar documents.
	// - `filter: {_alias: {sim: {_gt: 0.63}}}`: We filter out results with a
	//   similarity score below a certain threshold to ensure relevance. This
	//   threshold may need tuning based on your data and use case.
	log.Println("Querying DefraDB for similar documents...")
	queryResult := db.DB.ExecRequest(
		ctx,
		`query Search($queryVector: [Float32!]!) {
			Wiki(
				filter: {_alias: {sim: {_gt: 0.63}}},
				limit: 2,
				order: {_alias: {sim: DESC}}
			) {
				text
				sim: _similarity(text_v: {vector: $queryVector})
			}
		}`,
		client.WithVariables(map[string]any{
			"queryVector": embeddingResp.Data[0].Embedding,
		}),
	)
	if len(queryResult.GQL.Errors) > 0 {
		for _, gqlErr := range queryResult.GQL.Errors {
			log.Printf("GraphQL error on query: %v\n", gqlErr)
		}
		log.Fatalf("Failed to query documents from DefraDB.")
	}

	log.Printf("Search (incl. query embedding) took %s\n", time.Since(start))

	resultData, ok := queryResult.GQL.Data.(map[string]any)["Wiki"].([]map[string]any)
	if !ok || len(resultData) == 0 {
		log.Println("No relevant documents found in the knowledge base.")
		return
	}

	// Print the retrieved documents and their similarity to the question.
	log.Println("Found relevant documents:")
	var contexts []string
	for i, res := range resultData {
		// Remember to remove the "search_document: " prefix we added earlier
		// before passing the text to the LLM.
		content := strings.TrimPrefix(res["text"].(string), "search_document: ")
		log.Printf(" - Document %d (similarity: %.4f): \"%s...\"\n", i+1, res["sim"], content[:100])
		contexts = append(contexts, content)
	}

	// --- Step 4: Ask the LLM with RAG ---
	// Now we ask the same question again, but this time we provide the retrieved
	// documents as context to the LLM.
	log.Println("================================================================================")
	log.Println("Asking the LLM with retrieved knowledge (with RAG)")
	log.Println("================================================================================")
	log.Println("Asking LLM with augmented question...")
	reply = askLLM(ctx, contexts, question)
	log.Printf("Reply after augmenting the question with knowledge: \"%s\"\n", reply)

	/* Output (can differ slightly on each run):
	2024/08/02 14:30:10 Warming up Ollama...
	2024/08/02 14:30:12 ================================================================================
	2024/08/02 14:30:12 Attempt 1: Asking the LLM without providing any external knowledge (no RAG)
	2024/08/02 14:30:12 ================================================================================
	2024/08/02 14:30:12 Question: When did the Monarch Company exist?
	2024/08/02 14:30:12 Asking LLM...
	2024/08/02 14:30:13 Initial reply from the LLM: "I am unable to provide you with the specific dates of the Monarch Company's existence."
	...
	*/
}

// systemPromptTpl is a Go template for generating the system prompt.
// A system prompt is a powerful way to guide the LLM's behavior, setting its
// persona, instructions, and constraints.
//
// Prompt engineering is a critical part of building a successful RAG system.
// The quality of the prompt can significantly impact the quality of the answer.
//
// In this prompt:
//   - We tell the LLM it's a helpful assistant.
//   - We instruct it to be concise and unbiased.
//   - When context is provided (the `if .` block), we strictly instruct it to
//     answer *only* based on the provided search results. This helps prevent the
//     LLM from "hallucinating" or using its own (potentially outdated or incorrect)
//     internal knowledge.
//   - The `<context>` block is a common convention to clearly separate the
//     retrieved information from the user's question.
var systemPromptTpl = template.Must(template.New("system_prompt").Parse(`
You are a helpful assistant with access to a knowlege base, tasked with answering questions about the world and its history, people, places and other things.

Answer the question in a very concise manner. Use an unbiased and journalistic tone. Do not repeat text. Don't make anything up. If you are not sure about something, just say that you don't know.
{{- /* Stop here if no context is provided. The rest below is for handling contexts. */ -}}
{{- if . -}}
Answer the question solely based on the provided search results from the knowledge base. If the search results from the knowledge base are not relevant to the question at hand, just say that you don't know. Don't make anything up.

Anything between the following 'context' XML blocks is retrieved from the knowledge base, not part of the conversation with the user. The bullet points are ordered by relevance, so the first one is the most relevant.

<context>
    {{- if . -}}
    {{- range $context := .}}
    - {{.}}{{end}}
    {{- end}}
</context>
{{- end -}}

Don't mention the knowledge base, context or search results in your answer.
`))

// askLLM sends a request to the LLM with an optional context and a question.
func askLLM(ctx context.Context, contexts []string, question string) string {
	// We can use the standard OpenAI client because Ollama exposes an
	// OpenAI-compatible API. We just need to point the client to the local
	// Ollama server URL.
	openAIClient := openai.NewClientWithConfig(openai.ClientConfig{
		BaseURL:    ollamaBaseURL,
		HTTPClient: http.DefaultClient,
	})

	// We use the template to generate the final system prompt, injecting the
	// retrieved contexts if they exist.
	sb := &strings.Builder{}
	err := systemPromptTpl.Execute(sb, contexts)
	if err != nil {
		// This should not happen with a valid template.
		log.Fatalf("Failed to execute system prompt template: %v", err)
	}

	openAIClient.CreateEmbeddings(ctx, openai.EmbeddingRequest{
		Input: []string{question},
		Model: embeddingModel,
	})

	// We construct the chat messages. The conversation consists of:
	// 1. The system prompt (our instructions to the LLM).
	// 2. The user's question.
	messages := []openai.ChatCompletionMessage{
		{
			Role:    openai.ChatMessageRoleSystem,
			Content: sb.String(),
		}, {
			Role:    openai.ChatMessageRoleUser,
			Content: "Question: " + question,
		},
	}

	res, err := openAIClient.CreateChatCompletion(ctx, openai.ChatCompletionRequest{
		Model:    llmModel,
		Messages: messages,
	})
	if err != nil {
		log.Fatalf("Ollama chat completion failed: %v", err)
	}

	// The response from the LLM might have leading/trailing whitespace,
	// so we trim it for a cleaner output.
	reply := res.Choices[0].Message.Content
	return strings.TrimSpace(reply)
}
