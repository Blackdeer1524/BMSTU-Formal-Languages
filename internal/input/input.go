package input

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"strings"
	"unicode"
	"unicode/utf8"

	"LL1/internal/utils"
	"LL1/pkg/parsing"
)

// ParseInput parses input from a given reader, then returns grammar info
func ParseInput(
	r *bufio.Reader,
) (info parsing.GrammarInfo) {
	info.Productions = make(map[rune][]string)
	info.Terms = make(map[rune]struct{})
	info.Terms[parsing.EOS] = struct{}{}

	nonTerminalsStr, err := r.ReadString('\n')
	if err != nil {
		panic(err)
	}

	for _, term := range strings.Split(nonTerminalsStr, ",") {
		term = strings.TrimFunc(term, unicode.IsSpace)
		if utf8.RuneCountInString(term) != 1 {
			log.Println(
				"wrong terminal declaration [1]: terminal length has to be 1",
			)
			continue
		}

		info.Terms[utils.ExtractFirstRune(term)] = struct{}{}
	}

	lineNumber := 1
	for {
		lineNumber++

		productionStr, err := r.ReadString('\n')
		if len(productionStr) > 0 &&
			productionStr[len(productionStr)-1] == '\n' {
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
			log.Println(
				"wrong production [",
				lineNumber,
				"]: separator not found",
			)
			continue
		}

		lhs := strings.Trim(splitRes[0], " ")
		if utf8.RuneCountInString(lhs) != 1 {
			log.Println(
				"wrong production [",
				lineNumber,
				"]: length of left hand side has to be 1",
			)
			continue
		}
		varName := utils.ExtractFirstRune(lhs)

		if _, ok := info.Productions[varName]; ok {
			log.Println(
				"warning [",
				lineNumber,
				"]: nonterm with name",
				varName,
				" already defined, thus overwriting",
			)
		}
		info.Productions[varName] = make([]string, 0)

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

			if varName == parsing.START {
				rightProductions[i] = fmt.Sprintf(
					"%s%c",
					rightProductions[i],
					parsing.EOS,
				)
			}
			info.Productions[varName] = append(
				info.Productions[varName],
				rightProductions[i],
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
