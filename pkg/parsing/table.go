package parsing

import (
	"fmt"

	"LL1/internal/graphs"
	"LL1/internal/utils"
)

type GrammarInfo struct {
	Terms       map[string]struct{}
	Productions map[string][][]string
}

const (
	EPSILON = "ε"
	EOS     = "$"
	START   = "S"
)

func getEpsInfo(info GrammarInfo) (res map[string]bool) {
	res = make(map[string]bool, len(info.Productions))

	type color int
	const (
		WHITE color = iota // haven't seen yet
		GREY               // yet to finish processing
		BLACK              // processed
	)
	visited := make(map[string]color, len(info.Productions))

	for key := range info.Productions {
		res[key] = false
		visited[key] = WHITE
	}

	var visitVar func(v string) bool
	visitVar = func(v string) bool {
		if visited[v] == GREY {
			return true
		} else if visited[v] == BLACK {
			return res[v]
		}
		visited[v] = GREY

		isNullGenerating := false
		for _, prod := range info.Productions[v] {
			isEpsProduction := true
			for _, c := range prod {
				if _, ok := info.Terms[c]; ok {
					if c == EPSILON {
						continue
					}
					isEpsProduction = false
					break
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
		res[v] = isNullGenerating
		visited[v] = BLACK
		return isNullGenerating
	}

	for nonTerm := range info.Productions {
		if visited[nonTerm] == WHITE {
			visitVar(nonTerm)
		}
	}

	return res
}

// getFirstInfo provedes intem from FIRST set
func getFirstInfo(
	info GrammarInfo,
	epsInfo map[string]bool,
) map[string]map[string]struct{} {
	res := make(map[string]map[string]struct{})

	dependencies := make(map[string]map[string]struct{})
	for variable := range info.Productions {
		res[variable] = map[string]struct{}{}
		dependencies[variable] = make(map[string]struct{})
	}

	for v, prods := range info.Productions {
		for _, prod := range prods {
			for _, c := range prod {
				if _, ok := info.Terms[c]; ok {
					res[v][c] = struct{}{}
					if c == EPSILON {
						continue
					}
					break
				}

				dependencies[v][c] = struct{}{}
				if !epsInfo[c] {
					break
				}

			}
		}
	}

	order := graphs.TopoSort(dependencies)
	for _, v := range order {
		for dep := range dependencies[v] {
			utils.MergeInPlace(res[v], res[dep])
		}
	}

	return res
}

func getFollowInfo(
	info GrammarInfo,
	epsInfo map[string]bool,
	firstInfo map[string]map[string]struct{},
) map[string]map[string]struct{} {
	followSets := make(map[string]map[string]struct{})
	dependencies := make(map[string]map[string]struct{})
	for variable := range info.Productions {
		followSets[variable] = make(map[string]struct{})
		dependencies[variable] = make(map[string]struct{})
	}

	for v, prods := range info.Productions {
		for _, prod := range prods {
			suffix := prod
			for _, f := range prod {
				suffix = suffix[1:]
				if _, ok := info.Terms[f]; ok {
					continue
				}
				suffixFrist := strInfo(suffix, info.Terms, epsInfo, firstInfo)
				var producesEps bool
				if _, ok := suffixFrist[EPSILON]; ok {
					producesEps = true
				}
				delete(suffixFrist, EPSILON)

				utils.MergeInPlace(followSets[f], suffixFrist)
				if producesEps || len(suffixFrist) == 0 {
					dependencies[f][v] = struct{}{}
				}

			}
		}
	}

	order := graphs.TopoSort(dependencies)
	for _, v := range order {
		for dep := range dependencies[v] {
			utils.MergeInPlace(followSets[v], followSets[dep])
			delete(followSets[v], EPSILON)
		}
	}

	return followSets
}

func strInfo(
	alpha []string,
	terms map[string]struct{},
	epsInfo map[string]bool,
	firstInfo map[string]map[string]struct{},
) (first map[string]struct{}) {
	first = make(map[string]struct{})
	if len(alpha) == 0 {
		return
	}

	for _, c := range alpha {
		if _, ok := terms[c]; ok {
			if c == EPSILON {
				continue
			}
			first[c] = struct{}{}
			return
		}

		utils.MergeInPlace(first, firstInfo[c])
		delete(first, EPSILON)
		if !epsInfo[c] {
			return
		}
	}
	first[EPSILON] = struct{}{}
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

type Table = map[string]map[string][]string

// BuildTable builds a table for LL(1) parser
//
// Table = Variables x Terminals
func BuildTable(info GrammarInfo) Table {
	epsInfo := getEpsInfo(info)
	firstInfo := getFirstInfo(info, epsInfo)
	followInfo := getFollowInfo(info, epsInfo, firstInfo)

	res := make(Table, len(info.Productions))
	for v := range info.Productions {
		res[v] = make(map[string][]string, len(info.Terms))
		for t := range info.Terms {
			res[v][t] = []string{}
		}
	}

	for v, prods := range info.Productions {
		for _, prod := range prods {
			// https://web.cs.wpi.edu/~kal/PLT/PLT4.3.html
			firstSet := strInfo(prod, info.Terms, epsInfo, firstInfo)
			var eps bool
			if _, ok := firstSet[EPSILON]; ok {
				eps = true
			}
			delete(firstSet, EPSILON)
			for a := range firstSet {
				if len(res[v][a]) != 0 {
					panic("conflict found")
				}
				res[v][a] = prod
			}
			if eps {
				if _, ok := firstSet[EOS]; ok {
					res[v][EOS] = append(res[v][EOS], EPSILON)
				} else {
					for b := range followInfo[v] {
						res[v][b] = append(res[v][b], EPSILON)
					}
				}
			}
		}
	}

	return res
}
