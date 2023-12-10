package main

import (
	"bufio"
	"fmt"
	"os"
)

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
