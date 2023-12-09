package parsing

import (
	"fmt"
	"unicode/utf8"

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
			if utf8.RuneCountInString(prod) == 1 &&
				utils.ExtractFirstRune(prod) == EPSILON {
				isNullGenerating = true
				break
			}

			isEpsProduction := true
			for _, c := range prod {
				if _, ok := info.Terms[c]; ok {
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

	for nonTerm := range info.Productions {
		if visited[nonTerm] == WHITE {
			visitVar(nonTerm)
		}
	}

	return res
}

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
			if utf8.RuneCountInString(prod) == 1 &&
				utils.ExtractFirstRune(prod) == EPSILON {
				continue
			}

			for _, c := range prod {
				if _, ok := info.Terms[c]; ok {
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

	c := make(chan rune)
	done := make(chan struct{})
	for v, prods := range info.Productions {
		for _, prod := range prods {
			if utf8.RuneCountInString(prod) == 1 {
				f := utils.ExtractFirstRune(prod)
				dependencies[f][v] = struct{}{}
				continue
			}

			// 	go pairIter(prod, c, done)
			// 	var f, s rune
			// 	s = <-c
			// 	suffixStart := 1
			// L:
			// 	for {
			// 		f = s
			// 		select {
			// 		case s = <-c:
			// 		case <-done:
			// 			break L
			// 		}
			// 		suffixStart++
			//
			// 		if _, ok := info.Terms[f]; ok {
			// 			continue
			// 		}
			//
			// 		if _, ok := info.Terms[s]; ok {
			// 			followSets[f][s] = struct{}{}
			// 		} else {
			// 			suffix := prod[suffixStart:]
			// 			utils.MergeInPlace(followSets[f], tInfo(suffix, info.Terms, epsInfo, firstInfo))
			//
			// 			utils.MergeInPlace(followSets[f], firstInfo[s])
			// 			delete(followSets[f], EPSILON)
			//
			// 			strInfo(prod[suffixStart:], info.Terms, epsInfo, firstInfo)
			// 			if epsInfo[s] {
			// 				dependencies[f][s] = struct{}{}
			// 			}
			// 		}
			// 	}

			for i, f := range prod {
			}

			for i := 0; i < len(prod)-1; i++ {
				f, s = utils.ExtractPair(prod[i : i+2])
				if _, ok := info.Terms[f]; ok {
					continue
				}

				if _, ok := info.Terms[s]; ok {
					followSets[f][s] = struct{}{}
				} else {
					utils.MergeInPlace(followSets[f], firstInfo[s])
					delete(followSets[f], EPSILON)

					// strInfo(alpha string, terms map[rune]struct{}, epsInfo map[rune]bool, firstInfo map[rune]map[rune]struct{})
					// if epsInfo[s] {
					// 	dependencies[f][s] = struct{}{}
					// }
				}
			}
			if _, ok := info.Terms[s]; !ok {
				dependencies[s][v] = struct{}{}
			}
		}
	}

	order := utils.TopoSort(dependencies)
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

		utils.MergeInPlace(first, firstInfo[c])
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

// BuildTable Builds a table for LL(1) parser
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
			firstSet, eps := strInfo(prod, info.Terms, epsInfo, firstInfo)
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
