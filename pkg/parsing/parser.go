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

func NewNode(name string, p *Node) *Node {
	return &Node{
		name:     name,
		parent:   p,
		children: []*Node{},
		pos:      UNDEFINED,
		index:    -1,
	}
}

func (n *Node) findPos(pos int) *Node {
	if n.pos == pos {
		return n
	} else if n.pos > pos {
		return nil
	}
	var res *Node = nil
	for _, c := range n.children {
		res = c.findPos(pos)
		if res != nil {
			break
		}
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
	n.pos = p
	for n.parent != nil && n.parent.pos == UNDEFINED {
		n = n.parent
		n.pos = p
	}
}

func newLL1Parser(t Table, terms map[string]struct{}) LL1Parser {
	p := LL1Parser{
		table: t,
		terms: terms,
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
	S := NewNode("S", nil)

	d := deque.New[*Node]()
	d.PushFront(S)

	p.BuildTreeIncremental(w, 0, math.MaxInt, d)
	return S
}

func (p *LL1Parser) BuildTreeIncremental(s string, lastParsedPos int, n int, d *deque.Deque[*Node]) {
	if len(s) == 0 {
		panic("cannot parse empty string")
	}

	p.d = d
	strChan := make(chan rune)
	go iterString(s, strChan)

	q := deque.New[*Node]()

	i := 1
	current := string(<-strChan)
	for i <= n && p.d.Len() > 0 {
		front := p.d.PopFront()
		if _, ok := p.terms[front.name]; ok {
			if front.name != EPSILON {
				propogatePosition(front, i + lastParsedPos)
				i++
				if front.name == EOS {
					break
				}
				current = string(<-strChan)
			} else if front.parent != nil && len(front.parent.children) == 1 {
				q.PushBack(front)
			}
			continue
		}
		nextStack := p.table[front.name][current]
		if len(nextStack) == 0 {
			panic("s ∉ L")
		}

		buf := []*Node{}
		for i := len(nextStack) - 1; i >= 0; i-- {
			item := nextStack[i]
			node := NewNode(item, front)
			node.index = i
			buf = append(buf, node)
			p.d.PushFront(node)
		}
		for i := len(buf) - 1; i >= 0; i-- {
			front.children = append(front.children, buf[i])
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

func foo(w0 string, w1 string, info GrammarInfo, greedy bool) *Node {
	t := BuildTable(info)
	p := newLL1Parser(t, info.Terms)

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
	NmPos := len(w0) - zLen + 1
	Nm := T0.findPos(NmPos)
	oldNmPos := Nm.pos

	T1 := CopyUntil(xLen, T0, nil, p.d)
	NmPrimePos := len(w1) - zLen + 1
	nToParse := len(w1) - xLen - zLen + 1
	w1 = w1[xLen:]
	offset := xLen
	for {
		p.BuildTreeIncremental(w1, offset, nToParse, p.d)
		offset += nToParse
		w1 = w1[nToParse:]

		NmPrime := T1.findPos(NmPrimePos)
		if Nm.name == NmPrime.name {
			NmCopy := CopyUntil(math.MaxInt, Nm, nil, nil)
			NmPrime.parent.children[NmPrime.index] = NmCopy

			Nm = Nm.rightSibling()

			nToParse = Nm.pos - oldNmPos
			NmPrimePos += nToParse
			NmPos += nToParse
		} else {
			if greedy {
				Nm = Nm.rightSibling()
				if Nm == nil {
					panic("???")
					nToParse = len(w0) - oldNmPos
					NmPrimePos += nToParse
					NmPos += nToParse
				} else {
					nToParse = Nm.pos - oldNmPos
					NmPrimePos += nToParse
					NmPos += nToParse
				}

			} else {
				NmPrimePos++
				NmPos++
				nToParse = 1
				Nm = T0.findPos(NmPos)
			}
		}
	}
	return T1
}
