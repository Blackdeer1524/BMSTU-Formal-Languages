package parsing

import (
	"fmt"

	"LL1/internal/graphs"
	"LL1/internal/utils"
)

type GrammarInfo struct {
	Terms       map[rune]struct{}
	Productions map[rune][]string
}

const (
	EPSILON = 'ε'
	EOS     = '$'
	START   = 'S'
)

func getEpsInfo(info GrammarInfo) (res map[rune]bool) {
	res = make(map[rune]bool, len(info.Productions))

	type color int
	const (
		WHITE color = iota // haven't seen yet
		GREY               // yet to finish processing
		BLACK              // processed
	)
	visited := make(map[rune]color, len(info.Productions))

	for key := range info.Productions {
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
		for _, prod := range info.Productions[variable] {
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
		res[variable] = isNullGenerating
		visited[variable] = BLACK
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
		for _, prod := range info.Productions[nonterm] {
			for _, c := range prod {
				if _, ok := info.Terms[c]; ok {
					if c == EPSILON {
						continue
					}
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

	for variable := range info.Productions {
		varFirst(variable)
	}

	return res
}

func pairIter(s string, c chan<- rune, done chan<- struct{}) {
	for _, chr := range s {
		c <- chr
	}
	done <- struct{}{}
}

func getFollowInfo(
	info GrammarInfo,
	epsInfo map[rune]bool,
	firstInfo map[rune]map[rune]struct{},
) map[rune]map[rune]struct{} {
	followSets := make(map[rune]map[rune]struct{})
	dependencies := make(map[rune]map[rune]struct{})
	for variable := range info.Productions {
		followSets[variable] = make(map[rune]struct{})
		dependencies[variable] = make(map[rune]struct{})
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
	alpha string,
	terms map[rune]struct{},
	epsInfo map[rune]bool,
	firstInfo map[rune]map[rune]struct{},
) (first map[rune]struct{}) {
	first = make(map[rune]struct{})
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

// BuildTable builds a table for LL(1) parser
//
// Table = Variables x Terminals
func BuildTable(info GrammarInfo) map[rune]map[rune]string {
	epsInfo := getEpsInfo(info)
	firstInfo := getFirstInfo(info, epsInfo)
	printFirstTable(firstInfo)
	followInfo := getFollowInfo(info, epsInfo, firstInfo)

	res := make(map[rune]map[rune]string, len(info.Productions))
	for v := range info.Productions {
		res[v] = make(map[rune]string, len(info.Terms))
		for t := range info.Terms {
			res[v][t] = ""
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
