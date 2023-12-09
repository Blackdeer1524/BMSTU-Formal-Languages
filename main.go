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
	EOL     = '$'
	START   = 'S'
)

func parseInput(r *bufio.Reader) (terms map[rune]struct{}, nonTerms map[rune][]string) {
	nonTerms = make(map[rune][]string)
	terms = make(map[rune]struct{})
	terms[EOL] = struct{}{}

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
		nonTerms[varName] = make([]string, 0)

		rhs := strings.Trim(splitRes[1], " ")
		rightProductions := strings.Split(rhs, "|")
		for i := range rightProductions {
			rightProductions[i] = strings.Trim(rightProductions[i], " ")
			if len(rightProductions[i]) == 0 {
				log.Println("warning [", lineNumber, "]:", i, "th production is empty")
				continue
			}

			if varName == START {
				rightProductions[i] = fmt.Sprintf("%s%c", rightProductions[i], EOL)
			}
			nonTerms[varName] = append(nonTerms[varName], rightProductions[i])
		}

		if _, ok := nonTerms[varName]; ok {
			log.Println(
				"warning [",
				lineNumber,
				"]: nonterm with name",
				varName,
				" already defined, thus overwriting",
			)
		}

		if err != nil {
			if err == io.EOF {
				return
			}
			panic(err)
		}
	}
}

type NonTerminalInfo struct {
	isNullGenerating bool
	// productions      []string
	// nextTerminals    []rune
	// nextVariables    []rune
}

func newNonTerminalInfo() *NonTerminalInfo {
	return &NonTerminalInfo{
		// productions:      make([]string, 0),
		// nextTerminals:    make([]rune, 0),
		// nextVariables:    make([]rune, 0),
		isNullGenerating: false,
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
	// TODO: (Optional) recursion check (when there are still grey nodes)
	visited := make(map[rune]color, len(productions))

	for key := range productions {
		res[key] = false
		visited[key] = WHITE
	}

	var visitVar func(variable rune) bool

	visitVar = func(variable rune) bool {
		if _, ok := res[variable]; !ok {
			panic(fmt.Sprintf("unknows variable: %c", variable))
		}

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
					if c != EOL {
						isEpsProduction = false
					}
					continue
				}
				isEpsProduction = isEpsProduction && visitVar(c)
			}

			isNullGenerating = isNullGenerating || isEpsProduction
		}

		visited[variable] = BLACK

		res[variable] = isNullGenerating
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

func First(
	terms map[rune]struct{},
	productions map[rune][]string,
	epsInfo map[rune]bool,
) map[rune]map[rune]struct{} {
	res := make(map[rune]map[rune]struct{})

	visited := make(map[rune]struct{})
	var varFirst func(nonterm rune)
	varFirst = func(nonterm rune) {
		if _, ok := visited[nonterm]; ok {
			return
		}
		visited[nonterm] = struct{}{}

		first := make(map[rune]struct{})
		prods := productions[nonterm]
		for _, prod := range prods {
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
			panic(fmt.Sprintf("Found cyclic dependency on var %c", v))
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

func Follow(terms map[rune]struct{},
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

func main() {
	fmt.Println("%c", EPSILON)
	reader := bufio.NewReader(os.Stdin)
	terms, productions := parseInput(reader)
}
