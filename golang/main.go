package main

import (
	"fmt"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/benchmark"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
)

const (
	rustBinaryPath = "/Users/slavaruswarrior/Documents/GitHub/BMSTU-Formal-Languages/rust/target/release/lab2"
	countWords     = 100
	maxDumpSize    = 2000
)

// TODO: можно сделать красивый логер для всего этого
// и это было бы приятным дополнением
// TODO: для увеличения скорости построения слов перейти на strings.Builder

func main() {
	fmt.Println("start fuzzing")
	fmt.Println("Enter parametres for generating regular expressions:")

	// Например, можно использовать reggen.New(10, 5, 4, 30)

	var countRegex, alphabetSize, starHeight, letterCount int

	fmt.Println("Regex count:")
	_, _ = fmt.Scanf("%d\n", &countRegex)
	fmt.Println("Alphabet size:")
	_, _ = fmt.Scanf("%d\n", &alphabetSize)
	fmt.Println("Star height:")
	_, _ = fmt.Scanf("%d\n", &starHeight)
	fmt.Println("Letter count:")
	_, _ = fmt.Scanf("%d\n", &letterCount)

	fmt.Println("Your parametres:", countRegex, alphabetSize, starHeight, letterCount)

	regGenerator, _ := reggen.New(countRegex, alphabetSize, starHeight, letterCount)

	err := benchmark.Start(regGenerator, rustBinaryPath, countWords, maxDumpSize)
	if err != nil {
		fmt.Println("ОШИБКА:", err)
	}
}
