package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"os"
	"strings"
	"unicode"
	"unicode/utf8"
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
		term = strings.TrimFunc(term, unicode.IsSpace)
		if utf8.RuneCountInString(term) != 1 {
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
		if len(productionStr) == 0 {
			if err == nil {
				continue
			} else if err == io.EOF {
				return
			}
		}

		splitRes := strings.SplitN(productionStr, "->", 2)
		if len(splitRes) != 2 {
			log.Println("wrong production [", lineNumber, "]: separator not found")
			continue
		}

		lhs := strings.Trim(splitRes[0], " ")
		if utf8.RuneCountInString(lhs) != 1 {
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

func main() {
	fmt.Println(EPSILON)
	f, err := os.Open("input.txt")
	if err != nil {
		panic(err)
	}

	reader := bufio.NewReader(f)
	terms, productions := parseInput(reader)
	table := BuildTable(terms, productions)
	println(table)
}
