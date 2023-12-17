package main

import (
	"bufio"
	"fmt"
	"os"

	"LL1/internal/input"
	"LL1/pkg/parsing"
)

// TODO: nonterm reachability check

func main() {
	f, err := os.Open("input.txt")
	if err != nil {
		panic(err)
	}

	stratStr := os.Getenv("STRAT")
	if len(stratStr) == 0 {
		panic("expected STRAT environment variable")
	}

	var strat bool
	if stratStr == "simple" {
		strat = false
	} else if stratStr == "greedy" {
		strat = true
	} else {
		panic(fmt.Sprintf("unknown strategy: ", stratStr))
	}

	reader := bufio.NewReader(f)
	w0, w1, info := input.ParseInput(reader)
	table := parsing.BuildTable(info)

	p := parsing.NewLL1Parser(table, info.Terms)

	T0 := p.BuildTree(w0)
	T1 := parsing.Incremental(w0, T0, w1, info, strat)

	fmt.Println("T0\n", T0.Print(4))
	fmt.Println("T1\n", T1.Print(4))

	println(table)
}
