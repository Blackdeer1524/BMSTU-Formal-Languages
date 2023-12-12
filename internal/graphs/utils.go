package graphs

import "fmt"

type color int

const (
	white color = iota // haven't seen yet
	grey               // yet to finish processing
	black              // processed
)

// TopoSort sorts topologically given dependecies graph
func TopoSort(deps map[string]map[string]struct{}) []string {
	condGraph := buildCondensedGraph(deps)

	visited := make(map[int]color)
	for v := range condGraph {
		visited[v] = white
	}

	condSorted := make([]int, 0)

	var topoVisit func(int)
	topoVisit = func(v int) {
		if visited[v] == grey {
			panic(fmt.Sprintf("cyclic dependency found on var %c", v))
		}

		visited[v] = grey
		for d := range condGraph[v].Next {
			if visited[d] != black {
				topoVisit(d)
				condSorted = append(condSorted, d)
			}
		}
		condSorted = append(condSorted, v)
		visited[v] = black
	}

	for v := range condGraph {
		if visited[v] == white {
			topoVisit(v)
		}
	}

	res := make([]string, 0)
	for _, compIndex := range condSorted {
		res = append(res, condGraph[compIndex].Nodes...)
		for i := 0; i < len(condGraph[compIndex].Nodes)-1; i++ {
			res = append(res, condGraph[compIndex].Nodes[i])
		}
	}

	return res
}
