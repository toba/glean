package minigo

import (
	"net/http"
	"strings"
)

// Context carries per-request state through the middleware chain.
type Context struct {
	Request  *http.Request
	Writer   http.ResponseWriter
	handlers []HandlerFunc
	index    int
}

// Next advances to the next handler in the chain.
func (c *Context) Next() {
	c.index++
	for c.index < len(c.handlers) {
		c.handlers[c.index](c)
		c.index++
	}
}

// ClientIP returns the client's IP address, checking X-Forwarded-For first.
func (c *Context) ClientIP() string {
	forwarded := c.Request.Header.Get("X-Forwarded-For")
	if forwarded != "" {
		parts := strings.Split(forwarded, ",")
		return strings.TrimSpace(parts[0])
	}
	return c.Request.RemoteAddr
}
