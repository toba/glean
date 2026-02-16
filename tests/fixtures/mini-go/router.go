package minigo

import "net/http"

// Engine is the main router that handles HTTP requests.
type Engine struct {
	handlers []HandlerFunc
	basePath string
}

// ServeHTTP implements the http.Handler interface.
func (e *Engine) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	c := &Context{
		Request:  r,
		Writer:   w,
		handlers: e.handlers,
		index:    -1,
	}
	e.handleRequest(c)
}

// handleRequest dispatches the request through the middleware chain.
func (e *Engine) handleRequest(c *Context) {
	c.Next()
}
