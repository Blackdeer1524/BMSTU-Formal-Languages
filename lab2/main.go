package main

import (
	"fmt"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/gluskov"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/parser"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
)

func main() {
	fmt.Println("start")

	regGenerator, _ := reggen.New(1, 5, 3, 15)

	regexes := regGenerator.Generate()

	fmt.Println("regex:", regexes[0])

	regex := regexes[0]

	_ = parser.ParseRegexInDot(regex)

	tree, err := parser.ParseRegex(regex)

	if err != nil {
		fmt.Println("беда в парсере", err)
	}

	automaton := gluskov.Translate(tree)

	err = automaton.GetDotMachine()
	if err != nil {
		fmt.Println("беда с визуализацией автомата", err)
	}

	fmt.Printf("%+v\n", automaton)

}
