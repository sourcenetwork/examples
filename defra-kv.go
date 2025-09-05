// main.go
package main

import (
	"bytes"
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"syscall"
	"time"

	dclient "github.com/sourcenetwork/defradb/client"
	dnode   "github.com/sourcenetwork/defradb/node"
)

func defaultRootdir() string {
	if cwd, err := os.Getwd(); err == nil {
		return filepath.Join(cwd, ".defra-kv")
	}
	return ".defra-kv"
}

func expandHome(p string) string {
	if strings.HasPrefix(p, "~/") {
		if h, err := os.UserHomeDir(); err == nil {
			return filepath.Join(h, p[2:])
		}
	}
	return p
}

func resolveRootdir(p string) string {
	p = expandHome(p)
	if !filepath.IsAbs(p) {
		if abs, err := filepath.Abs(p); err == nil {
			p = abs
		}
	}
	if err := os.MkdirAll(p, 0o755); err != nil {
		log.Fatalf("create rootdir: %v", err)
	}
	return p
}

// Single JSON-based KV schema with indexes where useful.
const kvSchema = `
type KV {
  key: String @index
  value: JSON
  updatedAt: DateTime @index
}
`

func kvExists(ctx context.Context, n *dnode.Node) bool {
	res := n.DB.ExecRequest(ctx, `query { __type(name: "KV") { name } }`)
	if len(res.GQL.Errors) > 0 {
		return false
	}
	b, err := json.Marshal(res.GQL.Data)
	if err != nil {
		return false
	}
	return bytes.Contains(b, []byte(`"name":"KV"`))
}

func ensureKV(ctx context.Context, n *dnode.Node) error {
	if kvExists(ctx, n) {
		return nil
	}
	if _, err := n.DB.AddSchema(ctx, kvSchema); err != nil {
		return fmt.Errorf("KV schema add failed: %v", err)
	}
	return nil
}

func main() {
	// ---- Custom FlagSet (avoid random flags from transitive deps) ----
	fs := flag.NewFlagSet("defra-kv", flag.ExitOnError)
	rootdir := fs.String("rootdir", defaultRootdir(), "data/config directory (default: ./.defra-kv)")
	secret  := fs.String("keyring-secret", "", "keyring secret (or set DEFRA_KEYRING_SECRET)")
	query   := fs.String("query", "", "GraphQL query/mutation; if empty, read from stdin")
	varsStr := fs.String("vars", "", "JSON variables (optional)")
	pretty  := fs.Bool("pretty", true, "pretty-print JSON")
	reqTO   := fs.Duration("timeout", 10*time.Second, "per-request timeout")
	_ = fs.Parse(os.Args[1:])

	// Keyring secret (first run convenience).
	if *secret != "" {
		_ = os.Setenv("DEFRA_KEYRING_SECRET", *secret)
	}
	if os.Getenv("DEFRA_KEYRING_SECRET") == "" {
		_ = os.Setenv("DEFRA_KEYRING_SECRET", "dev-dev-dev")
	}

	// Read query (flag or stdin).
	q := strings.TrimSpace(*query)
	if q == "" {
		b, err := io.ReadAll(os.Stdin)
		if err != nil {
			log.Fatalf("read stdin: %v", err)
		}
		q = strings.TrimSpace(string(b))
	}
	if q == "" {
		fmt.Fprintln(os.Stderr, "no query provided; pass --query or pipe to stdin")
		os.Exit(2)
	}

	// Variables (optional, parsed later).
	var rawVars json.RawMessage
	if v := strings.TrimSpace(*varsStr); v != "" {
		rawVars = json.RawMessage(v)
	}

	// Context + signals.
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	// --- 1) Create the node (embedded, persistent Badger) ---
	n, err := dnode.New(
		ctx,
		dnode.WithDisableAPI(true),                    // no HTTP server
		dnode.WithDisableP2P(true),                    // local only
		dnode.WithBadgerInMemory(false),               // persistent
		dnode.WithStoreType(dnode.BadgerStore),
		dnode.WithStorePath(resolveRootdir(*rootdir)), // data dir
		dnode.WithLensRuntime(dnode.Wazero),           // pure-Go WASM runtime
	)
	if err != nil {
		log.Fatalf("node.New: %v", err)
	}
	defer func() { _ = n.Close(ctx) }()
	if err := n.Start(ctx); err != nil {
		log.Fatalf("node.Start: %v", err)
	}

	// --- 2) Ensure KV schema (idempotent) ---
	if err := ensureKV(ctx, n); err != nil {
		log.Fatalf("ensure KV schema: %v", err)
	}

	// --- 3) Execute the user’s GraphQL directly in-process ---
	var vars map[string]any
	if len(rawVars) > 0 {
		if err := json.Unmarshal(rawVars, &vars); err != nil {
			log.Fatalf("parse -vars: %v", err)
		}
	}

	reqCtx, cancel := context.WithTimeout(ctx, *reqTO)
	defer cancel()

	res := n.DB.ExecRequest(reqCtx, q, dclient.WithVariables(vars))
	if len(res.GQL.Errors) > 0 {
		enc, _ := json.MarshalIndent(res.GQL.Errors, "", "  ")
		fmt.Fprintln(os.Stderr, string(enc))
		os.Exit(1)
	}

	// Output JSON (pretty or compact).
	if *pretty {
		out, _ := json.MarshalIndent(map[string]any{"data": res.GQL.Data}, "", "  ")
		fmt.Println(string(out))
	} else {
		out, _ := json.Marshal(map[string]any{"data": res.GQL.Data})
		fmt.Println(string(out))
	}
}
