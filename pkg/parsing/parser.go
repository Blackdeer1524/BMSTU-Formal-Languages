package parsing

import (
	"fmt"
	"math"
	"strings"
	"unicode"
	"unicode/utf8"

	"LL1/internal/utils"

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

	index      int
	_inherited bool
}

const UNDEFINED = -1

func NewNode(name string, p *Node) *Node {
	return &Node{
		name:       name,
		parent:     p,
		children:   []*Node{},
		pos:        UNDEFINED,
		index:      -1,
		_inherited: false,
	}
}

func (n *Node) Debug() string {
	var builder strings.Builder
	for _, c := range n.children {
		builder.WriteString(c.Debug())
	}
	return fmt.Sprintf("(%s[%d,%d]:%s)", n.name, n.pos, n.index, builder.String())
}

func (n *Node) Print(indent int) string {
	var b strings.Builder
	printHelper(n, 0, &b, indent)
	return b.String()
}

func printHelper(n *Node, d int, b *strings.Builder, indent int) {
	if n == nil {
		return
	}
	for i := 0; i < d; i++ {
		b.WriteRune('┊')
		for j := 1; j < indent; j++ {
			b.WriteRune(' ')
		}
	}
	fmt.Fprintf(b, "%s[%d,%d,%t]\n", n.name, n.pos, n.index, n._inherited)
	for _, c := range n.children {
		printHelper(c, d+1, b, indent)
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
		if res != nil && !(len(res.children) == 1 && res.children[0].name == EPSILON) {
			break
		}
	}
	return res
}

func (n *Node) fixPos(e int) {
	if n.pos == UNDEFINED {
		return
	}
	n.pos -= e
	for _, c := range n.children {
		c.fixPos(e)
	}
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

func NewLL1Parser(t Table, terms map[string]struct{}) LL1Parser {
	p := LL1Parser{
		table: t,
		terms: terms,
		d:     deque.New[*Node](),
	}
	return p
}

func iterString(s string, c chan<- rune) {
	for _, chr := range s {
		if unicode.IsSpace(chr) {
			continue
		}
		c <- chr
	}
	c <- utils.ExtractFirstRune(EOS)
}

func (p *LL1Parser) BuildTree(w string) *Node {
	SPrime := NewNode(SYNTHETIC_START, nil)

	d := deque.New[*Node]()
	d.PushFront(SPrime)

	p.BuildTreeIncremental(w, 0, math.MaxInt, d)
	return SPrime
}

func (p *LL1Parser) BuildTreeIncremental(s string, lastParsedPos int, n int, d *deque.Deque[*Node]) {
	if len(s) == 0 {
		panic("cannot parse empty string")
	}
	if n == 0 {
		return
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
				propogatePosition(front, i+lastParsedPos)
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
	if right == nil {
		return UNDEFINED
	}
	res := right.deduceNodePosition()
	propogatePosition(n, res)
	return res
}

func CopyUntil(pos int, root *Node, d *deque.Deque[*Node]) *Node {
	if pos <= 0 {
		return nil
	}
	return copyUntilHelper(pos, root, nil, d)
}

func copyUntilHelper(pos int, c *Node, p *Node, d *deque.Deque[*Node]) *Node {
	n := &Node{
		name:       c.name,
		parent:     p,
		children:   []*Node{},
		pos:        c.pos,
		index:      c.index,
		_inherited: true,
	}
	if c.pos > pos || c.pos == UNDEFINED {
		n.pos = UNDEFINED
		d.PushBack(n)
		return n
	}

	for _, grandChild := range c.children {
		n.children = append(n.children, copyUntilHelper(pos, grandChild, n, d))
	}

	return n
}

func filterSpaces(s string) string {
	var b strings.Builder
	for _, c := range s {
		if !unicode.IsSpace(c) {
			b.WriteRune(c)
		}
	}
	return b.String()
}

func Incremental(w0 string, T0 *Node, w1 string, info GrammarInfo, greedy bool) *Node {
	w0 = filterSpaces(w0)
	w1 = filterSpaces(w1)

	t := BuildTable(info)
	p := NewLL1Parser(t, info.Terms)

	rw0 := []rune(w0)
	rw1 := []rune(w1)
	xLen := 0
	fullMatch := true
	for xLen = range rw0 {
		if xLen == len(rw1) {
			break
		}
		if rw0[xLen] != rw1[xLen] {
			fullMatch = false
			break
		}
	}

	i := len(rw0) - 1
	j := len(rw1) - 1
	zLen := 0
	if !fullMatch {
		for i >= 0 && j >= 0 {
			if rw0[i] != rw1[j] {
				break
			}
			zLen++
			i--
			j--
		}
	}

	var T1 *Node
	if xLen == 0 {
		T1 = NewNode(SYNTHETIC_START, nil)
		p.d.PushBack(T1)
	} else {
		T1 = CopyUntil(xLen, T0, p.d)
	}

	NmPos := utf8.RuneCountInString(w0) - zLen + 1
	Nm := T0.findPos(NmPos)

	NmPrimePos := utf8.RuneCountInString(w1) - zLen + 1
	nToParse := utf8.RuneCountInString(w1) - xLen - zLen + 1
	w1 = fmt.Sprintf("%s%s", w1[xLen:], EOS)
	lastParsedPos := xLen
	for {
		p.BuildTreeIncremental(w1, lastParsedPos, nToParse, p.d)
		lastParsedPos += nToParse

		// T1Str := T1.Debug()
		// fmt.Println(T1Str)

		if nToParse >= len(w1) {
			break
		} else {
			w1 = w1[nToParse:]
		}

		NmPrime := T1.findPos(NmPrimePos)

		// NmPrimeStr := NmPrime.Debug()
		// fmt.Println(NmPrimeStr)

		oldNmPos := Nm.pos
		if Nm.name == NmPrime.name {
			NmCopy := CopyUntil(math.MaxInt, Nm, nil)

			NmPrime.parent.children[NmPrime.index] = NmCopy
			NmCopy.parent = NmPrime.parent
			NmCopy.index = NmPrime.index

			posErr := NmCopy.pos - NmPrime.pos
			NmCopy.fixPos(posErr)

			Nm = Nm.rightSibling()
			if Nm == nil {
				nToParse = len(w1)
				NmPrimePos += nToParse
				NmPos += nToParse
			} else {
				nToParse = Nm.pos - oldNmPos
				NmPrimePos += nToParse
				NmPos += nToParse
			}
		} else {
			if greedy {
				Nm = Nm.rightSibling()
				if Nm == nil {
					nToParse = len(w1)
					NmPrimePos += nToParse
					NmPos += nToParse
				} else {
					nToParse = Nm.pos - oldNmPos
					NmPrimePos += nToParse
					NmPos += nToParse
				}
			} else {
				nToParse = 1
				NmPrimePos++
				NmPos++
				Nm = T0.findPos(NmPos)
			}
		}
	}
	return T1
}
