package graphs

import (
	"LL1/internal/stack"
)

type condensedNode struct {
	Next  map[int]struct{}
	Nodes []string
}

type tarjanTraverser struct {
	graph           map[string]map[string]struct{}
	visited         map[string]color
	compCount       int
	s               *stack.Stack
	time            int
	node2group      map[string]int
	group2component map[int]int
	components      map[int]condensedNode
}

func newTarjanTraverser(graph map[string]map[string]struct{}) tarjanTraverser {
	return tarjanTraverser{
		graph:           graph,
		visited:         make(map[string]color, len(graph)),
		compCount:       0,
		s:               stack.NewStack(),
		time:            0,
		node2group:      make(map[string]int, len(graph)),
		group2component: map[int]int{},
		components:      map[int]condensedNode{},
	}
}

func (t *tarjanTraverser) build() map[int]condensedNode {
	for key := range t.graph {
		t.visited[key] = white
		t.node2group[key] = 0
	}
	for parent := range t.graph {
		if t.visited[parent] == white {
			t.visit(parent)
		}
	}
	return t.components
}

func (t *tarjanTraverser) visit(p string) {
	t.time++
	t.s.Push(p)
	tin := t.time
	t.node2group[p] = tin

	t.visited[p] = grey
	for c := range t.graph[p] {
		if t.visited[c] == white {
			t.visit(c)
			t.node2group[p] = min(t.node2group[p], t.node2group[c])
		} else if t.visited[c] == grey {
			t.node2group[p] = min(t.node2group[p], t.node2group[c])
		}
	}
	t.visited[p] = black

	if t.node2group[p] != tin {
		return
	}

	t.group2component[t.node2group[p]] = t.compCount
	condNode := condensedNode{
		Next:  map[int]struct{}{},
		Nodes: []string{},
	}
	for {
		top := t.s.Pop().(string)
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

func buildCondensedGraph(graph map[string]map[string]struct{}) map[int]condensedNode {
	tr := newTarjanTraverser(graph)
	return tr.build()
}
