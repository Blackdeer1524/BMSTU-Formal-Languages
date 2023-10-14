package main

import (
	"fmt"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/bencmark"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/reggen"
)

const rustBinaryPath = "/Users/slavaruswarrior/Documents/GitHub/BMSTU-Formal-Languages/rust/target/release/lab2"

func main() {
	fmt.Println("start")

	// TODO: сканирую параметры для генерации регулярок
	regGenerator, _ := reggen.New(10, 5, 4, 30)

	err := bencmark.Start(regGenerator, rustBinaryPath)
	if err != nil {
		fmt.Println("ОШИБКА", err)
	}
}
