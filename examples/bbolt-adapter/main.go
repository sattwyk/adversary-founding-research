package main

import (
	adv "adversary.local/sdk"
	"encoding/json"
	"fmt"
	bolt "go.etcd.io/bbolt"
	"os"
	"path/filepath"
	"sort"
)

func main() {
	if err := run(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
func run() error {
	root := os.Getenv("ADVERSARY_DATA_DIR")
	if root == "" {
		return fmt.Errorf("ADVERSARY_DATA_DIR required")
	}
	if err := os.MkdirAll(root, 0700); err != nil {
		return err
	}
	db, err := bolt.Open(filepath.Join(root, "bbolt.db"), 0600, nil)
	if err != nil {
		return err
	}
	defer db.Close()
	mode := os.Getenv("ADVERSARY_EXAMPLE_MODE")
	return adv.Serve(func(r adv.Request, ctx *adv.Context) (any, error) {
		var p struct {
			State map[string]string `json:"state"`
			Batch map[string]string `json:"batch"`
			Keys  []string          `json:"keys"`
		}
		if err := json.Unmarshal(r.Params, &p); err != nil {
			return nil, err
		}
		switch r.Method {
		case "initialize", "apply":
			values := p.Batch
			init := r.Method == "initialize"
			if init {
				values = p.State
			}
			keys := []string{}
			for k := range values {
				keys = append(keys, k)
			}
			sort.Strings(keys)
			db.NoSync = !init && mode == "relaxed"
			if !init && mode == "split" {
				for i, k := range keys {
					if err := db.Update(func(tx *bolt.Tx) error { b := tx.Bucket([]byte("kv")); return b.Put([]byte(k), []byte(values[k])) }); err != nil {
						return nil, err
					}
					if i == 0 {
						if err := ctx.Checkpoint("after-first-write"); err != nil {
							return nil, err
						}
					}
				}
			} else {
				if err := db.Update(func(tx *bolt.Tx) error {
					b, err := tx.CreateBucketIfNotExists([]byte("kv"))
					if err != nil {
						return err
					}
					for i, k := range keys {
						if err := b.Put([]byte(k), []byte(values[k])); err != nil {
							return err
						}
						if !init && i == 0 {
							if err := ctx.Checkpoint("after-first-write"); err != nil {
								return err
							}
						}
					}
					return nil
				}); err != nil {
					return nil, err
				}
			}
			return map[string]bool{"accepted": true}, nil
		case "observe":
			values := map[string]*string{}
			err := db.View(func(tx *bolt.Tx) error {
				b := tx.Bucket([]byte("kv"))
				if b == nil {
					return fmt.Errorf("missing bucket")
				}
				for _, k := range p.Keys {
					v := b.Get([]byte(k))
					if v == nil {
						values[k] = nil
						continue
					}
					value := string(v)
					values[k] = &value
				}
				return nil
			})
			return values, err
		default:
			return nil, fmt.Errorf("unknown method")
		}
	})
}
