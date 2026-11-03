//go:build sim

package bolt_test

import (
	"fmt"
	"github.com/jellevandenhooff/gosim"
	bolt "go.etcd.io/bbolt"
	"os"
	"testing"
)

// Copy beside Gosim's examples/bolt/bolt_test.go; dependency remains bbolt 1.3.11.
func TestAdversaryDurability(t *testing.T) {
	for _, unsafe := range []bool{false, true} {
		t.Run(fmt.Sprint("unsafe=", unsafe), func(t *testing.T) {
			m := gosim.NewSimpleMachine(func() {
				db, e := bolt.Open("db", 0600, nil)
				if e != nil {
					panic(e)
				}
				if e = db.Update(func(tx *bolt.Tx) error {
					b, e := tx.CreateBucketIfNotExists([]byte("kv"))
					if e != nil {
						return e
					}
					for _, k := range []string{"a", "b"} {
						if e = b.Put([]byte(k), []byte("0")); e != nil {
							return e
						}
					}
					return nil
				}); e != nil {
					panic(e)
				}
				d, e := os.Open(".")
				if e != nil {
					panic(e)
				}
				if e = d.Sync(); e != nil {
					panic(e)
				}
				d.Close()
				db.NoSync = unsafe
				if e = db.Update(func(tx *bolt.Tx) error {
					for _, k := range []string{"a", "b"} {
						if e := tx.Bucket([]byte("kv")).Put([]byte(k), []byte("1")); e != nil {
							return e
						}
					}
					return nil
				}); e != nil {
					panic(e)
				}
				// Successful return is the acknowledged durability boundary.
				gosim.CurrentMachine().Crash()
			})
			m.Wait()
			m.SetMainFunc(func() {
				db, e := bolt.Open("db", 0600, nil)
				if e != nil {
					panic(e)
				}
				defer db.Close()
				if e = db.View(func(tx *bolt.Tx) error {
					got := string(tx.Bucket([]byte("kv")).Get([]byte("a")))
					other := string(tx.Bucket([]byte("kv")).Get([]byte("b")))
					want := "1"
					if unsafe {
						want = "0"
					}
					if got != want || other != want {
						t.Errorf("recovery got (%s,%s) want (%s,%s)", got, other, want, want)
					}
					t.Logf("unsafe=%v recovered=(%s,%s)", unsafe, got, other)
					return nil
				}); e != nil {
					panic(e)
				}
			})
			// Restart drops every unsynced effect; partial outcomes are a separate API.
			m.Restart()
			m.Wait()
		})
	}
}
