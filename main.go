package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"os"
	"strings"
)

func extractFirstRune(s string) rune {
	var first rune
	for _, c := range s {
		first = c
		break
	}
	return first
}

const (
	EPSILON = 'ε'
	EOS     = '$'
	START   = 'S'
)

func parseInput(r *bufio.Reader) (terms map[rune]struct{}, var2productions map[rune][]string) {
	var2productions = make(map[rune][]string)

	terms = make(map[rune]struct{})
	terms[EOS] = struct{}{}

	nonTerminalsStr, err := r.ReadString('\n')
	if err != nil {
		panic(err)
	}

	for _, term := range strings.Split(nonTerminalsStr, ",") {
		term = strings.Trim(term, " ")
		if len(term) != 1 {
			log.Println("wrong terminal declaration [1]: terminal length has to be 1")
			continue
		}

		terms[extractFirstRune(term)] = struct{}{}
	}

	lineNumber := 1
	for {
		lineNumber += 1

		productionStr, err := r.ReadString('\n')
		if len(productionStr) > 0 && productionStr[len(productionStr)-1] == '\n' {
			productionStr = productionStr[:len(productionStr)-1]
		}
		if len(productionStr) == 0 && err == nil {
			continue
		}

		splitRes := strings.SplitN(productionStr, "->", 2)
		if len(splitRes) != 2 {
			log.Println("wrong production [", lineNumber, "]: separator not found")
			continue
		}

		lhs := strings.Trim(splitRes[0], " ")
		if len(lhs) != 1 {
			log.Println("wrong production [", lineNumber, "]: length of left hand side has to be 1")
			continue
		}
		varName := extractFirstRune(lhs)

		if _, ok := var2productions[varName]; ok {
			log.Println(
				"warning [",
				lineNumber,
				"]: nonterm with name",
				varName,
				" already defined, thus overwriting",
			)
		}
		var2productions[varName] = make([]string, 0)

		rhs := splitRes[1]
		rightProductions := strings.Split(rhs, "|")
		for i := range rightProductions {
			rightProductions[i] = strings.Trim(rightProductions[i], " ")
			if len(rightProductions[i]) == 0 {
				log.Println(
					"warning [",
					lineNumber,
					"]:",
					i,
					"th production is empty. To set up ε-production you have to use 'ε' explicitly",
				)
				continue
			}

			if varName == START {
				rightProductions[i] = fmt.Sprintf("%s%c", rightProductions[i], EOS)
			}
			var2productions[varName] = append(var2productions[varName], rightProductions[i])
		}

		if err != nil {
			if err == io.EOF {
				return
			}
			panic(err)
		}
	}
}

// TODO: nonterm reachability check

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
			if len(prod) == 1 && extractFirstRune(prod) == EPSILON {
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
		if _, ok := visited[nonTerm]; ok {
			continue
		}
		visitVar(nonTerm)
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
		for _, prod := range productions[nonterm] {
			if len(prod) == 1 && extractFirstRune(prod) == EPSILON {
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
	followInfo map[rune]map[rune]struct{},
) (first map[rune]struct{}, eps bool) {
	res := make(map[rune]struct{})
	if len(alpha) == 0 {
		return res, false
	}

	for _, c := range alpha {
		if _, ok := terms[c]; ok {
			res[c] = struct{}{}
			return res, false
		}
		mergeInPlace(res, firstInfo[c])
		if !epsInfo[c] {
			return res, false
		}
	}
	return res, true
}

func BuildTable(
	terms map[rune]struct{},
	productions map[rune][]string,
) map[rune]map[rune]string {
	epsInfo := getEpsInfo(terms, productions)
	firstInfo := getFirstInfo(terms, productions, epsInfo)
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
			firstSet, eps := strInfo(prod, terms, epsInfo, firstInfo, followInfo)
			for a := range firstSet {
				if res[v][a] != "" {
					panic("conflict found")
				}
				res[v][a] = prod
			}
			if eps {
				for b := range followInfo[v] {
					if res[v][b] != "" {
						panic("conflict found")
					}
					res[v][b] = prod
				}
			}
		}
	}

	return res
}

func main() {
	fmt.Println("%c", EPSILON)
	reader := bufio.NewReader(os.Stdin)
	terms, productions := parseInput(reader)
}
