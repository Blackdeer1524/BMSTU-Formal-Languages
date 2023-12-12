package main

import (
	"bufio"
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
	info := input.ParseInput(reader)
	table := parsing.BuildTable(info)

	parsing.NewLL1Parser(table, info.Terms)

	println(table)
}
