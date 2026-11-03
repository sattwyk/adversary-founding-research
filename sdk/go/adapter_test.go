package adversary

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
)

func TestCheckpointRoundTrip(t *testing.T) {
	input := strings.NewReader("{\"v\":1,\"id\":\"op-1\",\"method\":\"apply\",\"params\":{}}\n{\"v\":1,\"id\":\"op-1\",\"kind\":\"continue\",\"name\":\"written\"}\n")
	var output bytes.Buffer
	err := ServeIO(input, &output, func(_ Request, c *Context) (any, error) {
		if err := c.Checkpoint("written"); err != nil {
			return nil, err
		}
		return map[string]bool{"accepted": true}, nil
	})
	if err != nil {
		t.Fatal(err)
	}
	lines := strings.Split(strings.TrimSpace(output.String()), "\n")
	if len(lines) != 2 {
		t.Fatal(output.String())
	}
	var checkpoint, response frame
	if err := json.Unmarshal([]byte(lines[0]), &checkpoint); err != nil {
		t.Fatal(err)
	}
	if err := json.Unmarshal([]byte(lines[1]), &response); err != nil {
		t.Fatal(err)
	}
	if checkpoint.Kind != "checkpoint" || response.Kind != "response" || response.Error != "" {
		t.Fatal(output.String())
	}
}
func TestBadContinuationCannotBecomeSuccess(t *testing.T) {
	input := strings.NewReader("{\"v\":1,\"id\":\"op-1\",\"method\":\"apply\",\"params\":{}}\n{\"v\":1,\"id\":\"wrong\",\"kind\":\"continue\",\"name\":\"written\"}\n")
	var output bytes.Buffer
	err := ServeIO(input, &output, func(_ Request, c *Context) (any, error) {
		if err := c.Checkpoint("written"); err != nil {
			return nil, err
		}
		return map[string]bool{"accepted": true}, nil
	})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(output.String(), "invalid checkpoint continuation") {
		t.Fatal(output.String())
	}
}
func TestVersionAndFrameLimit(t *testing.T) {
	if ServeIO(strings.NewReader("{\"v\":2,\"id\":\"x\",\"method\":\"x\",\"params\":{}}\n"), &bytes.Buffer{}, func(Request, *Context) (any, error) { return nil, nil }) == nil {
		t.Fatal("accepted invalid version")
	}
	if ServeIO(strings.NewReader(strings.Repeat("x", MaxFrameBytes+1)), &bytes.Buffer{}, func(Request, *Context) (any, error) { return nil, nil }) == nil {
		t.Fatal("accepted oversize frame")
	}
}
