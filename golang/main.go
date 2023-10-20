package main

import (
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/benchmark"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
)

const (
	countWords  = 100
	maxDumpSize = 200
)

// TODO: можно сделать красивый логер для всего этого
// TODO: поменять в эквивалентности генерацию слов без последней буквы

func main() {
	//var simplifierPath string
	//var countRegex, alphabetSize, starHeight, letterCount int
	//
	//flag.StringVar(&simplifierPath, "binpath", "", "regex simplifier binary path")
	//flag.IntVar(&countRegex, "regex-n", 1, "number of regexes")
	//flag.IntVar(&alphabetSize, "alphabet-size", 1, "alphabet size")
	//flag.IntVar(&starHeight, "max-height", 1, "max star height")
	//flag.IntVar(&letterCount, "max-len", 1, "max regex length")
	//flag.Parse()
	//
	//if len(os.Args) == 1 {
	//	flag.Usage()
	//	return
	//}
	//if letterCount <= 0 {
	//	panic("max regex length cannot be <= 0")
	//}
	//if alphabetSize <= 0 {
	//	panic("alphabet size cannot be <= 0")
	//}

	countRegex, alphabetSize, starHeight, letterCount := 100, 10, 10, 100

	regGenerator, _ := reggen.New(countRegex, alphabetSize, starHeight, letterCount)
	simplifierPath := "/Users/slavaruswarrior/Documents/GitHub/BMSTU-Formal-Languages/rust/target/release/lab2"

	eErr := benchmark.EquivalenceCheck(regGenerator, simplifierPath, 1, 10)
	if eErr != nil {
		panic("error: " + eErr.Error())
	}

	err := benchmark.Start(regGenerator, simplifierPath, countWords, maxDumpSize)
	if err != nil {
		panic("error: " + err.Error())
	}

}
