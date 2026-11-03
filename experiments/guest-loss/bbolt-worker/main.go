package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	bolt "go.etcd.io/bbolt"
	"os"
)

func must(err error) {
	if err != nil {
		panic(err)
	}
}
func main() {
	a := os.Args
	db, err := bolt.Open(a[2], 0600, nil)
	must(err)
	defer db.Close()
	if a[1] == "read" {
		values := map[string]string{}
		must(db.View(func(tx *bolt.Tx) error {
			b := tx.Bucket([]byte("kv"))
			if b == nil {
				return fmt.Errorf("missing bucket")
			}
			for _, k := range []string{"a", "b"} {
				values[k] = string(b.Get([]byte(k)))
			}
			return nil
		}))
		must(json.NewEncoder(os.Stdout).Encode(values))
		return
	}
	if len(a) > 3 && a[3] == "unsafe" {
		db.NoSync = true
	}
	must(db.Update(func(tx *bolt.Tx) error {
		b, e := tx.CreateBucketIfNotExists([]byte("kv"))
		if e != nil {
			return e
		}
		v := "1"
		if a[1] == "init" {
			v = "0"
		}
		for _, k := range []string{"a", "b"} {
			if e = b.Put([]byte(k), []byte(v)); e != nil {
				return e
			}
		}
		return nil
	}))
	fmt.Println(`{"status":"ok","op":"put_pair"}`)
	if a[1] == "write" {
		bufio.NewScanner(os.Stdin).Scan()
	}
}
