# DefraDB RAG Example with Ollama

This example demonstrates how to use DefraDB as a vector database for a Retrieval-Augmented Generation (RAG) pipeline. It is based on the [rag-wikipedia-ollama example](https://github.com/philippgille/chromem-go/tree/main/examples/rag-wikipedia-ollama) from `chromem-go`.

## Introduction

Retrieval-Augmented Generation (RAG) is a technique that enhances the accuracy and reliability of Large Language Models (LLMs) by grounding them in external knowledge bases.

This example showcases a complete RAG workflow:

1.  **Initial Question:** We first ask an LLM a question it likely doesn't know the answer to, demonstrating its limitations without external context.
2.  **Knowledge Loading:** We load a set of documents (from Wikipedia) into a DefraDB collection. DefraDB is configured to automatically generate vector embeddings for the document text using a local Ollama model.
3.  **Similarity Search:** When the user asks a question, we convert the question into a vector embedding. This vector is then used to query DefraDB to find the most semantically similar documents from our knowledge base.
4.  **Augmented Question:** The retrieved documents are provided as context to the LLM along with the original question.
5.  **Informed Answer:** The LLM, now equipped with relevant information, provides a much more accurate and informed answer.

## How to Run

### Prerequisites

1.  **Go:** Ensure you have Go (version 1.23 or later) installed.
2.  **Ollama:** Install and run Ollama locally. You can find instructions at https://ollama.com/.
3.  **Ollama Models:** Pull the necessary models for embedding and generation:
    ```sh
    ollama pull nomic-embed-text
    ollama pull gemma:2b
    ```

### Execution

Run the example from the `rag` directory:

```sh
go run .
```

## Expected Output

The program will log its progress. You will first see the LLM fail to answer the question correctly. Then, after loading the data into DefraDB and retrieving relevant context, it will provide the correct answer.

The output will look similar to this:

```
2024/08/02 14:30:12 ================================================================================
2024/08/02 14:30:12 Asking the LLM without providing any external knowledge (no RAG)
2024/08/02 14:30:12 ================================================================================
2024/08/02 14:30:12 Question: When did the Monarch Company exist?
2024/08/02 14:30:12 Asking LLM...
2024/08/02 14:30:13 Initial reply from the LLM: "I am unable to provide you with the specific dates of the Monarch Company's existence."

2024/08/02 14:30:13 ================================================================================
2024/08/02 14:30:13 Set up DefraDB and load knowledge base
2024/08/02 14:30:13 ================================================================================
2024/08/02 14:30:13 Setting up DefraDB...
2024/08/02 14:30:13 Adding 'Wiki' collection schema to DefraDB...
2024/08/02 14:30:13 Reading JSON lines from wiki.jsonl and adding to the 'Wiki' collection...
2024/08/02 14:30:25 Finished loading data into DefraDB.
2024/08/02 14:30:25 ================================================================================
2024/08/02 14:30:25 Retrieving relevant documents from DefraDB
2024/08/02 14:30:25 ================================================================================
2024/08/02 14:30:25 Creating embedding for the query...
2024/08/02 14:30:25 Querying DefraDB for similar documents...
2024/08/02 14:30:26 Search (incl. query embedding) took 1.1s
2024/08/02 14:30:26 Found relevant documents:
2024/08/02 14:30:26  - Document 1 (similarity: 0.7341): "The Monarch Company was an American manufacturer of confectionery, syrups and other food products. The..."
2024/08/02 14:30:26  - Document 2 (similarity: 0.6512): "Monarch Beverage Company, Inc. is an American beverage distributor based in Indianapolis, Indiana. Th..."
2024/08/02 14:30:26 ================================================================================
2024/08/02 14:30:26 Asking the LLM with retrieved knowledge (with RAG)
2024/08/02 14:30:26 ================================================================================
2024/08/02 14:30:26 Asking LLM with augmented question...
2024/08/02 14:30:28 Reply after augmenting the question with knowledge: "The Monarch Company was founded in 1903 and ceased operations in 1953."
```
