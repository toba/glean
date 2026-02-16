package minigo

import "fmt"

// HandlerFunc defines a handler function type.
type HandlerFunc func(*Context)

// Logger returns a middleware that logs each request.
func Logger() HandlerFunc {
	return func(c *Context) {
		ip := c.ClientIP()
		fmt.Printf("request from %s\n", ip)
		c.Next()
	}
}
