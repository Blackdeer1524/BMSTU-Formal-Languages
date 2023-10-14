package main

import (
	"fmt"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/benchmark"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
)

const (
	rustBinaryPath = "/Users/slavaruswarrior/Documents/GitHub/BMSTU-Formal-Languages/rust/target/release/lab2"
	countWords     = 10
	maxDumpSize    = 5
)

// TODO: можно сделать красивый логер для всего этого
// и это было бы приятным дополнением

func main() {
	fmt.Println("start")

	// TODO: сканирую параметры для генерации регулярок
	regGenerator, _ := reggen.New(10, 5, 4, 30)

	err := benchmark.Start(regGenerator, rustBinaryPath, countWords, maxDumpSize)
	if err != nil {
		fmt.Println("ОШИБКА", err)
	}
}
