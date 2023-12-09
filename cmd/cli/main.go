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
