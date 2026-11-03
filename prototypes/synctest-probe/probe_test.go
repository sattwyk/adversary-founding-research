package synctestprobe

import (
	"fmt"
	"testing"
	"testing/synctest"
	"time"
)

func TestSchedulingObservation(t *testing.T) {
	seen := map[string]int{}
	for k := 0; k < 50; k++ {
		synctest.Test(t, func(t *testing.T) {
			ch := make(chan int, 16)
			for j := 0; j < 16; j++ {
				go func(j int) { time.Sleep(time.Second); ch <- j }(j)
			}
			order := []int{}
			for j := 0; j < 16; j++ {
				order = append(order, <-ch)
			}
			seen[fmt.Sprint(order)]++
		})
	}
	t.Logf("50 identical workloads, distinct receive orders=%d", len(seen))
}
