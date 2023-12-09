package utils

import "fmt"

func extractFirstRune(s string) rune {
	var first rune
	for _, c := range s {
		first = c
		break
	}
	return first
}

func extractPair(str string) (f rune, s rune) {
	for i, c := range str {
		if i == 0 {
			f = c
		} else {
			s = c
		}
	}
	return
}

func mergeInPlace(dst map[rune]struct{}, src map[rune]struct{}) {
	for key := range src {
		dst[key] = struct{}{}
	}
}

func topoSort(dependencies map[rune]map[rune]struct{}) []rune {
	type color int
	const (
		WHITE color = iota // haven't seen yet
		GREY               // yet to finish processing
		BLACK              // processed
	)

	visited := make(map[rune]color)
	for v := range dependencies {
		visited[v] = WHITE
	}

	res := make([]rune, 0)

	var topoVisit func(rune)
	topoVisit = func(v rune) {
		if visited[v] == GREY {
			panic(fmt.Sprintf("cyclic dependency found on var %c", v))
		}

		visited[v] = GREY
		for d := range dependencies[v] {
			topoVisit(d)
			res = append(res, d)
		}
		res = append(res, v)

		visited[v] = BLACK
	}

	for v := range dependencies {
		if visited[v] == WHITE {
			topoVisit(v)
		}
	}

	return res
}
