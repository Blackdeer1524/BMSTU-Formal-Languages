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

	index int
}

const UNDEFINED = -1

func NewNode(name string) *Node {
	return &Node{
		name:     name,
		parent:   &Node{},
		children: []*Node{},
		pos:      UNDEFINED,
		index:    -1,
	}
}

func (n *Node) findPos(pos int) *Node {
	if n.pos == pos {
		return n
	}
	var res *Node = nil
	for _, c := range n.children {
		res = c.findPos(pos)
	}
	return res
}

func (n *Node) rightSibling() *Node {
	if n.parent == nil {
		return nil
	}
	if n.index == len(n.parent.children)-1 {
		return n.parent.rightSibling()
	}
	return n.parent.children[n.index+1]
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

	p.BuildTreeIncremental(w, 1, math.MaxInt, d)
	return S
}

// gives parsing results for s[left:right)
//
// left & right are indexed from 1!!!
func (p *LL1Parser) BuildTreeIncremental(s string, left int, right int, d *deque.Deque[*Node]) {
	s = s[left-1:]
	if len(s) == 0 {
		panic("cannot parse empty string")
	}

	p.d = d
	strChan := make(chan rune)
	go iterString(s, strChan)

	q := deque.New[*Node]()

	i := left
	current := string(<-strChan)
	for i < right && p.d.Len() > 0 {
		front := p.d.PopFront()
		if _, ok := p.terms[front.name]; ok {
			if front.name != EPSILON {
				current = string(<-strChan)
				propogatePosition(front, i)
				i++
			} else if front.parent != nil && len(front.parent.children) == 1 {
				q.PushBack(front)
			}

			continue
		}
		nextStack := p.table[front.name][current]
		for i := len(nextStack) - 1; i >= 0; i-- {
			item := nextStack[i]
			node := NewNode(item)
			node.index = i
			p.d.PushFront(node)
		}
	}

	for q.Len() > 0 {
		q.PopFront().deduceNodePosition()
	}
}

func (n *Node) deduceNodePosition() int {
	if n.pos != UNDEFINED {
		return n.pos
	}
	right := n.rightSibling()
	if right != nil {
		n.pos = right.deduceNodePosition()
	}
	return n.pos
}

func CopyUntil(pos int, c *Node, p *Node, d *deque.Deque[*Node]) *Node {
	if pos == 0 {
		return nil
	}
	n := &Node{
		name:     c.name,
		parent:   p,
		children: []*Node{},
		pos:      c.pos,
		index:    c.index,
	}
	if c.pos > pos || c.pos == UNDEFINED {
		d.PushBack(n)
		return n
	}

	for _, grandChild := range c.children {
		n.children = append(n.children, CopyUntil(pos, grandChild, n, d))
	}

	return n
}

func foo(w0 string, w1 string, info GrammarInfo, strat func(n *Node) int) {
	t := BuildTable(info)
	p := newLL1Parser(t)

	rw0 := []rune(w0)
	rw1 := []rune(w1)

	xLen := 0
	for xLen = range rw0 {
		if rw0[xLen] != rw1[xLen] {
			break
		}
	}

	i := len(rw0) - 1
	j := len(rw1) - 1
	zLen := 0
	for i >= 0 && j >= 0 {
		if rw0[i] != rw1[j] {
			break
		}
		zLen++
		i--
		j--
	}

	T0 := p.BuildTree(w0)
	Nm := T0

	T1 := CopyUntil(xLen, T0, nil, p.d)
	// NmPrime := T1

	w0bound := len(w0) - zLen + 1
	w1bound := len(w1) - zLen + 1
	w1left := xLen + 1
	for {
		oldNmPos := Nm.pos
		Nm = Nm.findPos(w0bound)

		p.BuildTreeIncremental(w1, w1left, w1bound, p.d)
		NmPrime := T1.findPos(w1bound)

		if Nm.name == NmPrime.name {
			NmCopy := CopyUntil(math.MaxInt, Nm, nil, nil)
			NmPrime.parent.children[NmPrime.index] = NmCopy
			Nm = Nm.rightSibling()
			w1left += Nm.pos - oldNmPos
		} else {

		}
	}
}
