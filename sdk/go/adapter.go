// Package adversary provides an NDJSON adapter transport and cooperative fault
// checkpoints. It does not replace the Go runtime or control its scheduler.
package adversary

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
)

const Version = 1
const MaxFrameBytes = 1024 * 1024

type Request struct {
	V      int             `json:"v"`
	ID     string          `json:"id"`
	Method string          `json:"method"`
	Params json.RawMessage `json:"params"`
}
type frame struct {
	V      int    `json:"v"`
	ID     string `json:"id"`
	Kind   string `json:"kind"`
	Name   string `json:"name,omitempty"`
	Result any    `json:"result,omitempty"`
	Error  string `json:"error,omitempty"`
}
type Context struct {
	id     string
	input  *bufio.Scanner
	output *json.Encoder
}
type Handler func(Request, *Context) (any, error)

func (c *Context) Checkpoint(name string) error {
	if err := c.output.Encode(frame{V: Version, ID: c.id, Kind: "checkpoint", Name: name}); err != nil {
		return err
	}
	if !c.input.Scan() {
		return fmt.Errorf("controller disconnected at checkpoint: %v", c.input.Err())
	}
	var control frame
	if err := json.Unmarshal(c.input.Bytes(), &control); err != nil {
		return err
	}
	if control.V != Version || control.ID != c.id || control.Kind != "continue" || control.Name != name {
		return fmt.Errorf("invalid checkpoint continuation")
	}
	return nil
}

func Serve(handler Handler) error { return ServeIO(os.Stdin, os.Stdout, handler) }
func ServeIO(input io.Reader, output io.Writer, handler Handler) error {
	scanner := bufio.NewScanner(input)
	scanner.Buffer(make([]byte, 4096), MaxFrameBytes)
	encoder := json.NewEncoder(output)
	for scanner.Scan() {
		var request Request
		if err := json.Unmarshal(scanner.Bytes(), &request); err != nil {
			return err
		}
		if request.V != Version {
			return fmt.Errorf("unsupported protocol version")
		}
		result, err := handler(request, &Context{id: request.ID, input: scanner, output: encoder})
		response := frame{V: Version, ID: request.ID, Kind: "response", Result: result}
		if err != nil {
			response.Result = nil
			response.Error = err.Error()
		}
		if err := encoder.Encode(response); err != nil {
			return err
		}
	}
	return scanner.Err()
}
