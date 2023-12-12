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

	reader := bufio.NewReader(f)
	w0, w1, info := input.ParseInput(reader)
	table := parsing.BuildTable(info)

	p := parsing.NewLL1Parser(table, info.Terms)

	T0 := p.BuildTree(w0)
	T1 := parsing.Incremental(w0, T0, w1, info, true)

	fmt.Println("T0: ", T0.Debug())
	fmt.Println("T1: ", T1.Debug())

	println(table)
}
