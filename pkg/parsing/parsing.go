package parsing

import (
	"fmt"
	"unicode/utf8"
)

const (
	EPSILON = 'ε'
	EOS     = '$'
	START   = 'S'
)

func getEpsInfo(terms map[rune]struct{}, productions map[rune][]string) (res map[rune]bool) {
	res = make(map[rune]bool, len(productions))

	type color int
	const (
		WHITE color = iota // haven't seen yet
		GREY               // yet to finish processing
		BLACK              // processed
	)
	visited := make(map[rune]color, len(productions))

	for key := range productions {
		res[key] = false
		visited[key] = WHITE
	}

	var visitVar func(variable rune) bool
	visitVar = func(variable rune) bool {
		if visited[variable] == GREY {
			return true
		} else if visited[variable] == BLACK {
			return res[variable]
		}
		visited[variable] = GREY

		isNullGenerating := false
		for _, prod := range productions[variable] {
			if utf8.RuneCountInString(prod) == 1 && extractFirstRune(prod) == EPSILON {
				isNullGenerating = true
				break
			}

			isEpsProduction := true
			for _, c := range prod {
				if _, ok := terms[c]; ok {
					isEpsProduction = false
					continue
				}
				isEpsProduction = isEpsProduction && visitVar(c)
				if !isEpsProduction {
					break
				}
			}

			isNullGenerating = isNullGenerating || isEpsProduction
			if isNullGenerating {
				break
			}
		}
		res[variable] = isNullGenerating
		visited[variable] = BLACK
		return isNullGenerating
	}

	for nonTerm := range productions {
		if visited[nonTerm] == WHITE {
			visitVar(nonTerm)
		}
	}

	return res
}

func getFirstInfo(
	terms map[rune]struct{},
	productions map[rune][]string,
	epsInfo map[rune]bool,
) map[rune]map[rune]struct{} {
	res := make(map[rune]map[rune]struct{})

	// NOTE: cyclic dependencies detection is needed, but who cares?
	visited := make(map[rune]struct{})

	var varFirst func(nonterm rune)
	varFirst = func(nonterm rune) {
		if _, ok := visited[nonterm]; ok {
			return
		}
		visited[nonterm] = struct{}{}

		first := make(map[rune]struct{})
		if epsInfo[nonterm] {
			first[EPSILON] = struct{}{}
		}
		for _, prod := range productions[nonterm] {
			if utf8.RuneCountInString(prod) == 1 && extractFirstRune(prod) == EPSILON {
				continue
			}

			for _, c := range prod {
				if _, ok := terms[c]; ok {
					first[c] = struct{}{}
					break
				}

				varFirst(c)
				for n := range res[c] {
					first[n] = struct{}{}
				}

				if !epsInfo[c] {
					break
				}
			}
		}
		res[nonterm] = first
	}

	for variable := range productions {
		varFirst(variable)
	}

	return res
}

func getFollowInfo(
	terms map[rune]struct{},
	productions map[rune][]string,
	epsInfo map[rune]bool,
	firstInfo map[rune]map[rune]struct{},
) map[rune]map[rune]struct{} {
	followSets := make(map[rune]map[rune]struct{})
	dependencies := make(map[rune]map[rune]struct{})
	for variable := range productions {
		followSets[variable] = make(map[rune]struct{})
		dependencies[variable] = make(map[rune]struct{})
	}

	for _, prods := range productions {
		for _, prod := range prods {
			for i := 0; i < len(prod)-1; i++ {
				f, s := extractPair(prod[i : i+2])
				if _, ok := terms[f]; ok {
					continue
				}

				if _, ok := terms[s]; ok {
					followSets[f][s] = struct{}{}
				} else {
					mergeInPlace(followSets[f], firstInfo[s])
					if epsInfo[s] {
						dependencies[f][s] = struct{}{}
					}
				}
			}
		}
	}

	order := topoSort(dependencies)
	for _, v := range order {
		for dep := range dependencies[v] {
			mergeInPlace(followSets[v], followSets[dep])
		}
	}

	return followSets
}

func strInfo(
	alpha string,
	terms map[rune]struct{},
	epsInfo map[rune]bool,
	firstInfo map[rune]map[rune]struct{},
) (first map[rune]struct{}, eps bool) {
	first = make(map[rune]struct{})
	eps = false
	if len(alpha) == 0 {
		return
	}

	for _, c := range alpha {
		if _, ok := terms[c]; ok {
			first[c] = struct{}{}
			return
		}
		if c == EPSILON {
			eps = true
			continue
		}

		mergeInPlace(first, firstInfo[c])
		delete(first, EPSILON)

		if !epsInfo[c] {
			return
		}
	}
	eps = true
	return
}

func printFirstTable(first map[rune]map[rune]struct{}) {
	for v, next := range first {
		fmt.Printf("Follow(%c): {", v)
		for n := range next {
			fmt.Printf("%c, ", n)
		}
		fmt.Printf("}\n")
	}
}

func BuildTable(
	terms map[rune]struct{},
	productions map[rune][]string,
) map[rune]map[rune]string {
	epsInfo := getEpsInfo(terms, productions)
	firstInfo := getFirstInfo(terms, productions, epsInfo)
	printFirstTable(firstInfo)
	followInfo := getFollowInfo(terms, productions, epsInfo, firstInfo)

	res := make(map[rune]map[rune]string, len(productions))
	for v := range productions {
		res[v] = make(map[rune]string, len(terms))
		for t := range terms {
			res[v][t] = ""
		}
	}

	for v, prods := range productions {
		for _, prod := range prods {
			// https://web.cs.wpi.edu/~kal/PLT/PLT4.3.html
			firstSet, eps := strInfo(prod, terms, epsInfo, firstInfo)
			for a := range firstSet {
				if res[v][a] != "" {
					panic("conflict found")
				}
				res[v][a] = prod
			}
			if eps {
				if _, ok := firstSet[EOS]; ok {
					res[v][EOS] = string(EPSILON)
				} else {
					for b := range followInfo[v] {
						res[v][b] = string(EPSILON)
					}
				}
			}
		}
	}

	return res
}
