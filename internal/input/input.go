package input

import (
	"bufio"
	"io"
	"log"
	"strconv"
	"strings"
	"unicode"
	"unicode/utf8"

	"LL1/pkg/parsing"
)

// ParseInput parses input from a given reader, then returns grammar info
func ParseInput(
	r *bufio.Reader,
) (w0 string, w1 string, info parsing.GrammarInfo) {
	info = parsing.NewGrammarInfo()

	nonTerminalsStr, err := r.ReadString('\n')
	nonTerminalsStr = strings.TrimFunc(nonTerminalsStr, unicode.IsSpace)

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

		info.Terms[term] = struct{}{}
	}

	lineNumber := 1
	for {
		lineNumber++

		productionStr, err := r.ReadString('\n')
		productionStr = strings.TrimFunc(productionStr, unicode.IsSpace)

		if err == io.EOF {
			panic("no input string was given to parse")
		} else if err != nil {
			panic(err)
		}

		if len(productionStr) == 0 {
			break
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

		if _, ok := info.Productions[lhs]; ok {
			log.Println(
				"warning [",
				lineNumber,
				"]: nonterm with name",
				lhs,
				" already defined, thus overwriting",
			)
		}
		info.Productions[lhs] = [][]string{}

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

			vars := strings.Split(rightProductions[i], " ")

			if lhs == parsing.START {
				vars = append(vars, parsing.EOS)
			}
			info.Productions[lhs] = append(
				info.Productions[lhs],
				vars,
			)
		}
	}

	inputLine, err := r.ReadString('\n')
	inputLine = strings.TrimFunc(inputLine, unicode.IsSpace)

	if err == io.EOF {
		panic("expected number of lines of w_0")
	} else if err != nil {
		panic(err)
	}
	w0Size, err := strconv.ParseInt(inputLine, 10, 64)
	if err != nil {
		panic(err)
	}

	var input strings.Builder
	for i := 0; i < int(w0Size); i++ {
		inputLine, err = r.ReadString('\n')
		inputLine = strings.TrimFunc(inputLine, unicode.IsSpace)

		if err == io.EOF {
			panic("expected more input for w_0")
		} else if err != nil {
			panic(err)
		}
		input.WriteString(inputLine)
	}
	w0 = input.String()
	input.Reset()

	inputLine, err = r.ReadString('\n')
	inputLine = strings.TrimFunc(inputLine, unicode.IsSpace)

	if err == io.EOF {
		panic("expected number of lines of w_1")
	} else if err != nil {
		panic(err)
	}
	w1Size, err := strconv.ParseInt(inputLine, 10, 64)
	if err != nil {
		panic(err)
	}

	for i := 0; i < int(w1Size); i++ {
		inputLine, err = r.ReadString('\n')
		inputLine = strings.TrimFunc(inputLine, unicode.IsSpace)

		if err == io.EOF {
			panic("expected more input for w_0")
		} else if err != nil {
			panic(err)
		}
		input.WriteString(inputLine)
	}
	w1 = input.String()
	return
}
