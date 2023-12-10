package graphs

import (
	"testing"
)

func contains(b []rune, r rune) bool {
	for _, c := range b {
		if r == c {
			return true
		}
	}
	return false
}

func TestTarjan(t *testing.T) {
	graph := map[rune]map[rune]struct{}{
		'1': {
			'2': struct{}{},
			'3': struct{}{},
		},
		'2': {
			'1': struct{}{},
			'3': struct{}{},
		},
		'3': {
			'1': struct{}{},
			'2': struct{}{},
		},
	}
	res := buildCondensedGraph(graph)
	if len(res) != 1 {
		t.Fatalf("expected %d got %d", 1, len(res))
	}

	if len(res[0].Next) != 0 {
		t.Fatalf("expected %d got %d", 0, len(res))
	}

	if len(res[0].Nodes) != 3 {
		t.Fatalf("expected %d got %d", 3, len(res))
	}

	nodes := res[0].Nodes
	if !contains(nodes, '1') {
		t.Errorf("expected %c in: %v", '1', nodes)
	}
	if !contains(nodes, '2') {
		t.Errorf("expected %c in: %v", '2', nodes)
	}
	if !contains(nodes, '3') {
		t.Errorf("expected %c in: %v", '3', nodes)
	}
}

func TestTarjanTwoComponents(t *testing.T) {
	graph := map[rune]map[rune]struct{}{
		'1': {
			'2': struct{}{},
		},
		'2': {
			'1': struct{}{},
			'3': struct{}{},
		},
		'3': {
			'4': struct{}{},
		},
		'4': {
			'3': struct{}{},
		},
	}
	res := buildCondensedGraph(graph)
	if len(res) != 2 {
		t.Fatalf("expected %d got %d", 2, len(res))
	}

	if len(res[0].Next) < len(res[1].Next) {
		res[0], res[1] = res[1], res[0]
	}

	if len(res[0].Next) != 1 {
		t.Fatalf("expected %d got %d", 1, len(res))
	}
	if len(res[1].Next) != 0 {
		t.Fatalf("expected %d got %d", 0, len(res))
	}

	if len(res[0].Nodes) != 2 {
		t.Fatalf("expected %d got %d", 2, len(res))
	}
	if len(res[1].Nodes) != 2 {
		t.Fatalf("expected %d got %d", 2, len(res))
	}

	f := res[0].Nodes
	if !contains(f, '1') {
		t.Errorf("expected %c in: %v", '1', f)
	}
	if !contains(f, '2') {
		t.Errorf("expected %c in: %v", '2', f)
	}

	s := res[1].Nodes
	if !contains(s, '3') {
		t.Errorf("expected %c in: %v", '3', s)
	}
	if !contains(s, '4') {
		t.Errorf("expected %c in: %v", '4', s)
	}
}
