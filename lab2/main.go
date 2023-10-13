package main

import (
	"fmt"
	"github.com/VyacheslavIsWorkingNow/tfl/lab2/internal/wordgen"
)

func main() {
	fmt.Println("start")

	// TODO: сканирую параметры для генерации регулярок
	// переношу в reggen.New()
	//regGenerator, _ := reggen.New(3, 5, 3, 15)
	//regexes := regGenerator.Generate()
	//

	regex := "((a(abc)*)b*)(c)******"

	// TODO: можно сканировать параметры для генерации слов в регулярках (максимальное число накачки и число слов)

	words, err := wordgen.GenerateWordsForRegex(regex, 15, 5)
	if err != nil {
		fmt.Println("беда в генерации слов")
	}

	fmt.Println("words")
	for i, w := range words {
		fmt.Println(i+1, ":", w)
	}

}
