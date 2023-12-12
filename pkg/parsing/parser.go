package parsing

import (
	"math"

	"github.com/gammazero/deque"
)

type LL1Parser struct {
	table Table
	terms map[string]struct{}
	d     *deque.Deque[*Node]
}

type Node struct {
	name     string
	parent   *Node
	children []*Node
	pos      int
}

const UNDEFINED = -1

func NewNode(name string) *Node {
	return &Node{
		name:     name,
		parent:   &Node{},
		children: []*Node{},
		pos:      UNDEFINED,
	}
}

func propogatePosition(n *Node, p int) {
	c := n
	for c.parent != nil && c.pos == UNDEFINED {
		c.pos = p
		c = c.parent
	}
}

func newLL1Parser(t Table) LL1Parser {
	p := LL1Parser{
		table: t,
		d:     deque.New[*Node](),
	}
	return p
}

func iterString(s string, c chan<- rune) {
	for _, chr := range s {
		c <- chr
	}
	c <- '$'
}

func (p *LL1Parser) BuildTree(w string) *Node {
	S := NewNode("S")

	d := deque.New[*Node]()
	d.PushFront(S)

	p.BuildTreeIncremental(w, math.MaxInt, d)
	return S
}

func (p *LL1Parser) BuildTreeIncremental(w string, pos int, d *deque.Deque[*Node]) {
	if len(w) == 1 {
		panic("cannot parse empty string")
	}

	p.d = d

	strChan := make(chan rune)
	go iterString(w, strChan)

	i := 1
	current := string(<-strChan)
	for i <= pos && p.d.Len() > 0 {
		front := p.d.PopFront()
		if _, ok := p.terms[front.name]; ok {
			if front.name != EPSILON {
				current = string(<-strChan)
				propogatePosition(front, i)
				i++
			}
			continue
		}
		nextStack := p.table[front.name][current]
		for i := len(nextStack) - 1; i != 0; i-- {
			item := nextStack[i]
			node := NewNode(item)
			p.d.PushFront(node)
		}
	}
}

func CopyUntil(pos int, c *Node, p *Node) *Node {
	if pos == 0 {
		return nil
	}

	n := &Node{
		name:     c.name,
		parent:   p,
		children: []*Node{},
		pos:      c.pos,
	}
	if c.pos > pos || c.pos == UNDEFINED {
		return n
	}

	for _, grandChild := range c.children {
		n.children = append(n.children, CopyUntil(pos, grandChild, n))
	}

	return n
}
