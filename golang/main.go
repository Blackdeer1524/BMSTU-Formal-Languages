package main

import (
	"flag"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/benchmark"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
	"os"
)

const (
	benchCountWords       = 100
	benchMaxDumpSize      = 200
	equivalenceCountWords = 1

	equivalenceMaxDumpSize = 5
)

func main() {
	var simplifierPath string
	var countRegex, alphabetSize, starHeight, letterCount int

	flag.StringVar(&simplifierPath, "binpath", "", "regex simplifier binary path")
	flag.IntVar(&countRegex, "regex-n", 1, "number of regexes")
	flag.IntVar(&alphabetSize, "alphabet-size", 1, "alphabet size")
	flag.IntVar(&starHeight, "max-height", 1, "max star height")
	flag.IntVar(&letterCount, "max-len", 1, "max regex length")
	flag.Parse()

	if len(os.Args) == 1 {
		flag.Usage()
		return
	}
	if letterCount <= 0 {
		panic("max regex length cannot be <= 0")
	}
	if alphabetSize <= 0 {
		panic("alphabet size cannot be <= 0")
	}

	regGenerator, _ := reggen.New(countRegex, alphabetSize, starHeight, letterCount)

	sErr := benchmark.Start(regGenerator, simplifierPath,
		benchCountWords, benchMaxDumpSize,
		equivalenceCountWords, equivalenceMaxDumpSize,
	)
	if sErr != nil {
		panic("error: " + sErr.Error())
	}

}
