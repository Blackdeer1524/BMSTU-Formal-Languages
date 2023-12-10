package graphs

import (
	"LL1/internal/internal/stack"
)

type CondensedNode struct {
	Next  map[int]struct{}
	Nodes []rune
}

type tarjanTraverser struct {
	graph           map[rune]map[rune]struct{}
	visited         map[rune]struct{}
	compCount       int
	s               stack.Stack
	time            int
	node2group      map[rune]int
	group2component map[int]int
	components      map[int]CondensedNode
}

func newTarjanTraverser(graph map[rune]map[rune]struct{}) tarjanTraverser {
	return tarjanTraverser{
		graph:           graph,
		visited:         make(map[rune]struct{}, len(graph)),
		compCount:       0,
		s:               stack.Stack{},
		time:            0,
		node2group:      make(map[rune]int, len(graph)),
		group2component: map[int]int{},
		components:      map[int]CondensedNode{},
	}
}

func (t *tarjanTraverser) build() map[int]CondensedNode {
	for key := range t.visited {
		t.node2group[key] = 0
	}
	for parent := range t.graph {
		if _, ok := t.visited[parent]; !ok {
			t.visit(parent)
		}
	}
	return t.components
}

func (t *tarjanTraverser) visit(p rune) {
	t.s.Push(p)
	tin := t.time
	t.node2group[p] = tin

	t.visited[p] = struct{}{}
	for c := range t.graph {
		if _, ok := t.visited[c]; !ok {
			t.time++
			t.visit(c)
		}
		t.node2group[p] = min(t.node2group[p], t.node2group[c])
	}

	if t.node2group[p] != tin {
		return
	}

	t.group2component[t.node2group[p]] = t.compCount
	condNode := CondensedNode{
		Next:  map[int]struct{}{},
		Nodes: []rune{},
	}
	for {
		top := t.s.Pop().(rune)
		condNode.Nodes = append(condNode.Nodes, top)
		for child := range t.graph[top] {
			childComp := t.group2component[t.node2group[child]]
			if childComp == t.compCount {
				continue
			}
			condNode.Next[childComp] = struct{}{}
		}

		if top == p {
			break
		}
	}
	t.components[t.compCount] = condNode
	t.compCount++
}

func BuildCondensedGraph(graph map[rune]map[rune]struct{}) map[int]CondensedNode {
	tr := newTarjanTraverser(graph)
	return tr.build()
}
