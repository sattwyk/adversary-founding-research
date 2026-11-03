module adversary.local/bbolt-example

go 1.23.2

require (
	adversary.local/sdk v0.0.0
	go.etcd.io/bbolt v1.4.3
)

require golang.org/x/sys v0.29.0 // indirect

replace adversary.local/sdk => ../../sdk/go
