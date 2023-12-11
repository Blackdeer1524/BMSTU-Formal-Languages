package parsing

import "LL1/internal/stack"

type LL1Parser struct {
	table Table
	s     stack.Stack
}
